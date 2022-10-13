use super::*;
use serde::de::{MapAccess};
use std::fmt;

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
