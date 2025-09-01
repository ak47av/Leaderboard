use std::io::{Write};
use std::fs::File;

pub fn write_to_file(str: &str, file_location: &str) -> std::io::Result<()> {
    let mut file = File::create(file_location)?;
    file.write_all(str.as_bytes())?;
    Ok(())
}

pub fn read_from_file(file_location: &str) -> std::io::Result<String> {
    let str = std::fs::read_to_string(file_location)?;
    Ok(str)
}