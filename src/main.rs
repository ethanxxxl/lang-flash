use std::error::Error;
use std::env;

use csv;
use std::io::{Write, stdin, stdout};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;

#[derive(Debug)]
struct Word {
    level: u32,
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
        cards.push(Word {
            level: record[0].parse().unwrap(),
            foreign: record[1].to_string().trim().to_string(),
            native: record[2].to_string().trim().to_string(),
        });
    }

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

    let mut card = cards.iter();
    let mut is_prompting = true;

    let mut current_word = card.next().unwrap();

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
        if is_prompting {
            match c {
                Key::Char(' ') => is_prompting = false,
                _ => {},
            }
        } else {
            match c {
                Key::Char(' ') | Key::Char('1') => {
                    // increment and save file
                    current_word = card.next().unwrap();
                    is_prompting = true;
                },
                Key::Char('2') => {
                    current_word = card.next().unwrap();
                    is_prompting = true;
                    // reset to 0
                }
                _ => {},
            }
        }

        // draw current screen (tied to state)
        if is_prompting {
            write!(stdout,
                    "{}{}",
                    termion::cursor::Goto(1,1),
                    current_word.foreign,
            )?;
            write!(stdout,
                    "{}press space to flip",
                    termion::cursor::Goto(1,2),
            )?;
        } else {
            write!(stdout,
                    "{}{} -> {}",
                    termion::cursor::Goto(1,1),
                    current_word.foreign,
                    current_word.native,
            )?;
            write!(stdout,
                    "{}{}(1) Correct{}  {}(2) Incorrect{}",
                    termion::cursor::Goto(1,2),
                    termion::style::Underline,
                    termion::style::Reset,
                    termion::style::Underline,
                    termion::style::Reset,
            )?;
        }

        stdout.flush()?;
    }

    Ok(())
}
