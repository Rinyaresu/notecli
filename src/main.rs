extern crate clap;
extern crate prettytable;
extern crate serde;
extern crate serde_json;

use chrono::{DateTime, Utc};
use clap::{App, Arg, SubCommand};
use crossterm::event::{self, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
    date: DateTime<Utc>,
}

fn main() {
    let matches = App::new("Note Taking CLI")
        .version("0.1")
        .author("Seu Nome")
        .about("CLI para tomar notas em Markdown")
        .subcommand(
            SubCommand::with_name("new")
                .about("Cria uma nova nota")
                .arg(
                    Arg::with_name("TITLE")
                        .help("O título da nota")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("Lista as notas"))
        .get_matches();

    match matches.subcommand_name() {
        Some("new") => {
            let sub_matches = matches.subcommand_matches("new").unwrap();
            let title = sub_matches.value_of("TITLE").unwrap().to_string();
            create_new_note(&title);
        }
        Some("list") => list_notes(),
        _ => println!("Comando não reconhecido."),
    }
}

fn ensure_notes_directory_exists() {
    let path = std::path::Path::new("notes");
    if !path.exists() {
        fs::create_dir(path).expect("Erro ao criar o diretório 'notes'");
    }
}

fn create_new_note(title: &str) {
    ensure_notes_directory_exists();
    let file_path = format!("notes/{}.md", title);
    let editor = std::env::var("EDITOR").unwrap_or("nano".to_string());
    Command::new(editor)
        .arg(&file_path)
        .status()
        .expect("Falha ao abrir o editor");

    let content = fs::read_to_string(&file_path).expect("Erro ao ler o arquivo");
    let note = Note {
        title: title.to_string(),
        content,
        date: Utc::now(),
    };

    save_note(note);
}

fn save_note(note: Note) {
    ensure_notes_directory_exists();
    let notes_file = "notes/notes.json";
    let mut notes: Vec<Note> = Vec::new();

    if fs::metadata(notes_file).is_ok() {
        let data = fs::read_to_string(notes_file).expect("Erro ao ler o arquivo JSON");
        notes = serde_json::from_str(&data).expect("Erro ao desserializar as notas");
    }

    notes.push(note);

    let json = serde_json::to_string(&notes).expect("Erro ao serializar a nota");
    fs::write(notes_file, json).expect("Erro ao escrever no arquivo JSON");
}

enum UserAction {
    MoveUp,
    MoveDown,
    Quit,
    None,
}

fn handle_user_input() -> UserAction {
    if let Ok(event) = event::read() {
        if let event::Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up | KeyCode::Char('k') => return UserAction::MoveUp,
                KeyCode::Down | KeyCode::Char('j') => return UserAction::MoveDown,
                KeyCode::Char('q') | KeyCode::Esc => return UserAction::Quit,
                _ => return UserAction::None,
            }
        }
    }
    UserAction::None
}

fn display_tui(notes: Vec<Note>) {
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let _ = crossterm::terminal::enable_raw_mode();

    terminal.clear().unwrap();

    let mut selected_index = 0;

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.size());
                let titles: Vec<ListItem> = notes
                    .iter()
                    .enumerate()
                    .map(|(i, note)| {
                        if i == selected_index {
                            ListItem::new(note.title.clone())
                                .style(Style::default().add_modifier(Modifier::BOLD))
                        } else {
                            ListItem::new(note.title.clone())
                        }
                    })
                    .collect();
                let selected_content = &notes[selected_index].content;

                let list =
                    List::new(titles).block(Block::default().borders(Borders::ALL).title("Notas"));
                let content = Paragraph::new(selected_content.as_str())
                    .block(Block::default().borders(Borders::ALL).title("Conteúdo"));

                f.render_widget(list, chunks[0]);
                f.render_widget(content, chunks[1]);
            })
            .unwrap();

        match handle_user_input() {
            UserAction::MoveUp => {
                if selected_index > 0 {
                    selected_index -= 1;
                }
            }
            UserAction::MoveDown => {
                if selected_index < notes.len() - 1 {
                    selected_index += 1;
                }
            }
            UserAction::Quit => break,
            UserAction::None => {}
        }
    }
    let _ = crossterm::terminal::disable_raw_mode();
}

fn list_notes() {
    let notes_file = "notes/notes.json";

    match fs::read_to_string(notes_file) {
        Ok(data) => match serde_json::from_str::<Vec<Note>>(&data) {
            Ok(notes) => display_tui(notes),
            Err(e) => {
                println!("Erro ao desserializar as notas: {:?}", e);
            }
        },
        Err(e) => {
            println!("Erro ao ler o arquivo {}: {:?}", notes_file, e);
        }
    }
}
