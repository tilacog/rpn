use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CalcError {
    StackUnderflow {
        operator: String,
        required: usize,
        available: usize,
    },
    DivisionByZero,
    NegativeSqrt,
    NothingToUndo,
    UnrecognizedToken(String),
    InvalidDisplayMode(String),
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
            CalcError::NegativeSqrt => {
                write!(f, "Error: square root of negative number")
            }
            CalcError::NothingToUndo => {
                write!(f, "Error: nothing to undo")
            }
            CalcError::UnrecognizedToken(token) => {
                write!(f, "Error: unrecognized token '{token}'")
            }
            CalcError::InvalidDisplayMode(mode) => {
                write!(
                    f,
                    "Error: invalid display mode '{mode}' (valid modes: horizontal, vertical)"
                )
            }
        }
    }
}

impl std::error::Error for CalcError {}
