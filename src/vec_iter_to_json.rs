use super::*;

/// Blanket impl [to_json_map()](trait.VecIterToJson.html#method.to_json_map) for all `IntoIterator<Item=&(K,V)>` types.
pub trait VecIterToJson<'a,K,V>: IntoIterator<Item=&'a (K,V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{
  /// Serialize any `IntoIterator<&(K,V)>` to a JSON map. This includes, but is not limited to, the following example types:  
  /// `Vec<(K,V)>`  
  /// return type of `Vec<(K,V)>::iter()`  
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
  /// let mut vec = Vec::<(Test, Test)>::new();
  /// vec.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
  /// 
  /// // Regular serde_json will serialize this as an array, not a map.
  /// // Outputs [[{"a":3,"b":5},{"a":7,"b":9}]]
  /// let ser1 = serde_json::to_string(&vec).unwrap();
  /// assert_eq!(ser1, "[[{\"a\":3,\"b\":5},{\"a\":7,\"b\":9}]]");
  /// 
  /// // Use this crate's utility function - elements are serialized lazily.
  /// // Outputs {"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
  /// let ser2 = vec.to_json_map().unwrap(); // vec.iter().to_json_map() is also valid
  ///
  /// assert_eq!(ser2, "{\"{\\\"a\\\":3,\\\"b\\\":5}\":{\"a\":7,\"b\":9}}");
  /// Ok(()) }
  /// try_main().unwrap();
  /// ```
  fn to_json_map(self) -> Result<String, serde_json::Error> {
    serde_json::to_string(&crate::SerializeVecIterWrapper {
      iter: RefCell::new(self.into_iter())
    })
  }
}

impl<'a,K,V,T: IntoIterator<Item=&'a (K,V)>> VecIterToJson<'a,K,V> for T where
T: IntoIterator<Item=&'a (K,V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a
{ }
