use serde::{Deserialize, Deserializer};

pub(crate) fn default_for_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[cfg(test)]
mod test {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    struct TestStruct {
        #[serde(default, deserialize_with = "default_for_null")]
        field: Vec<String>,
    }

    #[test]
    fn test_default_for_null() {
        let str = r#"
            {
                "field": null
            }
       "#;
        let test_struct = serde_json::from_str::<TestStruct>(str);
        assert!(test_struct.unwrap().field.is_empty())
    }
}
