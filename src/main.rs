mod leaderboard;
mod storage;
mod node;
mod app;

use std::error::Error;

use app::App;

fn main() -> color_eyre::Result<(), Box<dyn Error>> {

    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new().unwrap();
    let result = app.run(terminal)?;
    ratatui::restore();

    // we must be able to create new leaderboards from the App module

    Ok(result)
}
