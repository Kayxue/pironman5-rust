use serde_json::Value;
use std::collections::HashSet;

/// Merge two JSON objects recursively
pub fn merge_dict(dict1: &Value, dict2: &Value) -> Value {
    match (dict1, dict2) {
        (Value::Object(map1), Value::Object(map2)) => {
            let mut result = map1.clone();
            for (key, value) in map2.iter() {
                result.insert(
                    key.clone(),
                    if let Some(existing) = result.get(key) {
                        merge_dict(existing, value)
                    } else {
                        value.clone()
                    },
                );
            }
            Value::Object(result)
        }
        (Value::Array(arr1), Value::Array(arr2)) => {
            let mut result = arr1.clone();
            result.extend(arr2.clone());
            Value::Array(result)
        }
        _ => dict2.clone(),
    }
}

/// Check if two lists have common items
#[allow(dead_code)]
pub fn has_common_items(list1: &[String], list2: &[String]) -> bool {
    let set1: HashSet<_> = list1.iter().collect();
    let set2: HashSet<_> = list2.iter().collect();
    !set1.is_disjoint(&set2)
}

/// Check if target is included in the list
#[allow(dead_code)]
pub fn is_included(list: &[String], target: &str) -> bool {
    list.iter().any(|item| item == target)
}

/// Check if any target in the list is included
#[allow(dead_code)]
pub fn is_any_included(list: &[String], targets: &[String]) -> bool {
    has_common_items(list, targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_dict() {
        let dict1 = json!({"a": 1, "b": {"c": 2}});
        let dict2 = json!({"b": {"d": 3}, "e": 4});
        let result = merge_dict(&dict1, &dict2);
        assert_eq!(result, json!({"a": 1, "b": {"c": 2, "d": 3}, "e": 4}));
    }

    #[test]
    fn test_is_included() {
        let list = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(is_included(&list, "b"));
        assert!(!is_included(&list, "d"));
    }
}

