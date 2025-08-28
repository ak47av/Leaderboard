use std::collections::{HashMap};
use std::cmp::Ordering;

type ID = u32;
type RANK = u32;

#[derive(Debug)]
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

pub struct Leaderboard {
    entries: Vec<Node>,     // Sorted by ID
}

impl Leaderboard {

    pub fn new() -> Self {
        Leaderboard {
            entries: Vec::new(),
        }
    }

    pub fn insertNode(&mut self, name: String, rank: u32){
        let newID = self.entries.len() as u32 +1;
        let newNode = Node {name: name, rank: rank, id: newID};
        
        for node in &mut self.entries {
            if rank <= node.rank {
                node.rank += 1;
            }
        }

        self.entries.push(newNode);
        self.entries.sort();
    }

    pub fn debug_pretty(&self) {
        println!("=== LEADERBOARD DEBUG (PRETTY) ===");
        println!("Entries:");
        dbg!(&self.entries); // Shows current entries
    }

    pub fn display(&self){
        println!("=====LEADERBOARD=======");
        for entry in &self.entries {
            println!("{}: {}", entry.rank, entry.name);
        }
        println!("========================");
    }
}
