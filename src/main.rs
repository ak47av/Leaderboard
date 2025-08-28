mod model;
mod storage;

use model::{Leaderboard};

fn main() {
    let mut games = Leaderboard::open_leaderboard("Games".to_owned()).unwrap();
    games.new_entry("Horizon Zero dawn".to_owned(), 6);

    games.save_leaderboard();
    games.display();
}
