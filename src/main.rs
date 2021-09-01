use std::error::Error;
use std::env;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::io::{Write, stdin, stdout};
use std::fs;
use std::path::{Path, PathBuf};

use csv;
use serde_json::{self, Value};
use serde::{Deserialize, Serialize};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;
use rand::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CardData {
    answer: String,
    level: u64,
    last_viewed: SystemTime,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut file_path = PathBuf::new();

    file_path.push(Path::new(args.get(1).expect("error! expected 1 argument, got 0")));
    let mut meta_data_path = file_path.clone();
    meta_data_path.set_file_name({
        let mut name = file_path
            .file_name()
            .expect("error! You gave a stupid input, you got a stupid output")
            .to_os_string()
            .into_string()
            .expect("error! illegal file name.");

        name.insert(0, '.');
        name
    });
    meta_data_path.set_extension("json");

    let mut cached_cards: HashMap<String, CardData> = HashMap::new();
    // if a data file exists, populate the cards hash.
    if let Ok(data) = fs::read_to_string(meta_data_path.clone()) {
        cached_cards = serde_json::from_str(&data.as_str()).expect("json error");
    }

    let mut cards: HashMap<String, CardData> = HashMap::new();
    let mut csv_reader = csv::Reader::from_path(file_path)?;
    for result in csv_reader.records() {
        let record = result?;

        // update json file with new / removed cards
        let key = record[0].to_string();
        if let Some(card_data) = cached_cards.remove(&key) {
            cards.insert(key, card_data);
        } else {
            cards.insert(
                key,
                CardData {
                    answer: record[1].to_string(),
                    level: 0,
                    last_viewed: UNIX_EPOCH,
                });
        }
    }

    // filter out cards which aren't due
    let mut due_cards = cards.iter_mut().filter(|(_, card)| {
        let time_elapsed = SystemTime::now().duration_since(card.last_viewed).unwrap();
        match card.level {
            0 => time_elapsed >= Duration::from_secs(0),
            1 => time_elapsed >= Duration::from_secs(60),
            2 => time_elapsed >= Duration::from_secs(60 * 10),
            3 => time_elapsed >= Duration::from_secs(60 * 60 * 12),
            4 => time_elapsed >= Duration::from_secs(60 * 60 * 24),
            5 => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 5),
            6 => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 20),
            _ => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 60),
        }
    }).collect::<Vec<(&String, &mut CardData)>>();

    if due_cards.is_empty() {
        println!("no cards due!");
        return Ok(());
    }

    let mut rng = thread_rng();
    due_cards.shuffle(&mut rng);
    let mut due_cards = due_cards.iter_mut();

    // we can unwrap here because there will be at least 1 card
    let mut current_card = due_cards.next().unwrap();

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode()?;
    let stdin = stdin();
    let stdin = stdin.lock();

    write!(stdout,
           "{}{}press enter to begin flash cards",
           termion::clear::All,
           termion::cursor::Goto(1,1),
    )?;

    stdout.flush()?;

    enum State {
        Prompt,
        ShowAnswer,
        SaveExit,
    }

    let mut state = State::Prompt;

    for c in stdin.keys() {
        write!(stdout,
               "{}",
               termion::clear::All
        )?;
        let c = c?;

        // common keys
        match c {
            Key::Esc=> break,
            _ => {},
        }

        // Manage state
        match state {
            State::Prompt => {
                match c {
                    Key::Char(' ') => state = State::ShowAnswer,
                    _ => {},
                }
            },
            State::ShowAnswer => {
                match c {
                    Key::Char(' ') | Key::Char('1') => {
                        current_card.1.level += 1;
                        current_card.1.last_viewed = SystemTime::now();
                        if let Some(next) = due_cards.next() {
                            current_card = next;
                            state = State::Prompt;
                        } else {
                            state = State::SaveExit;
                        }
                    },
                    Key::Char('2') => {
                        current_card.1.level = 0;
                        current_card.1.last_viewed = SystemTime::now();
                        if let Some(next) = due_cards.next() {
                            current_card = next;
                            state = State::Prompt;
                        } else {
                            state = State::SaveExit;
                        }
                    }
                    _ => {},
                }
            },
            State::SaveExit => {
                let json = serde_json::to_string(&cards.clone()).expect("error! couldn't save json!");
                fs::write(meta_data_path.clone(), json);
                return Ok(());
            },
        }

        // draw current screen (tied to state)
        let (question, answer) = (current_card.0, &current_card.1.answer);
        match state {
            State::Prompt => {
                write!(stdout,
                        "{}{}",
                        termion::cursor::Goto(1,1),
                        question,
                )?;
                write!(stdout,
                        "{}press space to flip",
                        termion::cursor::Goto(1,2),
                )?;
            },
            State::ShowAnswer => {
                write!(stdout,
                        "{}{} -> {}",
                        termion::cursor::Goto(1,1),
                        question,
                        answer,
                )?;
                write!(stdout,
                        "{}{}(1) Correct{}  {}(2) Incorrect{}",
                        termion::cursor::Goto(1,2),
                        termion::style::Underline,
                        termion::style::Reset,
                        termion::style::Underline,
                        termion::style::Reset,
                )?;
            },
            State::SaveExit => {

            }
        }

        stdout.flush()?;
    }

    Ok(())
}
