use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CalcError {
    StackUnderflow {
        operator: String,
        required: usize,
        available: usize,
    },
    DivisionByZero,
    NothingToUndo,
    UnrecognizedToken(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::StackUnderflow {
                operator,
                required,
                available,
            } => {
                write!(
                    f,
                    "Error: stack underflow — '{operator}' requires {required} operands, but the stack has {available}"
                )
            }
            CalcError::DivisionByZero => {
                write!(f, "Error: division by zero")
            }
            CalcError::NothingToUndo => {
                write!(f, "Error: nothing to undo")
            }
            CalcError::UnrecognizedToken(token) => {
                write!(f, "Error: unrecognized token '{token}'")
            }
        }
    }
}

impl std::error::Error for CalcError {}
