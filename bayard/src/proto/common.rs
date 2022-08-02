use num::traits::FromPrimitive;

use crate::proto::index::query::Kind;

use super::index::sort::Order;

#[allow(clippy::should_implement_trait)]
impl Kind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Kind::All),
            "boolean" => Some(Kind::Boolean),
            "boost" => Some(Kind::Boost),
            "fuzzy_term" => Some(Kind::FuzzyTerm),
            "phrase" => Some(Kind::Phrase),
            "query_string" => Some(Kind::QueryString),
            "range" => Some(Kind::Range),
            "regex" => Some(Kind::Regex),
            "term" => Some(Kind::Term),
            _ => None,
        }
    }
}

impl FromPrimitive for Order {
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            0 => Some(Order::Unknown),
            1 => Some(Order::Asc),
            2 => Some(Order::Desc),
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Order::Unknown),
            1 => Some(Order::Asc),
            2 => Some(Order::Desc),
            _ => None,
        }
    }

    fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(Order::Unknown),
            1 => Some(Order::Asc),
            2 => Some(Order::Desc),
            _ => None,
        }
    }
}
