use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
};

use foca::Identity;
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};

use crate::rendezvous::{
    hash::{DefaultNodeHasher, RendezvousNodes},
    node::IdNode,
};

use super::member::Member;

// Our identity is a composite of a socket address and extra
// stuff, but downstream consumers likely only care about
// the address part.
//
// It's perfectly valid to temprarily have more than one member
// pointing at the same address (with a different `bump`): one
// could, for example: join the cluster, ^C the program and
// immediatelly join again. Before Foca detects that the previous
// identity is down we'll receive a notification about this new
// identity going up.
//
// So what we maintain here is a HashMap of addresses to an
// occurence count:
//
//  * The count will most of the time be 1;
//  * But in scenarios like above it may reach 2. Meaning:
//    something made the address change identities, but
//    it's still active
//  * And when the count reaches 0 the address is actually
//    down, so we remove it
//
#[derive(Clone)]
pub struct Members {
    inner: HashMap<SocketAddr, Member>,
    hash: RendezvousNodes<IdNode<SocketAddr>, DefaultNodeHasher>,
}

impl Members {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            hash: RendezvousNodes::default(),
        }
    }

    pub fn init(member_list: Vec<Member>) -> Self {
        let mut members = Members::default();
        for member in member_list {
            members.push(member);
        }
        members
    }

    // If the result is not None, it means that the valid list of cluster members has changed
    pub fn push(&mut self, member: Member) -> Option<Member> {
        match self.inner.get_mut(&member.addr) {
            Some(mut_member) => {
                if mut_member.metadata != member.metadata {
                    mut_member.metadata = member.metadata;
                    mut_member.renew();
                    self.hash.insert(IdNode::new(mut_member.addr));
                    Some(mut_member.clone())
                } else {
                    None
                }
            }
            None => {
                self.inner.insert(member.addr, member.clone());
                self.hash.insert(IdNode::new(member.addr));
                Some(member.clone())
            }
        }
    }

    /// If the result is not None, it means that the valid list of cluster members has changed
    pub fn remove(&mut self, addr: &SocketAddr) -> Option<Member> {
        if self.inner.contains_key(addr) {
            self.hash.remove(addr);
            self.inner.remove(addr)
        } else {
            None
        }
    }

    /// Returns whether or not the member at the specified address exists in the members.
    pub fn contains(&self, addr: &SocketAddr) -> bool {
        self.inner.contains_key(addr)
    }

    /// Returns the member at the specified address.
    pub fn get(&self, addr: &SocketAddr) -> Option<&Member> {
        self.inner.get(addr)
    }

    /// Return the iterator of member.
    pub fn iter(&self) -> impl Iterator<Item = &Member> {
        self.inner.values()
    }

    /// Returns the member responsible for the given key.
    pub fn lookup_member<'a>(&'a self, key: &'a str) -> Option<&Member> {
        self.lookup_members(key, 1).next()
    }

    pub fn lookup_members<'a>(&'a self, key: &'a str, num: usize) -> impl Iterator<Item = &Member> {
        self.hash
            .calc_top_n_candidates(&key, num)
            .map(|addr| self.get(&addr.clone().into_inner()).unwrap())
    }

    pub fn rotate_node(&self, key: &str, num: usize) -> Option<&Member> {
        let addr = if let Some(node) = self.hash.rotate_candidate(&key, num) {
            node.clone().into_inner()
        } else {
            return None;
        };
        self.get(&addr)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Serialize for Members {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Members", 1)?;
        state.serialize_field("members", &self.inner)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Members {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["members"];

        enum Field {
            Members,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`members`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "members" => Ok(Field::Members),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct MembersVisitor;

        impl<'de> Visitor<'de> for MembersVisitor {
            type Value = Members;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Members")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Members, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let members = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Members::init(members))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Members, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut members = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Members => {
                            if members.is_some() {
                                return Err(de::Error::duplicate_field("members"));
                            }
                            members = Some(map.next_value()?);
                        }
                    }
                }
                let members = members.ok_or_else(|| de::Error::missing_field("members"))?;

                Ok(Members::init(members))
            }
        }

        deserializer.deserialize_struct("Members", FIELDS, MembersVisitor)
    }
}

impl Default for Members {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Members {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Members")
            .field("members", &self.inner)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::cluster::members::Members;

    #[test]
    fn test_members_from_slice() {
        let members_json_str = r#"
        {
            "members": [
                {
                    "addr": "0.0.0.0:9901",
                    "metadata": {
                        "grpc_address": "0.0.0.0:9911",
                        "http_address": "0.0.0.0:9921"
                    },
                    "version": 1
                },
                {
                    "addr": "0.0.0.0:9902",
                    "metadata": {
                        "grpc_address": "0.0.0.0:9912",
                        "http_address": "0.0.0.0:9922"
                    },
                    "version": 1
                }
            ]
        }
        "#;
        let members_json_bytes = members_json_str.as_bytes();

        let members = serde_json::from_slice::<Members>(members_json_bytes).unwrap();
        assert_eq!(members.len(), 2);
    }
}
