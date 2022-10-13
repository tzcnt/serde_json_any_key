
use std::any::{Any, TypeId};
use std::hash::Hash;
use serde::ser::Error;
use serde::de::Deserialize;

/// Reverses to_json_map(), returning a `HashMap<K,V>`.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use serde::{Serialize, Deserialize};
/// use serde_json::Error;
/// use serde_json_any_key::*;
/// 
/// #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
/// pub struct Test {
///   pub a: i32,
///   pub b: i32
/// }
/// 
/// fn try_main() -> Result<(), Error> {
/// let mut map = HashMap::<Test, Test>::new();
/// map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
/// 
/// let ser = map.to_json_map().unwrap();
/// let deser: HashMap<Test, Test> = json_to_map(&ser).unwrap();
///
/// assert_eq!(map, deser);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn json_to_map<'a,K,V>(str: &'a str) -> Result<std::collections::HashMap<K,V>, serde_json::Error> where
for<'de> K: Deserialize<'de> + std::cmp::Eq + Hash + Any,
for<'de> V: Deserialize<'de>
{
  let mut map: std::collections::HashMap<K,V> = std::collections::HashMap::new();
  let v: serde_json::Value = serde_json::from_str(&str)?;
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a JSON map"))?;
  // handle strings specially as they are not objects
  // compiler seems to be able to optimize this branch away statically
  if TypeId::of::<K>() == TypeId::of::<String>() {
    for (key, val) in o.iter() {
      let key_obj: K = <K as Deserialize>::deserialize(serde_json::Value::from(key.as_str()))?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      map.insert(key_obj, val_obj);
    }
  } else {
    for (key, val) in o.iter() {
      let key_obj: K = serde_json::from_str(key)?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      map.insert(key_obj, val_obj);
    }
  }
  Ok(map)
}
