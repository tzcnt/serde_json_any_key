use std::any::Any;
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer};

use crate::serde_with_utils::serialize_iter_to_map;

pub trait MapIterToJsonSized<'a,K,V>: IntoIterator<Item=(&'a K,&'a V)> where
Self: Sized,
K: 'a + Serialize + Any,
V: 'a + Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{
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
    serialize_iter_to_map(serializer, &self.iter, Some(len))
  }
}
