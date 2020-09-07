use serde::de::IntoDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

pub(crate) fn default_for_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParseError {
    _priv: (),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided string was not `true` or `false`".fmt(f)
    }
}

pub(crate) fn tagged_or_untagged<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default, /* + FromStr<Err = ParseError> */
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + Default, /* + FromStr<Err = ParseError> */
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or `map {\"type\": string }`")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            T::deserialize(value.to_lowercase().into_deserializer())
        }
        fn visit_map<M>(self, mut map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                if key == "type" {
                    return self.visit_str(&value);
                }
            }
            Err(de::Error::missing_field("type"))
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(T::default())
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
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

    #[derive(Eq, PartialEq, Deserialize, Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    enum Tagged {
        One,
        Two,
    }
    impl Default for Tagged {
        fn default() -> Self {
            Tagged::One
        }
    }
    #[derive(Deserialize, Serialize, Debug)]
    struct Wrap {
        #[serde(deserialize_with = "tagged_or_untagged")]
        field: Tagged,
    }
    #[test]
    fn test_taggedd() {
        let str = r#"
            {
                "field": "One"
            }
       "#;

        let test_struct = serde_json::from_str::<Wrap>(str);
        assert_eq!(test_struct.unwrap().field, Tagged::One)
    }
    #[test]
    fn test_untagged() {
        let str = r#"
            {
                "field": {"type": "Two"}
            }
       "#;

        let test_struct = serde_json::from_str::<Wrap>(str);
        assert_eq!(test_struct.unwrap().field, Tagged::Two)
    }
}
