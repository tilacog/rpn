use std::io::{self, BufRead, IsTerminal, Write};

use rpn::engine::Calculator;

fn format_stack(stack: &[f64]) -> String {
    let values: Vec<String> = stack
        .iter()
        .map(|&v| {
            if v.fract() == 0.0 && v.is_finite() {
                format!("{}", v as i64)
            } else {
                format!("{v}")
            }
        })
        .collect();
    format!("[{}]", values.join(" "))
}

fn main() {
    let stdin = io::stdin();
    let is_tty = stdin.is_terminal();
    let mut calc = Calculator::new();

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

            match calc.process_line(&line) {
                Ok(true) => break, // quit
                Ok(false) => println!("{}", format_stack(calc.stack())),
                Err(e) => eprintln!("{e}"),
            }
        }
    } else {
        // Pipe mode
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => match calc.process_line(&line) {
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
        println!("{}", format_stack(calc.stack()));
    }
}
