mod model;

use model::{Leaderboard, Node};

fn main() {
    let mut leaderboard = Leaderboard::new();

    leaderboard.insertNode("Hades".to_owned(), 1);
    leaderboard.insertNode("Risebreak".to_owned(), 2);
    leaderboard.insertNode("Witcher 3".to_owned(), 2);
    leaderboard.insertNode("Sifu".to_owned(), 4);
    leaderboard.insertNode("Read Dead Redemption 2".to_owned(), 1);
    leaderboard.debug_pretty();
    leaderboard.display();
}
