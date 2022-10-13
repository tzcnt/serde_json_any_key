
use std::any::{Any, TypeId};
use serde::ser::Error;
use serde::de::Deserialize;

/// Reverses to_json_map(), returning an `Iterator<Item=Result<(K,V), serde_json::Error>>`.
/// 
/// Note that because JSON deserialization may fail for any individual element of the map,
/// you will need to check for errors with each element returned from the iterator.
///
/// # Examples
/// ```
/// use std::collections::{BTreeMap, HashMap};
/// use serde::{Serialize, Deserialize};
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut map = HashMap::<Test, Test>::new();
/// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// let ser = map.to_json_map().unwrap();
/// 
/// // Contruct any type of collection using from_iter(), collect(), or extend()
/// let deser1: HashMap<Test, Test> = HashMap::from_iter(json_to_iter::<Test,Test>(&ser).unwrap().map(|x| x.unwrap()));
/// assert_eq!(map, deser1);
/// 
/// let deser2: BTreeMap<Test, Test> = json_to_iter::<Test,Test>(&ser).unwrap().map(|x| x.unwrap()).collect();
/// 
/// let mut deser3: Vec<(Test, Test)> = Vec::new();
/// deser3.extend(json_to_iter::<Test,Test>(&ser).unwrap().map(|x| x.unwrap()));
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn json_to_iter<K,V>(str: &str) -> Result<impl Iterator<Item = Result<(K,V), serde_json::Error>>, serde_json::Error> where
for<'de> K: Deserialize<'de> + Any,
for<'de> V: Deserialize<'de>
{
  let json_value = serde_json::from_str(&str)?;
  let json_map = match json_value {
    serde_json::Value::Object(map) => map,
          _ => { return Err(serde_json::Error::custom("Value is not a JSON map")); },
  };
  Ok(JsonToTupleIter {
    iter: json_map.into_iter(),
    kv: std::marker::PhantomData
  })
}

/// Return type of [json_to_iter()](fn.json_to_iter.html). It implements `Iterator<Item = Result<(K,V), serde_json::Error>>`. 
struct JsonToTupleIter<K,V> {
  iter: serde_json::map::IntoIter,
  kv: std::marker::PhantomData<(K,V)>,
}


impl<K,V> Iterator for JsonToTupleIter<K,V> where
for<'de> K: Deserialize<'de> + Any,
for<'de> V: Deserialize<'de>
{
  type Item = Result<(K,V), serde_json::Error>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.iter.next() {
      Some(a) => {
        // handle strings specially as they are not objects
        // compiler seems to be able to optimize this branch away statically
        let key_obj: K = match TypeId::of::<K>() == TypeId::of::<String>() {
          true => match <K as Deserialize>::deserialize(serde_json::Value::from(a.0)) {
            Ok(k) => k,
            Err(e) => { return Some(Err(e)); }
          },
          false => match serde_json::from_str(&a.0) {
            Ok(k) => k,
            Err(e) => { return Some(Err(e)); }
          }
        };
        let val_obj: V = match <V as Deserialize>::deserialize(a.1) {
          Ok(v) => v,
          Err(e) => { return Some(Err(e)); }
        };
        Some(Ok((key_obj, val_obj)))
      },
      None => None
    }
  }
}
