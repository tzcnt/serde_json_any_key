
use crate::serde_with_utils;
use std::any::Any;
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer};

/// Blanket impl [to_json_map_sized()](MapIterToJsonSized::to_json_map_sized) for all `IntoIterator<Item=(&K,&V)> + ExactSizeIterator` types.
pub trait MapIterToJsonSized<'a,K,V>: IntoIterator<Item=(&'a K,&'a V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{
  /// Serialize any `IntoIterator<(&K,&V)> + ExactSizeIterator` to a JSON map.
  /// 
  /// For further information see [to_json_map()](crate::MapIterToJson::to_json_map)
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
  /// let mut map = HashMap::<Test, Test>::new();
  /// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
  /// 
  /// let ser1 = map.to_json_map_sized().unwrap(); // map.iter().to_json_map_sized() is also valid
  ///
  /// let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
  /// let ser2 = serde_json::to_string(&string_map).unwrap();
  ///
  /// assert_eq!(ser1, ser2);
  /// ```
  fn to_json_map_sized(self) -> Result<String, serde_json::Error> {
    serde_json::to_string(&SerializeMapIterWrapperSized {
      iter: RefCell::new(self.into_iter())
    })
  }
}

impl<'a,K,V,T> MapIterToJsonSized<'a,K,V> for T where
T: IntoIterator<Item=(&'a K,&'a V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{ }

pub(crate) struct SerializeMapIterWrapperSized<'a,K,V,I> where
I: Iterator<Item=(&'a K,&'a V)> + ExactSizeIterator,
K: 'a,
V: 'a
{
  pub iter: RefCell<I>
}

impl<'a,K,V,I> Serialize for SerializeMapIterWrapperSized<'a,K,V,I> where
  I: Iterator<Item=(&'a K,&'a V)> + ExactSizeIterator,
  K: Serialize + Any,
  V: Serialize,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
    S: Serializer
  {
    let len = self.iter.borrow().len();
    serde_with_utils::serialize_iter_to_map(serializer, &self.iter, Some(len))
  }
}
