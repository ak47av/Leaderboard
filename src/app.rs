use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::hash::Hash;
use std::ops::Drop;

use crate::leaderboard::Leaderboard;
use crate::storage::{read_from_file, write_to_file};

pub struct App {
    leaderboard_names: HashSet<String>,
}

impl App {
    
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let json_str = read_from_file("Leaderboards/Leaderboards.json")?;
        let hmap: HashMap<String, Vec<String>> = serde_json::from_str(&json_str)?;
        let mut hset: HashSet<String> = HashSet::new();
        for ldbs in hmap.get("leaderboards") {
            for ldb in ldbs {
                hset.insert(ldb.to_string());
            }
        }
        Ok(App {
            leaderboard_names: hset
        })
    }

    pub fn new_leaderboard(&mut self, name: &str) -> Result<Leaderboard, Box<dyn Error>> {
        if self.leaderboard_names.contains(name) {
            return Err(format!("Leaderboard named {} already exists!", name).into());
        }
        let new_lb = Leaderboard::new(name);
        new_lb.save_leaderboard()?;
        self.leaderboard_names.insert(name.to_string());
        Ok(new_lb)
    }

    pub fn open_leaderboard(&mut self, name: &str) -> Result<Leaderboard, Box<dyn Error>> {
        if !self.leaderboard_names.contains(name) {
            return Err(format!("Leaderboard named {} does not exist!", name).into());
        }
        let lb = Leaderboard::open_leaderboard(name)?;
        Ok(lb)
    }

    pub fn remove_leaderboard(&mut self, name: &str) -> Result<(), Box<dyn Error>>{
        if !self.leaderboard_names.contains(name) {
            return Err(format!("Leaderboard named {} does not exist!", name).into());
        }
        let mut file_name = "Leaderboards/".to_owned();
        file_name.push_str(name);
        file_name.push_str(".json");
        std::fs::remove_file(file_name)?;
        self.leaderboard_names.remove(name);
        Ok(())
    }

}

impl Drop for App {
    fn drop(&mut self) {
        let v : Vec<&String> = Vec::from_iter(&self.leaderboard_names);
        let ldb_names: Vec<String> = v.into_iter().cloned().collect();
        let mut hmap: HashMap<String, Vec<String>> = HashMap::new();
        hmap.insert("leaderboards".to_owned(), ldb_names);
        let json_str = serde_json::to_string(&hmap).unwrap();
        write_to_file(&json_str, "Leaderboards/Leaderboards.json").unwrap();
    }
}