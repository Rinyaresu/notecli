use chrono::{DateTime, Utc};
use clap::{App, Arg, SubCommand};
use crossterm::event::{self, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

const NOTES_DIR: &str = "notes";
const NOTES_JSON_FILE: &str = "notes/notes.json";

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
    date: DateTime<Utc>,
}

fn main() {
    let matches = App::new("Note Taking CLI")
        .version("0.1")
        .author("Rinyaresu")
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
    let path = std::path::Path::new(NOTES_DIR);
    if !path.exists() {
        fs::create_dir(path).expect("Erro ao criar o diretório 'notes'");
    }
}

fn create_new_note(title: &str) {
    ensure_notes_directory_exists();
    let file_path = format!("notes/{}.md", title);
    let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
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

    let mut notes: Vec<Note> = if fs::read_to_string(NOTES_JSON_FILE).is_ok() {
        let data = fs::read_to_string(NOTES_JSON_FILE).expect("Erro ao ler o arquivo JSON");
        serde_json::from_str(&data).expect("Erro ao desserializar as notas")
    } else {
        Vec::new()
    };

    notes.push(note);

    let json = serde_json::to_string(&notes).expect("Erro ao serializar a nota");
    fs::write(NOTES_JSON_FILE, json).expect("Erro ao escrever no arquivo JSON");
}

enum UserAction {
    MoveUp,
    MoveDown,
    Quit,
    Open,
    Delete,
    ToggleKeybinds,
    None,
}

fn handle_user_input() -> UserAction {
    if let Ok(event) = event::read() {
        if let event::Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up | KeyCode::Char('k') => return UserAction::MoveUp,
                KeyCode::Down | KeyCode::Char('j') => return UserAction::MoveDown,
                KeyCode::Char('q') | KeyCode::Esc => return UserAction::Quit,
                KeyCode::Enter => return UserAction::Open,
                KeyCode::Char('x') => return UserAction::Delete,
                KeyCode::Char('?') => return UserAction::ToggleKeybinds,
                _ => return UserAction::None,
            }
        }
    }
    UserAction::None
}

const KEYBINDS_TEXT: &str = "\
    ↑/k: Mover para cima
    ↓/j: Mover para baixo
    Enter: Abrir nota
    x: Deletar nota
    ?: Mostrar teclas de atalho 
    q/Esc: Sair";

fn display_tui(mut notes: Vec<Note>) {
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let _ = crossterm::terminal::enable_raw_mode();
    let mut show_keybinds = true;

    let selected_style = Style::default()
        .fg(tui::style::Color::LightMagenta)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(tui::style::Color::White);
    let keybinds_style = Style::default().fg(tui::style::Color::Black);

    terminal.clear().unwrap();

    let mut selected_index = 0;

    loop {
        terminal
            .draw(|f| {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());
                let upper_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(main_chunks[0]);

                let titles: Vec<ListItem> = notes
                    .iter()
                    .enumerate()
                    .map(|(i, note)| {
                        let title_with_date =
                            format!("{} - {}", note.title, note.date.format("%Y-%m-%d"));
                        if i == selected_index {
                            ListItem::new(title_with_date).style(selected_style)
                        } else {
                            ListItem::new(title_with_date).style(normal_style)
                        }
                    })
                    .collect();
                let selected_content = &notes[selected_index].content;

                let list =
                    List::new(titles).block(Block::default().borders(Borders::ALL).title("Notas"));
                let content = Paragraph::new(selected_content.as_str())
                    .block(Block::default().borders(Borders::ALL).title("Conteúdo"));

                f.render_widget(list, upper_chunks[0]);
                f.render_widget(content, upper_chunks[1]);
                if show_keybinds {
                    let keybinds = Paragraph::new(KEYBINDS_TEXT).style(keybinds_style).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Teclas de Atalho"),
                    );
                    f.render_widget(keybinds, main_chunks[1]);
                }
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
            UserAction::Open => {
                let note_path = format!("notes/{}.md", notes[selected_index].title);
                let editor = std::env::var("EDITOR").unwrap_or("nano".to_string());
                Command::new(editor)
                    .arg(&note_path)
                    .status()
                    .expect("Falha ao abrir o editor");
            }

            UserAction::Delete => {
                delete_note(&mut notes, selected_index);
                if selected_index >= notes.len() && selected_index > 0 {
                    selected_index -= 1;
                }
                let success_style = Style::default().fg(tui::style::Color::Green);
                show_message(&mut terminal, "Nota deletada com sucesso!", success_style);
            }
            UserAction::ToggleKeybinds => {
                show_keybinds = !show_keybinds;
            }
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

fn show_message(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    message: &str,
    style: Style,
) {
    terminal.clear().unwrap();
    terminal
        .draw(|f| {
            let size = f.size();
            let block = Paragraph::new(message)
                .style(style)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(block, size);
        })
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn delete_note(notes: &mut Vec<Note>, index: usize) {
    if index < notes.len() {
        let note = &notes[index];
        // Remover o arquivo físico
        let path = format!("notes/{}.md", note.title);
        fs::remove_file(path).expect("Falha ao deletar o arquivo da nota");

        // Remover a nota do vetor
        notes.remove(index);

        let notes_file = "notes/notes.json";
        let json = serde_json::to_string(&notes).expect("Erro ao serializar a nota");
        fs::write(notes_file, json).expect("Erro ao escrever no arquivo JSON");
    }
}
