use super::*;

/// Blanket impl [to_json_map()](trait.MapIterToJson.html#method.to_json_map) for all `IntoIterator<Item=(&K,&V)>` types.
pub trait MapIterToJson<'a,K,V>: IntoIterator<Item=(&'a K,&'a V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<(&K,&V)>` to a JSON map. This includes, but is not limited to, the following example types:  
  /// `HashMap<K,V>`  
  /// return type of `HashMap<K,V>::iter()`  
  /// `BTreeMap<K,V>`  
  /// return type of `BTreeMap<K,V>::iter()`
  /// 
  /// To create the JSON map keys, `serde_json::to_string()` will be called on each K element.
  /// 
  /// **This does not consume self**, and is not compatible with consuming iterators, such as those returned by the common
  /// `std::collections::Type::into_iter()` function. For those consuming iterators, call [into_json_map()](trait.ConsumingIterToJson.html#method.into_json_map) instead.
  ///
  /// # Examples
  /// ```
  /// use std::collections::HashMap;
  /// use serde::Serialize;
  /// use serde_json::Error;
  /// use serde_json_any_key::*;
  /// 
  /// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash)]
  /// pub struct Test {
  ///   pub a: i32,
  ///   pub b: i32
  /// }
  /// 
  /// fn try_main() -> Result<(), Error> {
  /// let mut map = HashMap::<Test, Test>::new();
  /// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  /// 
  /// // Regular serde_json cannot serialize this map.
  /// let fail = serde_json::to_string(&map);
  /// assert_eq!(fail.err().unwrap().to_string(), "key must be a string");
  /// 
  /// // Use this crate's utility function - elements are serialized lazily.
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser1 = map.to_json_map().unwrap(); // map.iter().to_json_map() is also valid
  ///
  /// // Compare to a long-winded workaround that constructs a new intermediate map.
  /// // Same output
  /// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  /// let ser2 = serde_json::to_string(&string_map).unwrap();
  ///
  /// assert_eq!(ser1, ser2);
  /// Ok(()) }
  /// try_main().unwrap();
  /// ```
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    serde_json::to_string(&SerializeMapIterWrapper {
      iter: RefCell::new(self.into_iter())
    })
  }
}

impl<'a,K,V,T> MapIterToJson<'a,K,V> for T where
T: IntoIterator<Item=(&'a K,&'a V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }

pub(crate) struct SerializeMapIterWrapper<'a,K,V,I> where
I: Iterator<Item=(&'a K,&'a V)>,
K: 'a,
V: 'a
{
  pub iter: RefCell<I>
}

impl<'a,K,V,I> Serialize for SerializeMapIterWrapper<'a,K,V,I> where
  I: Iterator<Item=(&'a K,&'a V)>,
  K: Serialize + Any,
  V: Serialize,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(None)?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    // compiler seems to be able to optimize this branch away statically
    if TypeId::of::<K>() == TypeId::of::<String>() {
      while let Some((k, v)) = iter.next() {
        let s = (k as &dyn Any).downcast_ref::<String>().ok_or(S::Error::custom("Failed to serialize String as string"))?;
        ser_map.serialize_entry(s, &v)?;
      }
    } else {
      while let Some((k, v)) = iter.next() {
        ser_map.serialize_entry(match &serde_json::to_string(&k)
        {
          Ok(key_string) => key_string,
          Err(e) => { return Err(e).map_err(S::Error::custom); }
        }, &v)?;
      }
    }
    ser_map.end()
  }
}
