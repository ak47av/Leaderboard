mod leaderboard;
mod storage;
mod node;
mod app;

use leaderboard::{Leaderboard};
use app::App;

fn main() {

    let mut app = App::new().unwrap();
    // we must be able to create new leaderboards from the App module

    // let mut games = Leaderboard::open_leaderboard("Games").unwrap();
    let mut games = app.open_leaderboard("Games").unwrap();

    games.display();
    games.save_leaderboard().unwrap();

    let mut cbe_restaurants= app.open_leaderboard("Coimbatore_Restaurants").unwrap();

    // cbe_restaurants.new_entry("Valarmathi Chettinad", 1);
    // cbe_restaurants.new_entry("Annalakshmi Chettinad", 2);
    cbe_restaurants.display();

    // let mut movies = Leaderboard::new("Movies");
    app.remove_leaderboard("Movies").unwrap();

    // movies.display();
    // movies.save_leaderboard();
}
