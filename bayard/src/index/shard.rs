use time::OffsetDateTime;

fn default_state() -> State {
    State::Unknown
}

fn default_version() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum State {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "serving")]
    Serving,
    #[serde(rename = "draining")]
    Draining,
    #[serde(rename = "drained")]
    Drained,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shard {
    pub id: String,
    #[serde(default = "default_state")]
    pub state: State,
    #[serde(default = "default_version")]
    version: i64,
}

impl Shard {
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: State::Serving,
            version: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::index::shard::{Shard, State};

    #[test]
    fn test_shard_from_slice() {
        let shard_json_str = r#"
        {
            "id": "shard_1"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Unknown);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "unknown"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Unknown);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "serving"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Serving);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "draining"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Draining);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "drained"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Drained);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "version": 1
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Unknown);
        assert_eq!(shard.version, 1);

        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "serving",
            "version": 1
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Serving);
        assert_eq!(shard.version, 1);
    }

    #[test]
    fn test_shard_to_vec() {
        let shard_json_str = r#"
        {
            "id": "shard_1",
            "state": "serving"
        }
        "#;
        let shard_json_bytes = shard_json_str.as_bytes();
        let shard = serde_json::from_slice::<Shard>(shard_json_bytes).unwrap();
        assert_eq!(shard.id, "shard_1");
        assert_eq!(shard.state, State::Serving);

        let _shard_vec = serde_json::to_vec(&shard).unwrap();
    }
}
