//! JSON pointer set/delete used to fold incremental board patches into the
//! cached snapshot new browsers receive on connect.

use serde_json::{Map, Value};

fn decode_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let b = raw.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'~' && i + 1 < b.len() {
            match b[i + 1] {
                b'0' => out.push('~'),
                b'1' => out.push('/'),
                other => {
                    out.push('~');
                    out.push(other as char);
                }
            }
            i += 2;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

fn tokens(path: &str) -> Vec<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return Vec::new();
    }
    trimmed
        .trim_start_matches('/')
        .split('/')
        .map(decode_token)
        .collect()
}

fn key_index(key: &str) -> Option<usize> {
    if key.is_empty() || (key.len() > 1 && key.starts_with('0')) {
        return None;
    }
    key.parse::<usize>().ok()
}

/// Set `path` (JSON pointer) on `root`, creating objects along the way.
pub fn set_pointer(root: &mut Value, path: &str, value: Value) -> bool {
    let parts = tokens(path);
    if parts.is_empty() {
        *root = value;
        return true;
    }
    let mut cur = root;
    for (i, part) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        if last {
            return write_slot(cur, part, value);
        }
        cur = descend_mut(cur, part);
    }
    false
}

fn write_slot(cur: &mut Value, key: &str, value: Value) -> bool {
    if let Some(idx) = key_index(key) {
        if !cur.is_array() {
            *cur = Value::Array(Vec::new());
        }
        let arr = cur.as_array_mut().expect("array");
        if idx > arr.len() {
            return false;
        }
        if idx == arr.len() {
            arr.push(value);
        } else {
            arr[idx] = value;
        }
        return true;
    }
    if !cur.is_object() {
        *cur = Value::Object(Map::new());
    }
    cur.as_object_mut().expect("object").insert(key.to_string(), value);
    true
}

fn descend_mut<'a>(cur: &'a mut Value, key: &str) -> &'a mut Value {
    if let Some(idx) = key_index(key) {
        if !cur.is_array() {
            *cur = Value::Array(Vec::new());
        }
        let arr = cur.as_array_mut().expect("array");
        while arr.len() <= idx {
            arr.push(Value::Object(Map::new()));
        }
        return &mut arr[idx];
    }
    if !cur.is_object() {
        *cur = Value::Object(Map::new());
    }
    let obj = cur.as_object_mut().expect("object");
    if !obj.contains_key(key) {
        obj.insert(key.to_string(), Value::Object(Map::new()));
    }
    obj.get_mut(key).expect("just inserted")
}

pub fn delete_pointer(root: &mut Value, path: &str) -> bool {
    let parts = tokens(path);
    if parts.is_empty() {
        return false;
    }
    let mut cur = root;
    for part in &parts[..parts.len() - 1] {
        cur = match step_mut(cur, part) {
            Some(next) => next,
            None => return false,
        };
    }
    let leaf = parts.last().expect("non-empty");
    if let Some(idx) = key_index(leaf) {
        if let Some(arr) = cur.as_array_mut() {
            if idx < arr.len() {
                arr.remove(idx);
                return true;
            }
        }
        return false;
    }
    if let Some(obj) = cur.as_object_mut() {
        return obj.remove(leaf).is_some();
    }
    false
}

fn step_mut<'a>(cur: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    if let Some(idx) = key_index(key) {
        return cur.as_array_mut().and_then(|arr| arr.get_mut(idx));
    }
    cur.as_object_mut().and_then(|obj| obj.get_mut(key))
}

/// Apply `{op, path, value?}` ops. Unknown ops are skipped.
pub fn apply_ops(root: &mut Value, ops: &[Value]) {
    for op in ops {
        let kind = op.get("op").and_then(|v| v.as_str()).unwrap_or("");
        let path = op.get("path").and_then(|v| v.as_str()).unwrap_or("");
        match kind {
            "set" | "replace" | "add" => {
                let value = op.get("value").cloned().unwrap_or(Value::Null);
                let _ = set_pointer(root, path, value);
            }
            "del" | "remove" => {
                let _ = delete_pointer(root, path);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_nested_and_array_index() {
        let mut root = json!({});
        assert!(set_pointer(&mut root, "/config/files", json!([{"id": "a", "active": false}])));
        assert!(set_pointer(&mut root, "/config/files/0/active", json!(true)));
        assert_eq!(root["config"]["files"][0]["active"], json!(true));
    }

    #[test]
    fn delete_object_key() {
        let mut root = json!({"settings": {"a": 1, "b": 2}});
        assert!(delete_pointer(&mut root, "/settings/a"));
        assert!(root["settings"].get("a").is_none());
        assert_eq!(root["settings"]["b"], json!(2));
    }

    #[test]
    fn pointer_unescapes_slash() {
        let mut root = json!({});
        assert!(set_pointer(&mut root, "/config/tab~1name", json!("x")));
        assert_eq!(root["config"]["tab/name"], json!("x"));
    }
}
