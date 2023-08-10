extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let title = matches.value_of("TITLE").unwrap().to_string();
        create_new_note(&title);
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
