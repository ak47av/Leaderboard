use std::{collections::HashMap};
use std::error::Error;
use std::ops::Drop;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::Clear;
use ratatui::{
    layout::Layout, prelude::{Constraint, Direction}, style::{Style, Stylize}, symbols, text::Line, widgets::{Block, List, ListItem, ListState, Paragraph, Tabs}, DefaultTerminal, Frame
};
use tui_textarea::{TextArea};
use std::path::Path;

use crate::leaderboard::Leaderboard;
use crate::storage::{read_from_file, write_to_file};
use crate::log::Log;

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
    current_leaderboard: Option<Leaderboard>,
    current_entry: usize,
    yanked_entry: Option<usize>,
    state: AppState,
    entry_name_input: TextArea<'a>,
    entry_name: String,
    entry_rank_input: TextArea<'a>,
    entry_rank: usize,
    ldb_name_input: TextArea<'a>,
    ldb_name: String,
    focus: EntryFocus,
    logger: Log,
    list_state: ListState
}

impl <'a> App <'_> {

    fn create_folder_and_main_json() -> Result<(), Box<dyn Error>> {
        std::fs::create_dir("Leaderboards")?;
        write_to_file("{\"leaderboards\":[]}", "Leaderboards/Leaderboards.json")?;
        //write_to_file("{\"name\":\"Games\",\"entries\":[],\"next_id\":33}", "Leaderboards/First_Leaderboard.json")?;
        Ok(())
    }
    
