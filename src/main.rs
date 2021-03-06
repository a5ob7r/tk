use std::collections::HashMap;
use std::env;
use std::io;
use std::io::BufRead;
use std::process;

mod parse;
mod token;

use parse::Word;

fn extract_command_names(s: &str) -> Option<Vec<String>> {
    let mut parser = parse::Parser::new(s);

    let mut commands = Vec::new();
    let mut is_first_word = true;

    if let Ok(words) = parser.parse() {
        for word in words {
            match word {
                Word::String(command) if is_first_word => {
                    commands.push(command);
                    is_first_word = false;
                }
                Word::And | Word::Or | Word::Pipe | Word::Terminator => {
                    is_first_word = true;
                }
                _ => continue,
            }
        }
    } else {
        return None;
    }

    Some(commands)
}

fn help() {
    print!(
        "\
Descriptions:
  Casual command line history statistics.

Usages:
  Input command history lines from stdin into `tk`.

  # bash
  fc -n -l 1 | tk

  # zsh
  history -n 1 | tk

  # fish
  history | tk

Options:
  -h, --help    Show this help message and exit.
  --version     Show version info and exit.
"
    )
}

fn version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                process::exit(0);
            }
            "--version" => {
                version();
                process::exit(0);
            }
            _ => {
                eprintln!("{}: Invalid option - {}", env!("CARGO_PKG_NAME"), arg);
                process::exit(0);
            }
        }
    }

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut total = 0;
    let mut map: HashMap<String, usize> = HashMap::new();

    let mut s = String::new();
    loop {
        let n = match handle.read_line(&mut s) {
            Ok(n) => n,
            // NOTE: Ignore non UTF-8 input.
            Err(e) if e.kind() == io::ErrorKind::InvalidData => continue,
            _ => break,
        };

        if n == 0 {
            break;
        }

        if let Some(cmds) = extract_command_names(&s) {
            for cmd in cmds {
                let counter = map.entry(cmd).or_insert(0usize);
                *counter += 1;
            }
        }

        total += 1;
        s.clear();
    }

    let mut vec: Vec<(usize, String)> = map.into_iter().map(|(k, v)| (v, k)).collect();
    vec.sort();
    vec.reverse();

    println!("       total entry number: {}", &total);
    println!("total unique entry number: {}", vec.len());

    for (v, k) in vec {
        println!("{:>10}({:.6}) {}", v, (v as f64) / (total as f64), k);
    }
}
