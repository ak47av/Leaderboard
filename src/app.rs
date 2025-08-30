use std::collections::HashMap;
use std::error::Error;
use std::ops::Drop;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Tabs, List, Clear},
    symbols::border,
    symbols,
    layout::Layout,
    prelude::{Direction, Constraint, Rect}
};
use tui_textarea::{Input, Key, TextArea};

use crate::leaderboard::Leaderboard;
use crate::storage::{read_from_file, write_to_file};

#[derive(Debug, PartialEq, Eq)]
enum AppState {
    Show,
    NewLDB,
    NewEntry
}

#[derive(Debug)]
enum EntryFocus {
    Name,
    Rank,
}

#[derive(Debug)]
pub struct App <'a>{
    leaderboard_names: Vec<String>,
    running: bool,
    current_leaderboard_index: usize,
    current_leaderboard: Leaderboard,
    state: AppState,
    entry_name_input: TextArea<'a>,
    entry_name: String,
    entry_rank_input: TextArea<'a>,
    entry_rank: usize,
    ldb_name_input: TextArea<'a>,
    ldb_name: String,
    focus: EntryFocus
}

impl <'a> App <'_> {
    
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let json_str = read_from_file("Leaderboards/Leaderboards.json")?;
        let hmap: HashMap<String, Vec<String>> = serde_json::from_str(&json_str)?;
        let mut ldb_vec: Vec<String> = Vec::new();
        if let Some(ldbs) = hmap.get("leaderboards") {
            for ldb in ldbs {
                ldb_vec.push(ldb.to_string());
            }
        }
        let lb = Leaderboard::open_leaderboard(&ldb_vec[0])?;
        Ok(App {
            leaderboard_names: ldb_vec,
            running: true,
            current_leaderboard_index: 0,
            current_leaderboard: lb,
            state: AppState::Show,
            entry_name_input: TextArea::default(),
            entry_rank_input: TextArea::default(),
            ldb_name_input: TextArea::default(),
            entry_name: String::new(),
            entry_rank: 100,
            focus: EntryFocus::Name,
            ldb_name: String::new(),
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
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),     // for Tabs
            Constraint::Min(0),        // for leaderboard content
        ])
        .split(frame.area());

        let tab_block = Block::bordered()
            .title(Line::from("Leaderboards".to_string()).bold().centered());
        frame.render_widget(
            Tabs::new(self.leaderboard_names.clone())
            .block(tab_block)
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(self.current_leaderboard_index)
            .divider(symbols::DOT)
            .padding("|", "|"),
            chunks[0]
        );

        let title_txt = &self.leaderboard_names[self.current_leaderboard_index];
        let title = Line::from(title_txt.clone().bold());
        let instructions = Line::from(vec![
            " <-".into(),
            "<Left> ".blue().bold(),
            " ->".into(),
            "<Right> ".blue().bold(),
            " New Entry".into(),
            "<Ctrl+n> ".blue().bold(),
            " New Leaderboard".into(),
            "<Ctrl+l> ".blue().bold(),
            " Back".into(),
            "<b> ".blue().bold(),
            " X".into(),
            "<Ctrl+C> ".blue().bold(),
        ]);
        let para_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        match self.state {
            AppState::Show => {
                frame.render_widget(Clear, chunks[1]);
                let ldb_entries = self.current_leaderboard.write_to_vector();
                frame.render_widget(
                    List::new(ldb_entries)
                        .block(para_block),
                    chunks[1]
                );
            },
            AppState::NewEntry => {
                let entry_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),     // for Tabs
                    Constraint::Length(3),        // for leaderboard content
                ])
                .split(chunks[1]);
                self.entry_name_input.set_block(
                    Block::bordered()
                    .title("New Entry name"),
                );
                frame.render_widget(&self.entry_name_input, entry_chunks[0]);
                self.entry_rank_input.set_block(
                    Block::bordered()
                    .title("New Entry rank"),
                );
                frame.render_widget(&self.entry_rank_input, entry_chunks[1]);
            },
            AppState::NewLDB => {
                self.ldb_name_input.set_block(
                    Block::bordered()
                    .title("New Leaderboard"),
                );
                frame.render_widget(&self.ldb_name_input, chunks[1]);
            }
        }

    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if let Ok(event) = crossterm::event::read() {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // send key to the textarea firstname
                match self.state {
                    AppState::NewEntry => {
                        match self.focus {
                            EntryFocus::Name => self.entry_name_input.input(key),
                            EntryFocus::Rank => self.entry_rank_input.input(key),
                        };

                        match key.code {
                            KeyCode::Enter => {
                                match self.focus {
                                    EntryFocus::Name => {
                                        if let Some(name) = self.entry_name_input.lines().get(0) {
                                            self.entry_name = name.clone();
                                        }
                                        // Move focus to rank input
                                        self.focus = EntryFocus::Rank;
                                    }
                                    EntryFocus::Rank => {
                                        if let Some(rank_line) = self.entry_rank_input.lines().get(0) {
                                            if let Ok(rank) = rank_line.parse::<usize>() {
                                                self.entry_rank = rank;
                                                //println!("Submitted name: {}, rank: {}", self.entry_name, self.entry_rank);
                                                // Done editing, maybe go back to main state
                                                self.current_leaderboard.new_entry(&self.entry_name, self.entry_rank);
                                                self.state = AppState::Show;
                                                self.focus = EntryFocus::Rank;
                                            } else {
                                                println!("Rank must be a number!");
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                self.state = AppState::Show;
                            }
                            _ => {}
                    }
                    }
                    AppState::NewLDB => {
                        self.ldb_name_input.input(key);
                    }
                    _ => {}
                }

                // handle custom key logic
                match key.code {
                    KeyCode::Esc => {
                        // exit or do something on ESC
                    }
                    KeyCode::Enter => {
                        if self.state == AppState::NewEntry {
                            if let Some(name_line) = self.entry_name_input.lines().get(0) {
                                self.entry_name = name_line.clone();
                            }
                            if let Some(rank_line) = self.entry_rank_input.lines().get(0) {
                                if let Ok(rank) = rank_line.parse::<usize>() {
                                    self.entry_rank = rank;
                                }
                            }
                        } else if self.state == AppState::NewLDB {
                            if let Some(name_line) = self.ldb_name_input.lines().get(0) {
                                let name_line = name_line.to_string();
                                self.new_leaderboard(&name_line).unwrap();
                                self.state = AppState::Show;
                            }
                        }
                    }
                    _ => {}
                }

                // if you also want your own handler:
                self.on_key_event(key);
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // setup for inputs
        match (key.modifiers, key.code) {
            // (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            // Add other key handlers here.
            (_, KeyCode::Left) => if self.state == AppState::Show { self.show_prev_leaderboard().unwrap() },
            (_, KeyCode::Right) => if self.state == AppState::Show { self.show_next_leaderboard().unwrap() },
            (KeyModifiers::CONTROL, KeyCode::Char('n')) => self.state = AppState::NewEntry,
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => self.state = AppState::NewLDB,
            (_, KeyCode::Esc) => self.state = AppState::Show,

            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn show_prev_leaderboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_leaderboard_index > 0 {
            self.current_leaderboard_index -= 1;
            self.current_leaderboard = Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index])?;
        }
        Ok(())
    }

    fn show_next_leaderboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_leaderboard_index < self.leaderboard_names.len()-1 {
            self.current_leaderboard_index += 1;
            self.current_leaderboard = Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index])?;
        }
        Ok(())
    }

}

impl Drop for App <'_> {
    fn drop(&mut self) {
        let v : Vec<&String> = Vec::from_iter(&self.leaderboard_names);
        let ldb_names: Vec<String> = v.into_iter().cloned().collect();
        let mut hmap: HashMap<String, Vec<String>> = HashMap::new();
        hmap.insert("leaderboards".to_owned(), ldb_names);
        let json_str = serde_json::to_string(&hmap).unwrap();
        write_to_file(&json_str, "Leaderboards/Leaderboards.json").unwrap();
    }
}