use std::io::{Write};
use std::fs::File;

pub fn write_to_file(json_str: &str) -> std::io::Result<String> {
    let mut file = File::create("leaderboard.json")?;
    file.write_all(json_str.as_bytes())?;
    Ok("Successfully wrote to file".to_owned())
}

pub fn read_from_file() -> std::io::Result<String> {
    let json_str = std::fs::read_to_string("leaderboard.json")?;
    Ok(json_str)
}