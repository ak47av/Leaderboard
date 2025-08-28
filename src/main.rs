mod model;
mod storage;

use bincode::de::read;
use model::{Leaderboard};

use crate::storage::read_from_file;

fn main() {
    let mut leaderboard = Leaderboard::new();

    leaderboard.new_entry("Hades".to_owned(), 1);
    leaderboard.new_entry("Risebreak".to_owned(), 2);
    leaderboard.new_entry("Witcher 3".to_owned(), 2);
    leaderboard.new_entry("God of War".to_owned(), 9);
    leaderboard.new_entry("Read Dead Redemption 2".to_owned(), 1);

    leaderboard.remove(89);
    leaderboard.remove(5);
    leaderboard.new_entry("Ori and the will of the wisps".to_owned(), 45);
    leaderboard.change_rank(1, 4).unwrap();
    
    let jstr = leaderboard.serialize_to_json().unwrap();
    storage::write_to_file(&jstr).unwrap();

    let read_string = read_from_file().unwrap();
    let mut newlb = Leaderboard::intialize_from_json(read_string).unwrap();

    newlb.new_entry("Cyberpunk 2077".to_owned(), 10);
    newlb.display();
}