    pub fn new(mut log: Log) -> Result<Self, Box<dyn Error>> {
        if !Path::new("Leaderboards").exists(){
            log.write("Leaderboards directory exists");
            App::create_folder_and_main_json()?;
        }
        let json_str = read_from_file("Leaderboards/Leaderboards.json")?;
        let hmap: HashMap<String, Vec<String>> = serde_json::from_str(&json_str)?;
        let mut ldb_vec: Vec<String> = Vec::new();
        if let Some(ldbs) = hmap.get("leaderboards") {
            for ldb in ldbs {
                ldb_vec.push(ldb.to_string());
            }
        }
        let lb: Option<Leaderboard>;
        if ldb_vec.is_empty() {
            lb = None;
        } else {
            lb = Some(Leaderboard::open_leaderboard(&ldb_vec[0])?);
        }
        Ok(App {
            leaderboard_names: ldb_vec,
            running: true,
            current_leaderboard_index: 0,
            current_leaderboard: lb,
            current_entry: 0,
            yanked_entry: None,
            state: AppState::Show,
            entry_name_input: TextArea::default(),
            entry_rank_input: TextArea::default(),
            ldb_name_input: TextArea::default(),
            entry_name: String::new(),
            entry_rank: 100,
            focus: EntryFocus::Name,
            ldb_name: String::new(),
            logger: log,
            list_state: ListState::default()
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
        if !std::path::Path::new(&file_name).exists() {
            self.logger.write("File does not exist!");
        } else {
            self.logger.write("File exists, attempting deletion");
        }
        match std::fs::remove_file(file_name) {
            Ok(()) => self.logger.write(format!("REMOVE LDB file {}.json Succeeded!", &self.leaderboard_names[index])),
            Err(e) => self.logger.write(format!("REMOVE LDB file {}.json Failed: {}", &self.leaderboard_names[index], e))
        };
        match self.open_leaderboard(0) {
            Ok(ldb) => self.current_leaderboard = Some(ldb),
            Err(e) => {
                self.current_leaderboard = None;
                self.logger.write(format!("OPEN LDB {} Failed: {}", &self.leaderboard_names[index], e))
            }
        }
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
            Constraint::Length(3),     // for Messages 
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

        let title_txt: String;
        match &self.leaderboard_names.get(self.current_leaderboard_index) {
            Some(s) => title_txt = s.to_string(),
            None => title_txt = "Add a new Leaderboard".to_string()
        }
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
            " Delete".into(),
            "<Ctrl+d> ".blue().bold(),
            " X".into(),
            "<Ctrl+C> ".blue().bold(),
        ]);
        let para_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        match self.state {
            AppState::Show => {
                match &self.current_leaderboard {
                    Some(ldb) => {
                        let ldb_entries = ldb.write_to_vector();
                        let mut items = Vec::new();
                        for i in 0..ldb.len() {
                            let mut item: ListItem = Line::raw(ldb_entries[i].clone()).into();
                            if i == self.current_entry {
                                item = Line::raw(ldb_entries[i].clone()).yellow().into();
                            } 
                            match self.yanked_entry {
                                Some(e) =>
                                if i == e {
                                    item = Line::raw(ldb_entries[i].clone()).red().into();
                                },
                                None => ()
                            }
                            items.push(item);
                        }
                        let list = List::default().items(items).block(para_block);//.block(para_block);
                        frame.render_widget(Clear, chunks[1]);
                        frame.render_stateful_widget(list,chunks[1],&mut self.list_state);
                    },
                    None => {
                        let line = Line::from("Add a new leaderboard using Ctrl + l");
                        let para = Paragraph::new(line).block(para_block);
                        frame.render_widget(para, chunks[1]);
                    }
                }
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
                                        self.entry_name_input = TextArea::default();
                                        self.entry_rank_input = TextArea::default();
                                    }
                                    EntryFocus::Rank => {
                                        if let Some(rank_line) = self.entry_rank_input.lines().get(0) {
                                            if let Ok(rank) = rank_line.parse::<usize>() {
                                                self.entry_rank = rank;
                                                //println!("Submitted name: {}, rank: {}", self.entry_name, self.entry_rank);
                                                // Done editing, maybe go back to main state
                                                match &mut self.current_leaderboard{
                                                    Some(ldb) => ldb.new_entry(&self.entry_name, self.entry_rank)
                                                        .unwrap_or_else(|e| self.logger.write(
                                                            format!("Unable to create new entry: {}", e)
                                                        )),
                                                    None => {}
                                                }
                                                self.state = AppState::Show;
                                                self.focus = EntryFocus::Name;
                                            } else {
                                                println!("Rank must be a number!");
                                            }
                                        }
                                        self.entry_name_input = TextArea::default();
                                        self.entry_rank_input = TextArea::default();
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
                        match key.code {
                            KeyCode::Enter => {
                                if let Some(name_line) = self.ldb_name_input.lines().get(0) {
                                    self.ldb_name = name_line.clone();
                                    match self.new_leaderboard(&self.ldb_name.clone()) {
                                        Ok(new_ldb) => {
                                            self.current_leaderboard = Some(new_ldb);
                                            self.logger.write(format!("Created new leaderboard: {}", self.ldb_name));
                                        },
                                        Err(err) => self.logger.write(format!("Error creating new leaderboard: {}", err)),
                                    }
                                    self.state = AppState::Show;
                                    self.ldb_name_input = TextArea::default();
                                }
                            },
                            _ => {} 
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
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => self.state = AppState::NewLDB,
            (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                match &self.current_leaderboard {
                    Some(_) => self.state = AppState::NewEntry,
                    None => {}
                }
            },
            (_, KeyCode::Char('h')) => if self.state == AppState::Show { self.show_prev_leaderboard().unwrap() },
            (_, KeyCode::Char('l')) => if self.state == AppState::Show { self.show_next_leaderboard().unwrap() },
            (_, KeyCode::Esc) => { self.state = AppState::Show; self.yanked_entry = None; },
            (_, KeyCode::Up) => if self.state == AppState::Show { 
                self.show_prev_entry().unwrap();
                self.list_state.scroll_up_by(1);
            },
            (_, KeyCode::Down) => if self.state == AppState::Show { 
                self.show_next_entry().unwrap();
                self.list_state.scroll_down_by(1);
            },
            (_, KeyCode::Char('k')) => if self.state == AppState::Show { self.show_prev_entry().unwrap() },
            (_, KeyCode::Char('j')) => if self.state == AppState::Show { self.show_next_entry().unwrap() },
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => if self.state == AppState::Show { 
                match &mut self.current_leaderboard{
                    Some(ldb) => ldb.remove(self.current_entry+1),
                    None => {}
                }
            },
            (KeyModifiers::CONTROL, KeyCode::Char('x')) => if self.state == AppState::Show { 
                match &mut self.current_leaderboard{
                    Some(_ldb) => self.remove_leaderboard(self.current_leaderboard_index)
                            .unwrap_or_else(|e| self.logger.write(format!("Unable to remove leaderboard: {}", e))), 
                    None => {}
                }
                self.current_leaderboard_index = 0;
            },
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => if self.state == AppState::Show { 
                self.yanked_entry = Some(self.current_entry);
            },
            (_, KeyCode::Char('p')) => if self.state == AppState::Show { 
                match self.yanked_entry {
                    Some(e) => {
                        if e == self.current_entry {
                            ()
                        }
                        else {
                            match &mut self.current_leaderboard{
                                Some(ldb) => ldb.change_rank(e+1, self.current_entry+1)
                                .unwrap_or_else(|e| self.logger.write(format!("Unable to change rank of entry: {}", e))), 
                                None => {}
                            }
                            self.yanked_entry = None;
                        }
                    },
                    None => ()
                }
            },
            _ => {}

        }
    }
    
    fn quit(&mut self) {
        self.running = false;
    }

    fn show_prev_leaderboard(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.leaderboard_names.is_empty() {
            if self.current_leaderboard_index > 0 {
                self.current_leaderboard_index -= 1;
                match Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index]) {
                    Ok(ldb) => self.current_leaderboard = Some(ldb),
                    Err(err) => self.logger.write(format!("Unable to open previous leeaderboard: {}", err)),
                }
            }
            self.current_entry = 0;
        }
        Ok(())
    }

