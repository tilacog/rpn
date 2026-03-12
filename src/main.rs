use std::io::{self, BufRead, IsTerminal, Write};

use rpn::engine::Calculator;
use rpn::error::CalcError;
use rpn::parser::{self, Token};

#[derive(Clone, Copy)]
enum DisplayMode {
    Horizontal,
    Vertical,
}

fn format_value(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn format_stack_horizontal(stack: &[f64]) -> String {
    let values: Vec<String> = stack.iter().rev().map(|&v| format_value(v)).collect();
    format!("[{}]", values.join(" "))
}

fn format_stack_vertical(stack: &[f64]) -> String {
    stack
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &v)| format!("{}. {}", i, format_value(v)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_stack(stack: &[f64], mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Horizontal => format_stack_horizontal(stack),
        DisplayMode::Vertical => format_stack_vertical(stack),
    }
}

fn handle_mode(arg: Option<String>, display_mode: &mut DisplayMode) {
    match arg.as_deref() {
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
    for token_result in tokens {
        let token = token_result?;
        match token {
            Token::Mode(arg) => handle_mode(arg, display_mode),
            token => {
                if calc.process_token(token)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn main() {
    let stdin = io::stdin();
    let is_tty = stdin.is_terminal();
    let mut calc = Calculator::new();
    let mut display_mode = DisplayMode::Horizontal;

    if is_tty {
        // REPL mode
        let mut stdout = io::stdout();
        loop {
            print!("> ");
            stdout.flush().unwrap();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }

            match process_line(&line, &mut calc, &mut display_mode) {
                Ok(true) => break, // quit
                Ok(false) => println!("{}", format_stack(calc.stack(), display_mode)),
                Err(e) => eprintln!("{e}"),
            }
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
        assert_eq!(format_stack_horizontal(&[1.0, 2.0, 3.0]), "[3 2 1]");
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
        assert_eq!(format_stack_horizontal(&[3.14, 2.0]), "[2 3.14]");
    }

    // format_stack_vertical tests (task 5.2)

    #[test]
    fn vertical_multi_element() {
        assert_eq!(
            format_stack_vertical(&[1.0, 2.0, 3.0]),
            "0. 3\n1. 2\n2. 1"
        );
    }

    #[test]
    fn vertical_single() {
        assert_eq!(format_stack_vertical(&[42.0]), "0. 42");
    }

    #[test]
    fn vertical_empty() {
        assert_eq!(format_stack_vertical(&[]), "");
    }

    #[test]
    fn vertical_float() {
        assert_eq!(format_stack_vertical(&[3.14, 2.0]), "0. 2\n1. 3.14");
    }
}
