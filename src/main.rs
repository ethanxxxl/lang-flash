use std::error::Error;
use std::env;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::io::{Write, stdin, stdout};

use csv;
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;
use rand::prelude::*;

#[derive(Debug)]
struct Word {
    level: u32,
    last_viewed: SystemTime,
    foreign: String,
    native: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let file_path = args.get(1).expect("error! expected 1 argument, got 0");

    let mut rdr = csv::Reader::from_path(file_path)?;

    let mut cards = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let last_viewed = Duration::from_secs(record[1].parse().unwrap());
        let last_viewed = SystemTime::UNIX_EPOCH + last_viewed;
        cards.push(Word {
            level: record[0].parse().unwrap(),
            last_viewed,
            foreign: record[2].to_string().trim().to_string(),
            native: record[3].to_string().trim().to_string(),
        });
    }

    let mut due_cards = cards.iter_mut().filter(|card| {
        let time_elapsed = SystemTime::now().duration_since(card.last_viewed).unwrap();
        match card.level {
            0 => time_elapsed >= Duration::from_secs(60),
            1 => time_elapsed >= Duration::from_secs(60 * 10),
            2 => time_elapsed >= Duration::from_secs(60 * 60 * 12),
            3 => time_elapsed >= Duration::from_secs(60 * 60 * 24),
            4 => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 5),
            5 => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 20),
            _ => time_elapsed >= Duration::from_secs(60 * 60 * 24 * 60),
        }
    }).collect::<Vec<&mut Word>>();

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
                        current_card.level += 1;
                        if let Some(next) = due_cards.next() {
                            current_card = next;
                            state = State::Prompt;
                        } else {
                            state = State::SaveExit;
                        }
                    },
                    Key::Char('2') => {
                        current_card.level = 0;
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
                return Ok(());
                // TODO implement save and exit
            },
        }

        // draw current screen (tied to state)
        match state {
            State::Prompt => {
                write!(stdout,
                        "{}{}",
                        termion::cursor::Goto(1,1),
                        current_card.foreign,
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
                        current_card.foreign,
                        current_card.native,
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
