mod leaderboard;
mod storage;
mod node;
mod app;
mod log;

use std::{error::Error};
use app::App;
use log::Log;

fn main() -> color_eyre::Result<(), Box<dyn Error>> {

    color_eyre::install()?;
    let log = Log::new("app.log").unwrap();
    let mut app = App::new(log)?;
    let terminal = ratatui::init();
    let result = app.run(terminal)?;
    ratatui::restore();

    // we must be able to create new leaderboards from the App module

    Ok(result)
}
