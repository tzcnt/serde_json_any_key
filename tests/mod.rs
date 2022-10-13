
#[cfg(test)]
mod tests {
  use super::*;
  use serde_json_any_key::*;
  use std::collections::HashMap;
  use serde::{Serialize, Deserialize};

  #[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
  struct Test {
    pub a: i32,
    pub b: i32
  }

  #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
  struct TestWithString {
    pub a: i32,
    pub b: i32,
    pub c: String
  }

  #[test]
  fn test_struct_attr_top_level_map() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map")]
      pub inner: HashMap<TestWithString,TestWithString>
    }
    
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(TestWithString,TestWithString)>
    }

    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct Outer {
      pub map: SerdeWithMap,
      pub vec: SerdeWithVec
    }

    let mut map = SerdeWithMap { inner: HashMap::new() };
    map.inner.insert(TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()});

    let mut vec = SerdeWithVec { inner: vec![] };
    vec.inner.push((TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()}));

    let outer = Outer {
      map: map,
      vec: vec
    };
    {
      let mut top_level_string_map = HashMap::<String, Outer>::new();
      top_level_string_map.insert("top".to_string(), outer.clone());

      let ser1 = serde_json::to_string(&top_level_string_map).unwrap();
      let ser2 = top_level_string_map.to_json_map().unwrap();
      assert_eq!(ser1, ser2);
      let deser1: HashMap<String, Outer> = serde_json::from_str(&ser1).unwrap();
      let deser2: HashMap<String, Outer> = json_to_map(&ser1).unwrap();
      assert_eq!(top_level_string_map, deser1);
      assert_eq!(top_level_string_map, deser2);
    }
    {
      let mut top_level_struct_map = HashMap::<TestWithString, Outer>::new();
      top_level_struct_map.insert(TestWithString { a: 10, b: 11, c: "bbq".to_string() }, outer.clone());

      let ser = top_level_struct_map.to_json_map().unwrap();
      let deser: HashMap<TestWithString, Outer> = json_to_map(&ser).unwrap();
      assert_eq!(top_level_struct_map, deser);
    }
  }
  
  #[test]
  fn test_struct_attr_1_level() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map")]
      pub inner: HashMap<TestWithString,TestWithString>
    }
    
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(TestWithString,TestWithString)>
    }

    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct Outer {
      pub map: SerdeWithMap,
      pub vec: SerdeWithVec
    }

    let mut map = SerdeWithMap { inner: HashMap::new() };
    map.inner.insert(TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()});

    let mut vec = SerdeWithVec { inner: vec![] };
    vec.inner.push((TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()}));

    let outer = Outer {
      map: map,
      vec: vec
    };

    let serialized = serde_json::to_string(&outer).unwrap();
    let deser: Outer = serde_json::from_str(&serialized).unwrap();
    assert_eq!(outer, deser);
  }

  #[test]
  fn test_struct_attr_2_level() {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct SerdeWithMap {
      #[serde(with = "any_key_map")]
      pub inner: HashMap<TestWithString,TestWithString>
    }
    
    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug, Hash)]
    struct SerdeWithVec {
      #[serde(with = "any_key_vec")]
      pub inner: Vec<(TestWithString,TestWithString)>
    }

    #[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct Outer {
      #[serde(with = "any_key_map")]
      pub map: HashMap<SerdeWithVec, SerdeWithMap>,
      #[serde(with = "any_key_vec")]
      pub vec: Vec<(SerdeWithMap, SerdeWithVec)>
    }

    let mut map = SerdeWithMap { inner: HashMap::new() };
    map.inner.insert(TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()});

    let mut vec = SerdeWithVec { inner: vec![] };
    vec.inner.push((TestWithString {a: 3, b: 5, c: "foo".to_string()},
      TestWithString {a: 7, b: 9, c: "bar".to_string()}));

    let mut outer_map = HashMap::<SerdeWithVec, SerdeWithMap>::new();
    outer_map.insert(vec.clone(), map.clone());
    let mut outer_vec = Vec::<(SerdeWithMap, SerdeWithVec)>::new();
    outer_vec.push((map, vec));
    let outer = Outer {
      map: outer_map,
      vec: outer_vec
    };

    let serialized = serde_json::to_string(&outer).unwrap();
    let deser: Outer = serde_json::from_str(&serialized).unwrap();
    assert_eq!(outer, deser);
  }

  #[test]
  fn test_struct_roundtrip_map() {
    let mut data = HashMap::<Test, Test>::new();
    data.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    data.insert(Test {a: 11, b: 12}, Test {a: 13, b: 14});
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_map_iter() {
    let mut data = HashMap::<Test, Test>::new();
    data.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    data.insert(Test {a: 11, b: 12}, Test {a: 13, b: 14});
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<Test, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(Test, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_roundtrip_vec_iter() {
    let mut data = Vec::<(Test, Test)>::new();
    data.push((Test {a: 3, b: 5}, Test {a: 7, b: 9}));
    data.push((Test {a: 11, b: 12}, Test {a: 13, b: 14}));
    let serialized = data.iter().to_json_map().unwrap();
    let mut deser: Vec<(Test, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_struct_canonical_serialization() {
    let mut map = HashMap::<Test, Test>::new();
    map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let string_map: HashMap<String, Test> = map.iter().map(|(k, &v)| (serde_json::to_string(k).unwrap(), v)).collect();
    let canonical_serialization = serde_json::to_string(&string_map).unwrap();
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<Test, Test>::new();
    btree.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }

  #[test]
  fn test_string_roundtrip_map() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_map_iter() {
    let mut data = HashMap::<String, i32>::new();
    data.insert("bar".to_string(), 7);
    data.insert("foo".to_string(), 5);
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<String, i32> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_roundtrip_vec_iter() {
    let mut data = Vec::<(String, i32)>::new();
    data.push(("bar".to_string(), 7));
    data.push(("foo".to_string(), 5));
    let serialized = data.iter().to_json_map().unwrap();
    let mut deser: Vec<(String, i32)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_string_canonical_serialization() {
    let mut map = HashMap::<String, i32>::new();
    map.insert("foo".to_string(), 5);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![("foo".to_string(), 5)];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<String, i32>::new();
    btree.insert("foo".to_string(), 5);
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }


  #[test]
  fn test_int_roundtrip_map() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = data.to_json_map().unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_map_iter() {
    let mut data = HashMap::<i32, Test>::new();
    data.insert(5, Test {a: 6, b: 7});
    data.insert(6, Test {a: 9, b: 11});
    let serialized = data.iter().to_json_map().unwrap();
    let deser: HashMap<i32, Test> = json_to_map(&serialized).unwrap();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = data.to_json_map().unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_roundtrip_vec_iter() {
    let mut data = Vec::<(i32, Test)>::new();
    data.push((5, Test {a: 6, b: 7}));
    data.push((6, Test {a: 9, b: 11}));
    let serialized = data.iter().to_json_map().unwrap();
    let mut deser: Vec<(i32, Test)> = json_to_vec(&serialized).unwrap();
    deser.sort();

    assert_eq!(data, deser);
  }

  #[test]
  fn test_int_canonical_serialization() {
    let mut map = HashMap::<i32, f32>::new();
    map.insert(5, 7.0f32);
    let canonical_serialization = serde_json::to_string(&map).unwrap();
    
    let serialized = map.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let vec = vec![(5, 7.0f32)];
    let serialized = vec.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);

    let mut btree = std::collections::BTreeMap::<i32, f32>::new();
    btree.insert(5, 7.0f32);
    let serialized = btree.to_json_map().unwrap();
    assert_eq!(serialized, canonical_serialization);
  }
}
