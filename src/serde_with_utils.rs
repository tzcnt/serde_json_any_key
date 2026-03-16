
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, Visitor};
use serde::de::{MapAccess};
use serde::ser::{Serialize, SerializeMap, Serializer, Error};
use std::fmt;

// Some utilities required to implement any_key_map / any_key_vec

pub struct MapIter<'de,A,K,V> {
  pub access: A,
  marker: PhantomData<(&'de (),K,V)>,
}

impl<'de,A,K,V> MapIter<'de,A,K,V> {
    pub fn new(access: A) -> Self
    where
        A: serde::de::MapAccess<'de>,
    {
        Self {
            access,
            marker: PhantomData,
        }
    }
}

impl<'de,A,K,V> Iterator for MapIter<'de,A,K,V>
where
    A: serde::de::MapAccess<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Item = Result<(K,V), A::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.access.next_entry().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.access.size_hint() {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

// any_key_map and any_key_vec use the same deserialize function
#[inline]
pub fn deserialize<'d,D,C,K,V>(deserializer: D) -> Result<C,D::Error> where
  D: Deserializer<'d>,
  C: FromIterator<(K,V)> + Sized,
  for<'de> K: Deserialize<'de> + Any + 'd,
  for<'de> V: Deserialize<'de> + 'd,
{
  struct Helper<C,K,V>(PhantomData<(C,K,V)>);
  impl<'d,C,K,V> Visitor<'d> for Helper<C,K,V>
  where
  C: FromIterator<(K,V)> + Sized,
  for<'de> K: Deserialize<'de> + Any + 'd,
  for<'de> V: Deserialize<'de> + 'd
  {
      type Value = C;

      fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          write!(formatter, "a JSON map")
      }

      fn visit_map<A>(self, seq: A) -> Result<Self::Value, A::Error>
      where
          A: MapAccess<'d>,
      {
        // https://stackoverflow.com/a/26370894/19260728
        let coll: Result<C, A::Error> = MapIter::<'d, A, String, V>::new(seq)
          .map(|res| {
            res.and_then(|value: (String,V)| {
              // handle strings specially as they are not objects
              // compiler seems to be able to optimize this branch away statically
              let key_obj: K = match TypeId::of::<K>() == TypeId::of::<String>() {
                true => match <K as Deserialize>::deserialize(serde_json::Value::from(value.0)) {
                  Ok(k) => k,
                  Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                },
                false => match serde_json::from_str(&value.0) {
                  Ok(k) => k,
                  Err(e) => { return Err(e).map_err(serde::de::Error::custom); }
                }
              };
              Ok((key_obj, value.1))
            })
          }).collect();
        coll
      }
  }
  
  deserializer.deserialize_map(Helper(PhantomData))
}

pub fn serialize_iter_to_map<'a,S,K,V,I>(serializer: S, iter: &RefCell<I>, len: Option<usize>) -> Result<S::Ok, S::Error>
where
I: Iterator<Item=(&'a K,&'a V)>,
K: 'a + Serialize + Any,
V: 'a + Serialize,
S: Serializer
{
  let mut iter = iter.borrow_mut();
  let mut ser_map = serializer.serialize_map(len)?;
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
