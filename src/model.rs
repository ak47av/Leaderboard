use std::cmp::Ordering;

type ID = usize;
type RANK = usize;

#[derive(Debug, Clone)]
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
    next_id: usize
}

impl Leaderboard {

    pub fn new() -> Self {
        Leaderboard {
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

    pub fn new_entry(&mut self, name: String, rank: usize) {
        let rank = std::cmp::min(rank, self.entries.len()  + 1);
        let new_node = Node {name: name.clone(), rank: rank, id: self.next_id};
        self.next_id += 1;

        match self.insert_node_at_rank(new_node, rank) {
            Ok(r) => println!("{} successfully inserted at rank: {}", name, r),
            Err(s) => println!("{}", s)
        }
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
        Ok((removed_name, rank))
    }

    pub fn remove(&mut self, rank:usize) {
        match self.remove_node_by_rank(rank) {
            Ok((n, _r)) => println!("{} removed from rank: {}", n, rank),
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
        println!("=====LEADERBOARD=======");
        for entry in &self.entries {
            println!("{}: {}", entry.rank, entry.name);
        }
        println!("========================");
    }
}
