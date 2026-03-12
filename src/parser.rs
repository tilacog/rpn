use crate::error::CalcError;

#[derive(Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Clear,
    Pop,
    Quit,
    Undo,
    Rotate(i32),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Operator(Op),
    Command(Cmd),
    Mode(Option<String>),
}

pub fn parse_line(input: &str) -> Vec<Result<Token, CalcError>> {
    let mut tokens = Vec::new();
    let mut words = input.split_whitespace();
    while let Some(tok) = words.next() {
        let result = match tok {
            "+" => Ok(Token::Operator(Op::Add)),
            "-" => Ok(Token::Operator(Op::Sub)),
            "*" => Ok(Token::Operator(Op::Mul)),
            "/" => Ok(Token::Operator(Op::Div)),
            "clear" => Ok(Token::Command(Cmd::Clear)),
            "pop" => Ok(Token::Command(Cmd::Pop)),
            "quit" => Ok(Token::Command(Cmd::Quit)),
            "undo" => Ok(Token::Command(Cmd::Undo)),
            "mode" => Ok(Token::Mode(words.next().map(|s| s.to_string()))),
            other if other.starts_with('r') => {
                let rest = &other[1..];
                let n = match rest {
                    "" => Ok(1),
                    "-" => Ok(-1),
                    _ => rest.parse::<i32>(),
                };
                match n {
                    Ok(n) => Ok(Token::Command(Cmd::Rotate(n))),
                    Err(_) => Err(CalcError::UnrecognizedToken(other.to_string())),
                }
            }
            other => other
                .parse::<f64>()
                .map(Token::Number)
                .map_err(|_| CalcError::UnrecognizedToken(other.to_string())),
        };
        tokens.push(result);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(input: &str) -> Vec<Token> {
        parse_line(input).into_iter().map(|r| r.unwrap()).collect()
    }

    #[test]
    fn parse_integer() {
        assert_eq!(parse_ok("42"), vec![Token::Number(42.0)]);
    }

    #[test]
    fn parse_float() {
        assert_eq!(parse_ok("3.14"), vec![Token::Number(3.14)]);
    }

    #[test]
    fn parse_negative_number() {
        assert_eq!(parse_ok("-5"), vec![Token::Number(-5.0)]);
    }

    #[test]
    fn parse_operators() {
        assert_eq!(
            parse_ok("+ - * /"),
            vec![
                Token::Operator(Op::Add),
                Token::Operator(Op::Sub),
                Token::Operator(Op::Mul),
                Token::Operator(Op::Div),
            ]
        );
    }

    #[test]
    fn parse_commands() {
        assert_eq!(parse_ok("clear"), vec![Token::Command(Cmd::Clear)]);
        assert_eq!(parse_ok("quit"), vec![Token::Command(Cmd::Quit)]);
    }

    #[test]
    fn parse_unrecognized_token() {
        let result = parse_line("foo");
        assert_eq!(
            result,
            vec![Err(CalcError::UnrecognizedToken("foo".to_string()))]
        );
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse_line("").is_empty());
        assert!(parse_line("   ").is_empty());
    }

    #[test]
    fn parse_mixed_expression() {
        assert_eq!(
            parse_ok("3 4 +"),
            vec![
                Token::Number(3.0),
                Token::Number(4.0),
                Token::Operator(Op::Add)
            ]
        );
    }

    #[test]
    fn parse_extra_whitespace() {
        assert_eq!(
            parse_ok("  3   4   +  "),
            vec![
                Token::Number(3.0),
                Token::Number(4.0),
                Token::Operator(Op::Add)
            ]
        );
    }

    #[test]
    fn parse_pop() {
        assert_eq!(parse_ok("pop"), vec![Token::Command(Cmd::Pop)]);
    }

    #[test]
    fn parse_undo() {
        assert_eq!(parse_ok("undo"), vec![Token::Command(Cmd::Undo)]);
    }

    #[test]
    fn parse_undo_in_mixed_expression() {
        assert_eq!(
            parse_ok("3 4 + undo"),
            vec![
                Token::Number(3.0),
                Token::Number(4.0),
                Token::Operator(Op::Add),
                Token::Command(Cmd::Undo),
            ]
        );
    }

    #[test]
    fn parse_rotate_default() {
        assert_eq!(parse_ok("r"), vec![Token::Command(Cmd::Rotate(1))]);
    }

    #[test]
    fn parse_rotate_explicit_1() {
        assert_eq!(parse_ok("r1"), vec![Token::Command(Cmd::Rotate(1))]);
    }

    #[test]
    fn parse_rotate_count() {
        assert_eq!(parse_ok("r2"), vec![Token::Command(Cmd::Rotate(2))]);
    }

    #[test]
    fn parse_rotate_negative_default() {
        assert_eq!(parse_ok("r-"), vec![Token::Command(Cmd::Rotate(-1))]);
    }

    #[test]
    fn parse_rotate_negative_explicit_1() {
        assert_eq!(parse_ok("r-1"), vec![Token::Command(Cmd::Rotate(-1))]);
    }

    #[test]
    fn parse_rotate_negative_count() {
        assert_eq!(parse_ok("r-2"), vec![Token::Command(Cmd::Rotate(-2))]);
    }

    #[test]
    fn parse_rotate_zero() {
        assert_eq!(parse_ok("r0"), vec![Token::Command(Cmd::Rotate(0))]);
    }

    #[test]
    fn parse_rotate_invalid_suffix() {
        let result = parse_line("rx");
        assert_eq!(
            result,
            vec![Err(CalcError::UnrecognizedToken("rx".to_string()))]
        );
    }

    #[test]
    fn parse_rotate_invalid_mixed_suffix() {
        let result = parse_line("r1x");
        assert_eq!(
            result,
            vec![Err(CalcError::UnrecognizedToken("r1x".to_string()))]
        );
    }

    #[test]
    fn parse_mode_bare() {
        assert_eq!(parse_ok("mode"), vec![Token::Mode(None)]);
    }

    #[test]
    fn parse_mode_horizontal() {
        assert_eq!(
            parse_ok("mode horizontal"),
            vec![Token::Mode(Some("horizontal".to_string()))]
        );
    }

    #[test]
    fn parse_mode_vertical() {
        assert_eq!(
            parse_ok("mode vertical"),
            vec![Token::Mode(Some("vertical".to_string()))]
        );
    }

    #[test]
    fn parse_mode_invalid() {
        assert_eq!(
            parse_ok("mode foo"),
            vec![Token::Mode(Some("foo".to_string()))]
        );
    }
}