    fn show_prev_entry(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.current_leaderboard {
            Some(ldb) => {
                if !ldb.is_empty() {
                    if self.current_entry > 0 {
                        self.current_entry -= 1;
                    }
                }
            },
            None => {}
        }
        Ok(())
    }

    fn show_next_leaderboard(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.leaderboard_names.is_empty() {
            if self.current_leaderboard_index < self.leaderboard_names.len()-1 {
                self.current_leaderboard_index += 1;
                match Leaderboard::open_leaderboard(&self.leaderboard_names[self.current_leaderboard_index]) {
                    Ok(ldb) => self.current_leaderboard = Some(ldb),
                    Err(err) => self.logger.write(format!("Unable to open next leeaderboard: {}", err)),
                }
            }
            self.current_entry = 0;
        }
        Ok(())
    }
    
    fn show_next_entry(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.current_leaderboard {
            Some(ldb) => {
                if !ldb.is_empty() {
                    if self.current_entry < ldb.len()-1 {
                        self.current_entry += 1;
                    }
                }
            },
            None => {}
        };
        Ok(())
    }

}

impl Drop for App <'_> {
    fn drop(&mut self) {
        let v : Vec<&String> = Vec::from_iter(&self.leaderboard_names);
        let ldb_names: Vec<String> = v.into_iter().cloned().collect();
        let mut hmap: HashMap<String, Vec<String>> = HashMap::new();
        hmap.insert("leaderboards".to_owned(), ldb_names);
        match serde_json::to_string(&hmap) {
            Ok(s) => write_to_file(&s, "Leaderboards/Leaderboards.json")
                            .unwrap_or_else(|e|
                                self.logger.write(format!("Unable to write Leaderboard to file: {}", e)),
                            ),
            Err(err) => self.logger.write(format!("Unable to encode Leaderboard: {}", err)),
        }
    }
}