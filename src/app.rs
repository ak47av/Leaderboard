use std::collections::HashMap;
use std::error::Error;
use std::ops::Drop;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph},
    symbols::border,
};
use crate::leaderboard::Leaderboard;
use crate::storage::{read_from_file, write_to_file};

#[derive(Debug, Default)]
pub struct App {
    leaderboard_names: Vec<String>,
    running: bool,
    current_leaderboard_index: usize,
    current_leaderboard: Option<Leaderboard>
}

impl App {
    
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let json_str = read_from_file("Leaderboards/Leaderboards.json")?;
        // dbg!(&json_str);
        let hmap: HashMap<String, Vec<String>> = serde_json::from_str(&json_str)?;
        // dbg!(&hmap);
        let mut ldb_vec: Vec<String> = Vec::new();
        if let Some(ldbs) = hmap.get("leaderboards") {
            for ldb in ldbs {
                ldb_vec.push(ldb.to_string());
            }
        }
        // dbg!(&ldb_vec);
        let lb = Leaderboard::open_leaderboard(&ldb_vec[0])?;
        Ok(App {
            leaderboard_names: ldb_vec,
            running: true,
            current_leaderboard_index: 0,
            current_leaderboard: Some(lb)
        })
    }

    pub fn new_leaderboard(&mut self, name: &str) -> Result<Leaderboard, Box<dyn Error>> {
        if self.leaderboard_names.contains(&name.to_string()) {
            return Err(format!("Leaderboard named {} already exists!", name).into());
        }
        let new_lb = Leaderboard::new(name);
        new_lb.save_leaderboard()?;
        self.leaderboard_names.push(name.to_string());
        Ok(new_lb)
    }

    pub fn open_leaderboard(&mut self, index: usize) -> Result<Leaderboard, Box<dyn Error>> {
        if index >= self.leaderboard_names.len() {
            return Err(format!("No leaderboard at index {}", index).into());
        }
        let lb = Leaderboard::open_leaderboard(&self.leaderboard_names[index])?;
        Ok(lb)
    }

    pub fn remove_leaderboard(&mut self, index: usize) -> Result<(), Box<dyn Error>>{
        if index >= self.leaderboard_names.len() {
            return Err(format!("No leaderboard at that index {}", index).into());
        }
        let name = &self.leaderboard_names[index];
        let mut file_name = "Leaderboards/".to_owned();
        file_name.push_str(name);
        file_name.push_str(".json");
        std::fs::remove_file(file_name)?;
        self.leaderboard_names.remove(index);
        Ok(())
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }
    
    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title_txt = &self.leaderboard_names[self.current_leaderboard_index];
        let title = Line::from(title_txt.clone().bold());
        let instructions = Line::from(vec![
            " Previous ".into(),
            " <Left> ".blue().bold(),
            " Next ".into(),
            " <Right> ".blue().bold(),
            " Quit ".into(),
            " <Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let mut ldb_txt = String::new();
        match &self.current_leaderboard {
            Some(ldb) => ldb_txt = ldb.write_to_string(),
            None => ldb_txt = "Poi oombu".to_string()
        }

        let counter_text = Text::from(vec![Line::from(vec![
            ldb_txt.to_string().yellow()
        ])]);

        frame.render_widget(
            Paragraph::new(counter_text)
                .centered()
                .block(block),
            frame.area()
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Left) => self.show_prev_leaderboard().unwrap(),
            (_, KeyCode::Right) => self.show_next_leaderboard().unwrap(),
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn show_prev_leaderboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_leaderboard_index > 0 {
            self.current_leaderboard_index -= 1;
            self.current_leaderboard = Some(Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index])?);
        }
        Ok(())
    }

    fn show_next_leaderboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_leaderboard_index < self.leaderboard_names.len()-1 {
            self.current_leaderboard_index += 1;
            self.current_leaderboard = Some(Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index])?);
        }
        Ok(())
    }

}

impl Drop for App {
    fn drop(&mut self) {
        let v : Vec<&String> = Vec::from_iter(&self.leaderboard_names);
        let ldb_names: Vec<String> = v.into_iter().cloned().collect();
        let mut hmap: HashMap<String, Vec<String>> = HashMap::new();
        hmap.insert("leaderboards".to_owned(), ldb_names);
        let json_str = serde_json::to_string(&hmap).unwrap();
        write_to_file(&json_str, "Leaderboards/Leaderboards.json").unwrap();
    }
}