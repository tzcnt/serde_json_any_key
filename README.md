# serde_json_tuple_iter
Serialize any iterator of 2-tuples into a JSON map with serde_json, even if the first element is a struct type.

This allows you to work around the "key must be a string" error when trying to serialize a HashMap with a struct key. The output will be the same as if you manually serialized the struct key to a string.

```
#[derive(Serialize, PartialEq, Eq, Hash)]
pub struct Test {
  pub a: i32,
  pub b: i32
}

fn main() {
  let mut map = HashMap::<Test, Test>::new();
  map.insert(Test {a: 3, b: 5}, Test {a: 7, b: 9});

  let serialized = serde_json::to_string(&map);
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }
  
  let serialized = serde_json_tuple_iter::map_to_json(&mut map.iter());
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }

  let v = vec![(Test {a: 3, b: 5}, Test {a: 7, b: 9})];
  let serialized = serde_json_tuple_iter::vec_to_json(&mut v.iter());
  match serialized {
    Ok(s) => { println!("{}", s); }
    Err(e) => { println!("Error: {}", e); }
  }
}
```

Output:
```
Error: key must be a string
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
{"{\"a\":3,\"b\":5}":{"a":7,"b":9}}
```
