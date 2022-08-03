use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};

use crate::rendezvous::{
    hash::{DefaultNodeHasher, RendezvousNodes},
    node::IdNode,
};

use super::shard::Shard;

#[derive(Clone)]
pub struct Shards {
    keys: Vec<String>,
    inner: HashMap<String, Shard>,
    hash: RendezvousNodes<IdNode<String>, DefaultNodeHasher>,
}

impl Shards {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            inner: HashMap::new(),
            hash: RendezvousNodes::default(),
        }
    }

    pub fn init(shards: Vec<Shard>) -> Self {
        let mut s = Shards::new();
        for shard in shards {
            s.insert(shard);
        }
        s
    }

    pub fn insert(&mut self, shard: Shard) -> bool {
        let mut changed = false;

        // Push shard id.
        changed |= if self.keys.contains(&shard.id) {
            false
        } else {
            self.keys.push(shard.id.clone());
            true
        };

        // Insert shard.
        changed |= if self.inner.contains_key(&shard.id) {
            // Shard exists.
            // Nothing changed.
            false
        } else {
            // Shard does not exist.
            // Add new shard.
            self.inner.insert(shard.clone().id, shard.clone());
            true
        };

        // Insert shard to Rendezvous hash.
        changed |= if self.hash.contains(&shard.id) {
            // Node exists in hash.
            false
        } else {
            // Node does not exist in hash.
            self.hash.insert(IdNode::new(shard.id));
            true
        };

        changed
    }

    pub fn remove(&mut self, shard: &Shard) -> bool {
        let mut changed = false;

        // Remove shard id.
        changed |= if self.keys.contains(&shard.id) {
            // Remove shard id.
            self.keys.retain(|id| id != &shard.id);
            true
        } else {
            false
        };

        // Remove shard.
        changed |= matches!(self.inner.remove(&shard.id), Some(_shard));

        // Remove shard from Rendezvous hash.
        changed |= self.hash.remove(&shard.id).is_some();

        changed
    }

    pub fn pop(&mut self) -> bool {
        let shard_id = match self.keys.pop() {
            Some(key) => key,
            None => return false,
        };

        self.remove(&Shard::new(shard_id))
    }

    /// Returns whether or not the shard at the specified ID exists in the shards.
    pub fn contains(&self, id: &String) -> bool {
        self.inner.contains_key(id)
    }

    /// Returns the shard at the specified ID.
    pub fn get(&self, id: &String) -> Option<&Shard> {
        self.inner.get(id)
    }

    /// Return the iterator of shard.
    pub fn iter(&self) -> impl Iterator<Item = &Shard> {
        self.inner.values()
    }

    /// Returns the shard responsible for the given key.
    pub fn lookup_shard<'a>(&'a self, key: &'a str) -> Option<&Shard> {
        self.lookup_shards(key, 1).next()
    }

    pub fn lookup_shards<'a>(&'a self, key: &'a str, num: usize) -> impl Iterator<Item = &Shard> {
        self.hash
            .calc_top_n_candidates(&key, num)
            .map(|id| self.get(&id.clone().into_inner()).unwrap())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for Shards {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Shards {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shards")
            .field("shard_list", &self.inner)
            .finish()
    }
}

