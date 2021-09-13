use std::collections::HashMap;
use std::env;
use std::io;
use std::io::BufRead;
use std::process;

mod parse;
mod token;

fn extract_command_name(s: &String) -> Option<String> {
    let mut parser = parse::Parser::new(s.as_str());

    let syns = parser.parse();

    match syns {
        Ok(syntaxes) => {
            for syn in syntaxes {
                if let parse::Syntax::Command(s) = syn {
                    return Some(s);
                }
            }

            None
        }
        _ => None,
    }
}

fn help() {
    print!(
        "\
Descriptions:
  Casual command line history statistics.

Usages:
  Input command history lines from stdin into `tk`.

  # bash, fish
  history | tk

  # zsh
  history -n 1 | tk

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

    let mut count = 0;
    let mut map: HashMap<String, usize> = HashMap::new();

    let mut s = String::new();
    while let Ok(n) = handle.read_line(&mut s) {
        if n == 0 {
            break;
        }

        if let Some(cmd) = extract_command_name(&s) {
            let counter = map.entry(cmd).or_insert(0usize);
            *counter += 1;
        }

        count += 1;
        s.clear();
    }

    let mut vec: Vec<(usize, String)> = map.into_iter().map(|(k, v)| (v, k)).collect();
    vec.sort();
    vec.reverse();

    for (v, k) in vec {
        println!("{:>10}({:.6}) {}", v, (v as f64) / (count as f64), k);
    }
}