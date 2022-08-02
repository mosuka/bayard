use std::net::SocketAddr;

use foca::Identity;
use time::OffsetDateTime;
use tracing::info;

use super::metadata::Metadata;

fn default_version() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Member {
    pub addr: SocketAddr,
    pub metadata: Option<Metadata>,
    #[serde(default = "default_version")]
    pub version: i64,
}

impl Member {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            metadata: None,
            version: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn new_with_metadata(addr: SocketAddr, metadata: Metadata) -> Self {
        Self {
            addr,
            metadata: Some(metadata),
            version: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

impl Identity for Member {
    // Since a client outside the cluster will not be aware of our
    // `version` field, we implement the optional trait method
    // `has_same_prefix` to allow anyone that knows our `addr`
    // to join our cluster.
    fn has_same_prefix(&self, other: &Self) -> bool {
        self.addr.eq(&other.addr)
    }

    // And by implementing `renew` we enable automatic rejoining:
    // when another member declares us as down, Foca immediatelly
    // switches to this new identity and rejoins the cluster for us
    fn renew(&self) -> Option<Self> {
        let new_member = Self {
            addr: self.addr,
            metadata: self.metadata,
            version: OffsetDateTime::now_utc().unix_timestamp(),
        };
        info!(?new_member, "Renew.");
        Some(new_member)
    }
}

#[cfg(test)]
mod tests {
    use crate::cluster::member::Member;

    #[test]
    fn test_member_from_slice() {
        let member_json_str = r#"
        {
            "addr": "0.0.0.0:9901",
            "metadata": {
                "grpc_address": "0.0.0.0:9911",
                "http_address": "0.0.0.0:9921"
            },
            "version": 1
        }
        "#;
        let member_json_bytes = member_json_str.as_bytes();

        let member = serde_json::from_slice::<Member>(member_json_bytes).unwrap();
        assert_eq!(member.addr, "0.0.0.0:9901".parse().unwrap());
        assert_eq!(
            member.metadata.unwrap().grpc_address,
            Some("0.0.0.0:9911".parse().unwrap())
        );
        assert_eq!(
            member.metadata.unwrap().http_address,
            Some("0.0.0.0:9921".parse().unwrap())
        );
        assert_eq!(member.version, 1);

        let member_json_str = r#"
        {
            "addr": "0.0.0.0:9901",
            "version": 1
        }
        "#;
        let member_json_bytes = member_json_str.as_bytes();

        let member = serde_json::from_slice::<Member>(member_json_bytes).unwrap();
        assert_eq!(member.addr, "0.0.0.0:9901".parse().unwrap());
        assert_eq!(member.metadata, None);
        assert_eq!(member.version, 1);

        let member_json_str = r#"
        {
            "addr": "0.0.0.0:9901"
        }
        "#;
        let member_json_bytes = member_json_str.as_bytes();

        let member = serde_json::from_slice::<Member>(member_json_bytes).unwrap();
        assert_eq!(member.addr, "0.0.0.0:9901".parse().unwrap());
        assert_eq!(member.metadata, None);
    }
}
