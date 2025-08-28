mod leaderboard;
mod storage;
mod node;

use leaderboard::{Leaderboard};

fn main() {
    let mut games = Leaderboard::open_leaderboard("Games").unwrap();

    games.display();
    games.save_leaderboard().unwrap();


    let mut movies = Leaderboard::new("Movies");

    movies.display();
    movies.save_leaderboard();
}
