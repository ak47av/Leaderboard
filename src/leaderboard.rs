use serde_json::Result as JSONResult;
use std::error::Error;
use serde::{Deserialize, Serialize};
use std::ops::Drop;
use std::io::Write;

use crate::node::Node;
use crate::storage;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Leaderboard {
    name: String,
    entries: Vec<Node>,     // Sorted by ID
    next_id: usize
}

impl Leaderboard {

    pub fn new(n: &str) -> Self {
        Leaderboard {
            name: n.to_owned(),
            entries: Vec::new(),
            next_id: 1
        }
    }

    fn insert_node_at_rank(&mut self, node: Node, rank: usize) -> Result<usize, String>{
        if rank < 1 {
            return Err(format!("Rank must be higher than 0"));
        }
        for node in &mut self.entries {
            if rank <= node.rank {
                node.rank += 1;
            }
        }
        self.entries.push(node);
        self.entries.sort();
        Ok(rank)
    }

    pub fn new_entry(&mut self, name: &str, rank: usize) {
        let rank = std::cmp::min(rank, self.entries.len()  + 1);
        let new_node = Node {name: name.to_owned(), rank: rank, id: self.next_id};
        self.next_id += 1;

        match self.insert_node_at_rank(new_node, rank) {
            Ok(r) => (), //println!("{} successfully inserted at rank: {}", name, r),
            Err(s) => println!("{}", s)
        };
        self.save_leaderboard().unwrap();
    }

    fn remove_node_by_rank(&mut self, rank: usize) -> Result<(String, usize), String> {
        if (rank > self.entries.len() ) || (rank < 1) {
            return Err(format!("No entry at Rank: {}, remove failed", rank));
        }
        let removed_name = self.entries[rank-1].name.clone();
        self.entries.remove(rank  - 1);
        for node in &mut self.entries {
            if rank < node.rank {
                node.rank -= 1;
            }
        }
        self.save_leaderboard().unwrap();
        Ok((removed_name, rank))
    }

    pub fn remove(&mut self, rank:usize) {
        match self.remove_node_by_rank(rank) {
            Ok((n, _r)) => (),//println!("{} removed from rank: {}", n, rank),
            Err(s) => println!("{}", s)
        }
    }

    pub fn change_rank(&mut self, rank:usize, to_rank: usize) -> Result<usize, String> {
        let mut temp = self.entries[rank-1].clone();
        if let Err(err) = self.remove_node_by_rank(rank) {
            return Err(format!("Change failed: {}", err));
        }
        temp.rank = to_rank;
        if let Err(err) = self.insert_node_at_rank(temp, to_rank) {
            return Err(format!("Change failed: {}", err));
        }
        Ok(to_rank)
    }

    pub fn debug_pretty(&self) {
        println!("=== LEADERBOARD DEBUG (PRETTY) ===");
        println!("Entries:");
        dbg!(&self.entries); // Shows current entries
    }

    pub fn display(&self){
        println!("========={}===========", self.name);
        for entry in &self.entries {
            println!("{}: {}", entry.rank, entry.name);
        }
        println!("========================");
    }

    pub fn write_to_vector(&self) -> Vec<String> {
        let mut s = Vec::new();
        for entry in &self.entries {
            s.push(format!("{}: {}\n", entry.rank, entry.name));
        }
        s
    }

    pub fn serialize_to_json(&self) -> JSONResult<String>{
        let json_string = serde_json::to_string(self)?;
        Ok(json_string)
    }

    pub fn intialize_from_json(json_string: &str) -> JSONResult<Self> {
        let leaderboard = serde_json::from_str(json_string)?;
        Ok(leaderboard)
    }

    fn get_leaderboard_file_location(name: &str) -> String {
        let mut file_location = "Leaderboards/".to_owned();
        file_location.push_str(name);
        file_location.push_str(".json");
        file_location
    }

    pub fn save_leaderboard(&self) -> Result<String, Box<dyn Error>> {
        let data = self.serialize_to_json()?;
        let file_location = Leaderboard::get_leaderboard_file_location(&self.name);
        storage::write_to_file(&data, &file_location)?;
        Ok(format!("Successfully saved {}", self.name).to_owned())
    }

    pub fn open_leaderboard(name: &str) -> Result<Leaderboard, Box<dyn Error>>  {
        let file_location = Leaderboard::get_leaderboard_file_location(name);
        let data = storage::read_from_file(&file_location)?;
        Ok(Leaderboard::intialize_from_json(&data)?)
    }

}

impl Drop for Leaderboard {
    fn drop(&mut self) {
        self.save_leaderboard().unwrap();
    }
}