
use std::any::{Any, TypeId};
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};

/// Blanket impl [into_json_map_sized()](ConsumingIterToJsonSized::into_json_map_sized) for all `IntoIterator<Item=(K,V)> + ExactSizeIterator` types.
pub trait ConsumingIterToJsonSized<'a,K,V>: IntoIterator<Item=(K,V)> where
Self: Sized,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{
  /// Serialize any `IntoIterator<(K,V)> + ExactSizeIterator` to a JSON map.

  /// **This consumes self**, and is not compatible with non-consuming iterators, such as those returned by the common
  /// `std::collections::Type::iter()` function. For those non-consuming iterators *with known length*, call [to_json_map_sized()](crate::MapIterToJsonSized::to_json_map_sized) instead.
  /// 
  /// For further information see [`into_json_map()`](crate::ConsumingIterToJson::into_json_map).
  ///
  /// # Examples
  /// ```
  /// use std::collections::{HashMap, BTreeMap};
  /// use serde::Serialize;
  /// use serde_json::Error;
  /// use serde_json_any_key::*;
  /// 
  /// #[derive(Clone, Copy, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
  /// pub struct Test {
  ///   pub a: i32,
  ///   pub b: i32
  /// }
  /// 
  /// let mut map = HashMap::<Test, Test>::new();
  /// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  /// 
  /// let mut btr = BTreeMap::<Test, Test>::new();
  /// btr.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  ///
  /// let mut vec = Vec::<(Test, Test)>::new();
  /// vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
  /// 
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser1 = map.into_json_map_sized().unwrap(); // map.into_iter().into_json_map_sized() is also valid
  /// let ser2 = btr.into_json_map_sized().unwrap(); // btr.into_iter().into_json_map_sized() is also valid
  /// let ser3 = vec.into_json_map_sized().unwrap(); // vec.into_iter().into_json_map_sized() is also valid
  /// 
  /// // map, btr, and vec have all been consumed.
  ///
  /// assert_eq!(ser1, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
  /// assert_eq!(ser1, ser2);
  /// assert_eq!(ser1, ser3);
  /// ```
  fn into_json_map_sized(self) -> Result<String, serde_json::Error> {
    serde_json::to_string(&SerializeConsumingIterWrapperSized {
      iter: RefCell::new(self.into_iter())
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=(K,V)>> ConsumingIterToJsonSized<'a,K,V> for T where
T: IntoIterator<Item=(K,V)>,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{ }

struct SerializeConsumingIterWrapperSized<K,V,I> where
I: Iterator<Item=(K,V)>,
{
  pub iter: RefCell<I>
}

impl<K,V,I> Serialize for SerializeConsumingIterWrapperSized<K,V,I> where
  I: Iterator<Item=(K,V)> + ExactSizeIterator,
  K: Serialize + Any,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let mut ser_map = serializer.serialize_map(Some(self.iter.borrow().len()))?;
    let mut iter = self.iter.borrow_mut();
    // handle strings specially so they don't get escaped and wrapped inside another string
    // compiler seems to be able to optimize this branch away statically
    if TypeId::of::<K>() == TypeId::of::<String>() {
      while let Some((k, v)) = iter.next() {
        let s = (&k as &dyn Any).downcast_ref::<String>().ok_or(S::Error::custom("Failed to serialize String as string"))?;
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
