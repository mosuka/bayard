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

use super::shard::{Shard, State};

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

    pub fn init(shard_list: Vec<Shard>) -> Self {
        let mut shards = Shards::new();
        for shard in shard_list {
            shards.push(shard);
        }
        shards
    }

    pub fn remove(&mut self, id: &String) -> Option<Shard> {
        if self.inner.contains_key(id) {
            self.hash.remove(id);
            self.keys.retain(|shard_id| shard_id != id);
            self.inner.remove(id)
        } else {
            None
        }
    }

    pub fn push(&mut self, shard: Shard) -> Option<Shard> {
        if self.inner.contains_key(&shard.id) {
            None
        } else {
            self.inner.insert(shard.id.clone(), shard.clone());
            self.keys.push(shard.id.clone());
            self.hash.insert(IdNode::new(shard.clone().id));
            Some(shard)
        }
    }

    pub fn pop(&mut self) -> Option<Shard> {
        let shard_id = match self.keys.last() {
            Some(shard_id) => shard_id.clone(),
            None => return None,
        };

        self.remove(&shard_id)
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

    fn iter_filterd_shards<'a>(&'a self, state: State) -> impl Iterator<Item = &Shard> {
        self.iter().filter(move |shard| shard.state == state)
    }

    pub fn iter_serving_shards<'a>(&'a self) -> impl Iterator<Item = &Shard> {
        self.iter_filterd_shards(State::Serving)
    }

    pub fn iter_draining_shards<'a>(&'a self) -> impl Iterator<Item = &Shard> {
        self.iter_filterd_shards(State::Draining)
    }

    pub fn iter_drained_shards<'a>(&'a self) -> impl Iterator<Item = &Shard> {
        self.iter_filterd_shards(State::Drained)
    }

    pub fn lookup_shards<'a>(&'a self, key: &'a str, num: usize) -> impl Iterator<Item = &Shard> {
        self.hash
            .calc_top_n_candidates(&key, num)
            .filter_map(|id| self.get(&id.clone().into_inner()))
    }

    pub fn lookup_shard<'a>(&'a self, key: &'a str) -> Option<&Shard> {
        self.lookup_shards(key, 1).next()
    }

    fn lookup_filterd_shards<'a>(
        &'a self,
        key: &'a str,
        num: usize,
        state: State,
    ) -> impl Iterator<Item = &Shard> {
        self.lookup_shards(key, num)
            .filter(move |shard| shard.state == state)
    }

    pub fn lookup_serving_shards<'a>(
        &'a self,
        key: &'a str,
        num: usize,
    ) -> impl Iterator<Item = &Shard> {
        self.lookup_filterd_shards(key, num, State::Serving)
    }

    pub fn lookup_serving_shard<'a>(&'a self, key: &'a str) -> Option<&Shard> {
        self.lookup_serving_shards(key, 1).next()
    }

    pub fn lookup_draining_shards<'a>(
        &'a self,
        key: &'a str,
        num: usize,
    ) -> impl Iterator<Item = &Shard> {
        self.lookup_filterd_shards(key, num, State::Draining)
    }

    pub fn lookup_drained_shards<'a>(
        &'a self,
        key: &'a str,
        num: usize,
    ) -> impl Iterator<Item = &Shard> {
        self.lookup_filterd_shards(key, num, State::Drained)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn serving_shards_len(&self) -> usize {
        self.iter_serving_shards().count()
    }

    pub fn draining_shards_len(&self) -> usize {
        self.iter_draining_shards().count()
    }

    pub fn drained_shards_len(&self) -> usize {
        self.iter_drained_shards().count()
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
    use crate::index::{
        shard::{Shard, State},
        shards::Shards,
    };

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
    fn test_shards_to_vec() {
        let shards_json_str = r#"
        {
            "shard_list": [
                {
                    "id": "aaaaaaaa",
                    "state": "serving",
                    "version": 1
                },
                {
                    "id": "bbbbbbb",
                    "state": "draining",
                    "version": 1
                }
            ]
        }
        "#;
        let shards_json_bytes = shards_json_str.as_bytes();

        let shards = serde_json::from_slice::<Shards>(shards_json_bytes).unwrap();

        let _shards_vec = serde_json::to_vec(&shards).unwrap();
    }

    #[test]
    fn test_insert() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        assert!(shards.push(shard).is_some());

        let shard2 = Shard::new("bar".to_string());
        assert!(shards.push(shard2).is_some());

        let shard3 = Shard::new("bar".to_string());
        assert!(!shards.push(shard3).is_some());
    }

    #[test]
    fn test_remove() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.push(shard);

        let shard2 = Shard::new("bar".to_string());
        shards.push(shard2.clone());

        assert!(shards.remove(&shard2.id).is_some());

        assert!(shards.remove(&shard2.id).is_none());
    }

    #[test]
    fn test_contains() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.push(shard);

        assert!(shards.contains(&"foo".to_string()));

        assert!(!shards.contains(&"bar".to_string()));
    }

    #[test]
    fn test_iter() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.push(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.push(shard2);

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
        shards.push(shard);

        let shard = shards.get(&"foo".to_string());
        assert_eq!(shard.unwrap().id, "foo".to_string());

        let shard = shards.get(&"bar".to_string());
        assert!(shard.is_none());
    }

    #[test]
    fn test_len() {
        let mut shards = Shards::new();

        let shard = Shard::new("foo".to_string());
        shards.push(shard);
        assert_eq!(shards.len(), 1);

        let shard2 = Shard::new("bar".to_string());
        shards.push(shard2.clone());
        assert_eq!(shards.len(), 2);

        let shard3 = Shard::new("bar".to_string());
        shards.push(shard3);
        assert_eq!(shards.len(), 2);

        shards.remove(&shard2.id);
        assert_eq!(shards.len(), 1);

        shards.remove(&shard2.id);
        assert_eq!(shards.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut shards = Shards::new();
        assert!(shards.is_empty());

        let shard1 = Shard::new("foo".to_string());
        shards.push(shard1.clone());
        assert!(!shards.is_empty());

        shards.remove(&shard1.id);
        assert!(shards.is_empty());
    }

    #[test]
    fn test_lookup_shards() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.push(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.push(shard2);

        let shard3 = Shard::new("baz".to_string());
        shards.push(shard3);

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

    #[test]
    fn test_lookup_shard() {
        let mut shards = Shards::new();

        let shard1 = Shard::new("foo".to_string());
        shards.push(shard1);

        let shard2 = Shard::new("bar".to_string());
        shards.push(shard2);

        let shard3 = Shard::new("baz".to_string());
        shards.push(shard3);

        let shard = shards.lookup_shard("hoge");
        assert_eq!(shard.unwrap().id, "foo".to_string());

        let shard = shards.lookup_shard("fuga");
        assert_eq!(shard.unwrap().id, "baz".to_string());

        let shard = shards.lookup_shard("piyo");
        assert_eq!(shard.unwrap().id, "bar".to_string());
    }

    #[test]
    fn test_lookup_filterd_shards() {
        let mut shards = Shards::new();

        let mut shard1 = Shard::new("foo".to_string());
        shard1.state = State::Serving;
        shards.push(shard1);

        let mut shard2 = Shard::new("bar".to_string());
        shard2.state = State::Draining;
        shards.push(shard2);

        let mut shard3 = Shard::new("baz".to_string());
        shard3.state = State::Drained;
        shards.push(shard3);

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
