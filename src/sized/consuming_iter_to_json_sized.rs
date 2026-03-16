
use std::any::{Any, TypeId};
use std::cell::RefCell;
use serde::ser::{Serialize, Serializer, SerializeMap, Error};

pub trait ConsumingIterToJsonSized<'a,K,V>: IntoIterator<Item=(K,V)> where
Self: Sized,
K: Serialize + Any,
V: Serialize,
<Self as IntoIterator>::IntoIter: 'a + ExactSizeIterator
{
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
