use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum EncodedPayload {
    Base64(String),
    Legacy(Vec<u8>),
}

pub(super) fn serialize<S>(payload: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    STANDARD.encode(payload).serialize(serializer)
}

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    match EncodedPayload::deserialize(deserializer)? {
        EncodedPayload::Base64(value) => STANDARD.decode(value).map_err(serde::de::Error::custom),
        EncodedPayload::Legacy(value) => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_base64_and_reads_legacy_arrays() {
        assert_eq!(
            serde_json::to_string(&Bytes(vec![0, 255])).unwrap(),
            "\"AP8=\""
        );
        assert_eq!(
            serde_json::from_str::<Bytes>("[0,255]").unwrap().0,
            vec![0, 255]
        );
    }

    struct Bytes(Vec<u8>);

    impl Serialize for Bytes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serialize(&self.0, serializer)
        }
    }

    impl<'de> Deserialize<'de> for Bytes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize(deserializer).map(Self)
        }
    }
}
