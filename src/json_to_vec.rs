use super::*;

/// Reverses to_json_map(), returning a `Vec<(K,V)>`.
///
/// # Examples
/// ```
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
/// let vec: Vec<(Test,Test)> = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
/// 
/// let ser = vec.to_json_map().unwrap();
/// let deser: Vec<(Test,Test)> = json_to_vec(&ser).unwrap();
///
/// assert_eq!(vec, deser);
/// Ok(()) }
/// try_main().unwrap();
/// ```
pub fn json_to_vec<'a,K,V>(str: &'a str) -> Result<Vec<(K,V)>, serde_json::Error> where
for<'de> K: Deserialize<'de> + Any,
for<'de> V: Deserialize<'de>
{
  let mut vec: Vec<(K,V)> = vec![];
  let v: serde_json::Value = serde_json::from_str(&str)?;
  let o = v.as_object().ok_or(serde_json::Error::custom("Value is not a JSON map"))?;
  // handle strings specially as they are not objects
  // compiler seems to be able to optimize this branch away statically
  if TypeId::of::<K>() == TypeId::of::<String>() {
    for (key, val) in o.iter() {
      let key_obj: K = <K as Deserialize>::deserialize(serde_json::Value::from(key.as_str()))?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      vec.push((key_obj, val_obj));
    }
  } else {
    for (key, val) in o.iter() {
      let key_obj: K = serde_json::from_str(key)?;
      let val_obj: V = <V as Deserialize>::deserialize(val)?;
      vec.push((key_obj, val_obj));
    }
  }
  Ok(vec)
}
