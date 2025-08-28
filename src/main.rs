mod model;

use model::{Leaderboard};

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
    // leaderboard.display();
    let jstr = leaderboard.serialize_to_json().unwrap();
    let mut newlb = Leaderboard::intialize_from_json(jstr).unwrap();

    newlb.new_entry("Cyberpunk 2077".to_owned(), 10);
    newlb.display();
}
