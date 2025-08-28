use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

type ID = usize;
type RANK = usize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub name: String,
    pub rank: RANK,
    pub id: ID
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl Eq for Node {
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}
