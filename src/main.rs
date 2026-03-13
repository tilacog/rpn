#![warn(clippy::pedantic)]

use std::io::{self, BufRead, IsTerminal};

use clap::{Parser, ValueEnum};
use pol::engine::{Calculator, get_help_text};
use pol::error::CalcError;
use pol::parser::{self, Cmd, Token};
use rustyline::DefaultEditor;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;

#[derive(Clone, Copy, ValueEnum)]
enum DisplayMode {
    Horizontal,
    Vertical,
}

#[derive(Parser)]
#[command(about, version)]
struct Args {
    /// Stack display mode
    #[arg(long, value_enum, default_value_t = DisplayMode::Vertical)]
    mode: DisplayMode,
}

fn format_value(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

fn format_stack_horizontal(stack: &[f64]) -> String {
    let values: Vec<String> = stack.iter().map(|&v| format_value(v)).collect();
    format!("[{}]", values.join(" "))
}

fn format_stack_vertical(stack: &[f64]) -> String {
    stack
        .iter()
        .enumerate()
        .map(|(i, &v)| format!("{}: {}", stack.len() - i, format_value(v)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_stack(stack: &[f64], mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Horizontal => format_stack_horizontal(stack),
        DisplayMode::Vertical => format_stack_vertical(stack),
    }
}

fn handle_mode(arg: Option<&str>, display_mode: &mut DisplayMode) {
    match arg {
        None => {
            let name = match display_mode {
                DisplayMode::Horizontal => "horizontal",
                DisplayMode::Vertical => "vertical",
            };
            println!("{name}");
        }
        Some("horizontal") => *display_mode = DisplayMode::Horizontal,
        Some("vertical") => *display_mode = DisplayMode::Vertical,
        Some(invalid) => {
            eprintln!("{}", CalcError::InvalidDisplayMode(invalid.to_string()));
        }
    }
}

/// Process a line, intercepting Mode tokens before the engine.
/// Returns true if quit was requested.
fn process_line(
    line: &str,
    calc: &mut Calculator,
    display_mode: &mut DisplayMode,
) -> Result<bool, CalcError> {
    let tokens = parser::parse_line(line);
    let mut print_help_after = false;
    for token_result in tokens {
        let token = token_result?;
        match token {
            Token::Mode(arg) => handle_mode(arg.as_deref(), display_mode),
            Token::Command(Cmd::Help) => print_help_after = true,
            token => {
                if calc.process_token(token)? {
                    return Ok(true);
                }
            }
        }
    }
    if print_help_after {
        print_help();
    }
    Ok(false)
}

fn print_help() {
    print!("{}", get_help_text());
}

fn main() {
    let args = Args::parse();

    let stdin = io::stdin();
    let is_tty = stdin.is_terminal();
    let mut calc = Calculator::new();
    let mut display_mode = args.mode;

    if is_tty {
        // REPL mode
        let history_path = home::home_dir().map(|h| h.join(".pol_history"));
        let mut rl = DefaultEditor::new().expect("failed to initialize editor");
        let _ = rl.set_max_history_size(1000);
        if let Some(path) = &history_path {
            let _ = rl.load_history(path);
        }

        loop {
            match rl.readline("> ") {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);
                    match process_line(&line, &mut calc, &mut display_mode) {
                        Ok(true) => break, // quit
                        Ok(false) => println!("{}", format_stack(calc.stack(), display_mode)),
                        Err(e) => eprintln!("{e}"),
                    }
                }
                Err(ReadlineError::Interrupted) => {}
                Err(ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }
        }

        if let Some(path) = &history_path {
            let _ = rl.save_history(path);
        }
    } else {
        // Pipe mode
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => match process_line(&line, &mut calc, &mut display_mode) {
                    Ok(true) => return,
                    Ok(false) => {}
                    Err(e) => eprintln!("{e}"),
                },
                Err(e) => {
                    eprintln!("Error: {e}");
                    return;
                }
            }
        }
        println!("{}", format_stack(calc.stack(), display_mode));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // format_stack_horizontal tests (task 5.1)

    #[test]
    fn horizontal_multi_element() {
        assert_eq!(format_stack_horizontal(&[1.0, 2.0, 3.0]), "[1 2 3]");
    }

    #[test]
    fn horizontal_single() {
        assert_eq!(format_stack_horizontal(&[42.0]), "[42]");
    }

    #[test]
    fn horizontal_empty() {
        assert_eq!(format_stack_horizontal(&[]), "[]");
    }

    #[test]
    fn horizontal_float() {
        assert_eq!(format_stack_horizontal(&[1.5, 2.0]), "[1.5 2]");
    }

    // format_stack_vertical tests (task 5.2)

    #[test]
    fn vertical_multi_element() {
        assert_eq!(format_stack_vertical(&[1.0, 2.0, 3.0]), "3: 1\n2: 2\n1: 3");
    }

    #[test]
    fn vertical_single() {
        assert_eq!(format_stack_vertical(&[42.0]), "1: 42");
    }

    #[test]
    fn vertical_empty() {
        assert_eq!(format_stack_vertical(&[]), "");
    }

    #[test]
    fn vertical_float() {
        assert_eq!(format_stack_vertical(&[1.5, 2.0]), "2: 1.5\n1: 2");
    }
}
