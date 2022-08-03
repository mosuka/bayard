use time::OffsetDateTime;

fn default_version() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shard {
    pub id: String,
    #[serde(default = "default_version")]
    version: i64,
}

impl Shard {
    pub fn new(id: String) -> Self {
        Self {
            id,
            version: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::index::shard::Shard;

    #[test]
    fn test_shard_from_slice() {
        let shard_json_str = r#"
        {
            "id": "aaaaaaaa"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();

        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "aaaaaaaa");

        let shard_json_str = r#"
        {
            "id": "aaaaaaaa",
            "version": 1
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();

        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "aaaaaaaa");
        assert_eq!(shard.version, 1);
    }
}