impl Serialize for Shards {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Shards", 1)?;
        state.serialize_field("shard_list", &self.inner.values().collect::<Vec<&Shard>>())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Shards {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["shard_list"];

        enum Field {
            ShardList,
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
                        formatter.write_str("`shard_list`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "shard_list" => Ok(Field::ShardList),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ShardsVisitor;

        impl<'de> Visitor<'de> for ShardsVisitor {
            type Value = Shards;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Shards")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Shards, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let shard_list = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Shards::init(shard_list))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Shards, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut shard_list = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ShardList => {
                            if shard_list.is_some() {
                                return Err(de::Error::duplicate_field("shard_list"));
                            }
                            shard_list = Some(map.next_value()?);
                        }
                    }
                }
                let shard_list =
                    shard_list.ok_or_else(|| de::Error::missing_field("shard_list"))?;

                Ok(Shards::init(shard_list))
            }
        }

        deserializer.deserialize_struct("Shards", FIELDS, ShardsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{shard::Shard, shards::Shards};

    #[test]
    fn test_shards_from_slice() {
        let shards_json_str = r#"
        {
            "shard_list": [
                {
                    "id": "aaaaaaaa"
                },
                {
                    "id": "bbbbbbb",
                    "version": 1
                }
            ]
        }
        "#;
        let shards_json_bytes = shards_json_str.as_bytes();

        let shards = serde_json::from_slice::<Shards>(shards_json_bytes).unwrap();
        assert_eq!(shards.len(), 2);
    }

    #[test]
    fn test_insert() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        assert!(shards.insert(shard));

        let shard2 = Shard::new("bar".to_string());
        assert!(shards.insert(shard2));

        let shard3 = Shard::new("bar".to_string());
        assert!(!shards.insert(shard3));
    }

    #[test]
    fn test_remove() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.insert(shard);

        let shard2 = Shard::new("bar".to_string());
        shards.insert(shard2.clone());

        assert!(shards.remove(&shard2));

        assert!(!shards.remove(&shard2));
    }

    #[test]
    fn test_contains() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.insert(shard);

        assert!(shards.contains(&"foo".to_string()));

        assert!(!shards.contains(&"bar".to_string()));
    }

    #[test]
    fn test_iter() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.insert(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.insert(shard2);

        let shards_iter = shards.iter();
        let shard_ids = shards_iter
            .map(|shard| shard.id.clone())
            .collect::<Vec<_>>();
        assert!(shard_ids.contains(&"foo".to_string()));
        assert!(shard_ids.contains(&"bar".to_string()));
        assert!(!shard_ids.contains(&"baz".to_string()));
    }

    #[test]
    fn test_get() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.insert(shard);

        let shard = shards.get(&"foo".to_string());
        assert_eq!(shard.unwrap().id, "foo".to_string());

        let shard = shards.get(&"bar".to_string());
        assert!(shard.is_none());
    }

    #[test]
    fn test_len() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.insert(shard);
        assert_eq!(shards.len(), 1);

        let shard2 = Shard::new("bar".to_string());
        shards.insert(shard2.clone());
        assert_eq!(shards.len(), 2);

        let shard3 = Shard::new("bar".to_string());
        shards.insert(shard3);
        assert_eq!(shards.len(), 2);

        shards.remove(&shard2);
        assert_eq!(shards.len(), 1);

        shards.remove(&shard2);
        assert_eq!(shards.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut shards = Shards::new();
        assert!(shards.is_empty());

        let shard1 = Shard::new("foo".to_string());
        shards.insert(shard1.clone());
        assert!(!shards.is_empty());

        shards.remove(&shard1);
        assert!(shards.is_empty());
    }

    #[test]
    fn test_lookup_shard() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.insert(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.insert(shard2);

        let shard3 = Shard::new("baz".to_string());
        shards.insert(shard3);

        let shard = shards.lookup_shard("hoge");
        assert_eq!(shard.unwrap().id, "foo".to_string());

        let shard = shards.lookup_shard("fuga");
        assert_eq!(shard.unwrap().id, "baz".to_string());

        let shard = shards.lookup_shard("piyo");
        assert_eq!(shard.unwrap().id, "bar".to_string());
    }

    #[test]
    fn test_lookup_shards() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.insert(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.insert(shard2);

        let shard3 = Shard::new("baz".to_string());
        shards.insert(shard3);

        let mut shards_iter = shards.lookup_shards("hoge", 2);
        assert_eq!(shards_iter.next().unwrap().id, "foo".to_string());
        assert_eq!(shards_iter.next().unwrap().id, "baz".to_string());
        assert!(shards_iter.next().is_none());

        let mut shards_iter = shards.lookup_shards("fuga", 2);
        assert_eq!(shards_iter.next().unwrap().id, "baz".to_string());
        assert_eq!(shards_iter.next().unwrap().id, "foo".to_string());
        assert!(shards_iter.next().is_none());

        let mut shards_iter = shards.lookup_shards("piyo", 2);
        assert_eq!(shards_iter.next().unwrap().id, "bar".to_string());
        assert_eq!(shards_iter.next().unwrap().id, "foo".to_string());
        assert!(shards_iter.next().is_none());
    }
}
