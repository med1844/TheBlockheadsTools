// https://github.com/ebarnard/rust-plist/pull/55#issuecomment-771113306

use serde::{Deserialize, Serialize, de, ser};

pub fn deserialize_some<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(de).map(Some)
}

pub fn serialize_some<S, T>(value: &Option<T>, ser: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: Serialize,
{
    value
        .as_ref()
        .expect(r#"`serialize_some` must be used with `skip_serializing_if = "Option::is_none"`"#)
        .serialize(ser)
}
