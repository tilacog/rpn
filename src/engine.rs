use crate::error::CalcError;
use crate::parser::{self, Cmd, Op, Token};

#[must_use]
pub fn get_help_text() -> String {
    "\
Usage: pol [OPTIONS]

An interactive RPN (Reverse Polish Notation) calculator.

Reads expressions in postfix notation. Runs as an interactive REPL when
started in a terminal, or processes piped input in batch mode.

Operators:
    +    Addition
    -    Subtraction
    *    Multiplication
    /    Division
    ^    Exponentiation (a ^ b = a raised to the power b)
    %    Modulo (remainder after division)

Commands:
    clear       Clear the stack
    help        Show this help text
    pop         Remove the top element
    quit        Exit the calculator
    undo        Undo the last operation
    r, r<N>     Rotate stack left by N (default 1)
    r-, r-<N>   Rotate stack right by N (default 1)
    sqrt        Square root of the top element

Display modes:
    mode                Show current display mode
    mode horizontal     Stack on one line (default): [1 2 3]
    mode vertical       Stack with indices:
                            3. 3
                            2. 2
                            1. 1
"
    .to_string()
}

pub struct Calculator {
    stack: Vec<f64>,
    history: Vec<Vec<f64>>,
}

impl Calculator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            history: Vec::new(),
        }
    }

    fn save_snapshot(&mut self) {
        self.history.push(self.stack.clone());
    }

    pub fn push(&mut self, value: f64) {
        self.save_snapshot();
        self.stack.push(value);
    }

    #[must_use]
    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

    /// Removes the top element from the stack.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::StackUnderflow`] if the stack is empty.
    pub fn pop(&mut self) -> Result<(), CalcError> {
        if self.stack.is_empty() {
            return Err(CalcError::StackUnderflow {
                operator: "pop".to_string(),
                required: 1,
                available: 0,
            });
        }
        self.save_snapshot();
        self.stack.pop();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.save_snapshot();
        self.stack.clear();
    }

    pub fn rotate(&mut self, n: i32) {
        let len = self.stack.len();
        if len <= 1 {
            return;
        }
        let len_i32 = i32::try_from(len).unwrap_or(i32::MAX);
        let n = usize::try_from(n.rem_euclid(len_i32)).unwrap_or(0);
        if n == 0 {
            return;
        }
        self.save_snapshot();
        self.stack.rotate_left(n);
    }

    /// Reverts the last operation by restoring the previous stack state.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::NothingToUndo`] if there is no history to revert.
    pub fn undo(&mut self) -> Result<(), CalcError> {
        match self.history.pop() {
            Some(previous) => {
                self.stack = previous;
                Ok(())
            }
            None => Err(CalcError::NothingToUndo),
        }
    }

    /// Replaces the top element with its square root.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::StackUnderflow`] if the stack is empty, or
    /// [`CalcError::NegativeSqrt`] if the top value is negative.
    pub fn sqrt(&mut self) -> Result<(), CalcError> {
        let Some(&a) = self.stack.last() else {
            return Err(CalcError::StackUnderflow {
                operator: "sqrt".to_string(),
                required: 1,
                available: 0,
            });
        };
        if a < 0.0 {
            return Err(CalcError::NegativeSqrt);
        }
        self.save_snapshot();
        self.stack.pop();
        self.stack.push(a.sqrt());
        Ok(())
    }

    /// Applies a binary operator to the top two stack elements.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::StackUnderflow`] if fewer than two elements are on
    /// the stack, or [`CalcError::DivisionByZero`] for `/` or `%` when the
    /// divisor is zero.
    pub fn apply_operator(&mut self, op: &Op) -> Result<(), CalcError> {
        let op_str = match op {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Pow => "^",
            Op::Mod => "%",
        };

        if self.stack.len() < 2 {
            return Err(CalcError::StackUnderflow {
                operator: op_str.to_string(),
                required: 2,
                available: self.stack.len(),
            });
        }

        let b = self.stack[self.stack.len() - 1];
        let a = self.stack[self.stack.len() - 2];

        if matches!(op, Op::Div | Op::Mod) && b == 0.0 {
            return Err(CalcError::DivisionByZero);
        }

        // Only pop after validation passes
        self.save_snapshot();
        self.stack.pop();
        self.stack.pop();

        let result = match op {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
            Op::Pow => a.powf(b),
            Op::Mod => a % b,
        };

        self.stack.push(result);
        Ok(())
    }

    /// Process a single token. Returns `Ok(true)` if quit was requested.
    /// Mode tokens are ignored by the engine (handled by the caller).
    ///
    /// # Errors
    ///
    /// Returns a [`CalcError`] if the token triggers a stack or arithmetic error.
    pub fn process_token(&mut self, token: Token) -> Result<bool, CalcError> {
        match token {
            Token::Number(n) => self.push(n),
            Token::Operator(op) => self.apply_operator(&op)?,
            Token::Command(Cmd::Clear) => self.clear(),
            Token::Command(Cmd::Help) | Token::Mode(_) => {} // handled by main
            Token::Command(Cmd::Pop) => self.pop()?,
            Token::Command(Cmd::Quit) => return Ok(true),
            Token::Command(Cmd::Undo) => self.undo()?,
            Token::Command(Cmd::Rotate(n)) => self.rotate(n),
            Token::Command(Cmd::Sqrt) => self.sqrt()?,
        }
        Ok(false)
    }

    /// Process a line of input. Returns `Ok(true)` if quit was requested.
    ///
    /// # Errors
    ///
    /// Returns the first [`CalcError`] encountered while processing tokens.
    pub fn process_line(&mut self, line: &str) -> Result<bool, CalcError> {
        let tokens = parser::parse_line(line);
        for token_result in tokens {
            let token = token_result?;
            if self.process_token(token)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_inspect() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        assert_eq!(calc.stack(), &[3.0, 4.0]);
    }

    #[test]
    fn addition() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        calc.apply_operator(&Op::Add).unwrap();
        assert_eq!(calc.stack(), &[7.0]);
    }

    #[test]
    fn subtraction() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Sub).unwrap();
        assert_eq!(calc.stack(), &[7.0]);
    }

    #[test]
    fn subtraction_negative_result() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(10.0);
        calc.apply_operator(&Op::Sub).unwrap();
        assert_eq!(calc.stack(), &[-7.0]);
    }

    #[test]
    fn multiplication() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        calc.apply_operator(&Op::Mul).unwrap();
        assert_eq!(calc.stack(), &[12.0]);
    }

    #[test]
    fn division() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(2.0);
        calc.apply_operator(&Op::Div).unwrap();
        assert_eq!(calc.stack(), &[5.0]);
    }

    #[test]
    fn division_by_zero() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(0.0);
        let err = calc.apply_operator(&Op::Div).unwrap_err();
        assert_eq!(err, CalcError::DivisionByZero);
        assert_eq!(calc.stack(), &[10.0, 0.0]);
    }

    #[test]
    fn stack_underflow_empty() {
        let mut calc = Calculator::new();
        let err = calc.apply_operator(&Op::Mul).unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "*".to_string(),
                required: 2,
                available: 0,
            }
        );
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn stack_underflow_one_element() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        let err = calc.apply_operator(&Op::Add).unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "+".to_string(),
                required: 2,
                available: 1,
            }
        );
        assert_eq!(calc.stack(), &[5.0]);
    }

    #[test]
    fn clear_stack() {
        let mut calc = Calculator::new();
        calc.push(1.0);
        calc.push(2.0);
        calc.clear();
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn process_multi_token_expression() {
        let mut calc = Calculator::new();
        calc.process_line("3 4 +").unwrap();
        assert_eq!(calc.stack(), &[7.0]);
    }

    #[test]
    fn stack_persists_across_lines() {
        let mut calc = Calculator::new();
        calc.process_line("3").unwrap();
        calc.process_line("4 +").unwrap();
        assert_eq!(calc.stack(), &[7.0]);
    }

    #[test]
    fn operator_preserves_rest_of_stack() {
        let mut calc = Calculator::new();
        calc.push(1.0);
        calc.push(2.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Add).unwrap();
        assert_eq!(calc.stack(), &[1.0, 5.0]);
    }

    #[test]
    fn process_line_quit() {
        let mut calc = Calculator::new();
        let quit = calc.process_line("quit").unwrap();
        assert!(quit);
    }

    #[test]
    fn process_line_clear() {
        let mut calc = Calculator::new();
        calc.push(1.0);
        calc.push(2.0);
        calc.process_line("clear").unwrap();
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn undo_reverts_push() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[3.0]);
    }

    #[test]
    fn undo_reverts_operator() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        calc.apply_operator(&Op::Add).unwrap();
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[3.0, 4.0]);
    }

    #[test]
    fn undo_reverts_clear() {
        let mut calc = Calculator::new();
        calc.push(3.0);
        calc.push(4.0);
        calc.clear();
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[3.0, 4.0]);
    }

    #[test]
    fn multiple_consecutive_undos() {
        let mut calc = Calculator::new();
        calc.process_line("3 4 +").unwrap();
        calc.undo().unwrap(); // undo add
        calc.undo().unwrap(); // undo push 4
        assert_eq!(calc.stack(), &[3.0]);
    }

    #[test]
    fn undo_all_the_way_to_empty() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        calc.undo().unwrap();
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn undo_on_fresh_calculator() {
        let mut calc = Calculator::new();
        let err = calc.undo().unwrap_err();
        assert_eq!(err, CalcError::NothingToUndo);
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn undo_after_exhausting_history() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        calc.undo().unwrap();
        let err = calc.undo().unwrap_err();
        assert_eq!(err, CalcError::NothingToUndo);
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn undo_does_not_create_history_entry() {
        let mut calc = Calculator::new();
        calc.push(3.0); // history: [[] ]
        calc.push(4.0); // history: [[], [3]]
        calc.apply_operator(&Op::Add).unwrap(); // history: [[], [3], [3, 4]]
        calc.undo().unwrap(); // history: [[], [3]] — undo does NOT add entry
        // After undoing the add, we have history for push(3) and push(4)
        // but undo consumed one, so history is [[], [3]]
        calc.undo().unwrap(); // undo push 4 → stack [3], history: [[]]
        calc.undo().unwrap(); // undo push 3 → stack [], history: []
        let err = calc.undo().unwrap_err();
        assert_eq!(err, CalcError::NothingToUndo);
    }

    #[test]
    fn pop_from_stack_with_elements() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.pop().unwrap();
        assert_eq!(calc.stack(), &[1.0, 2.0]);
    }

    #[test]
    fn pop_from_single_element_stack() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        calc.pop().unwrap();
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn pop_from_empty_stack() {
        let mut calc = Calculator::new();
        let err = calc.pop().unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "pop".to_string(),
                required: 1,
                available: 0,
            }
        );
    }

    #[test]
    fn undo_reverts_pop() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.pop().unwrap();
        assert_eq!(calc.stack(), &[1.0, 2.0]);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn rotate_left_by_1() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(1);
        assert_eq!(calc.stack(), &[2.0, 3.0, 1.0]);
    }

    #[test]
    fn rotate_left_by_2() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(2);
        assert_eq!(calc.stack(), &[3.0, 1.0, 2.0]);
    }

    #[test]
    fn rotate_right_by_1() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(-1);
        assert_eq!(calc.stack(), &[3.0, 1.0, 2.0]);
    }

    #[test]
    fn rotate_right_by_2() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(-2);
        assert_eq!(calc.stack(), &[2.0, 3.0, 1.0]);
    }

    #[test]
    fn rotate_full_cycle() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(1);
        calc.rotate(1);
        calc.rotate(1);
        assert_eq!(calc.stack(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn rotate_empty_stack() {
        let mut calc = Calculator::new();
        calc.rotate(1);
        assert!(calc.stack().is_empty());
    }

    #[test]
    fn rotate_single_element() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        calc.rotate(1);
        assert_eq!(calc.stack(), &[5.0]);
    }

    #[test]
    fn rotate_zero() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(0);
        assert_eq!(calc.stack(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn rotate_mod_wrapping() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(4); // 4 mod 3 = 1
        assert_eq!(calc.stack(), &[2.0, 3.0, 1.0]);
    }

    #[test]
    fn rotate_negative_mod_wrapping() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(-4); // -4 rem_euclid 3 = 2
        assert_eq!(calc.stack(), &[3.0, 1.0, 2.0]);
    }

    #[test]
    fn undo_reverts_rotate() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3").unwrap();
        calc.rotate(1);
        assert_eq!(calc.stack(), &[2.0, 3.0, 1.0]);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn rotate_mid_expression() {
        let mut calc = Calculator::new();
        calc.process_line("1 2 3 r 4 +").unwrap();
        // [1,2,3] → rotate → [2,3,1] → push 4 → [2,3,1,4] → add → [2,3,5]
        assert_eq!(calc.stack(), &[2.0, 3.0, 5.0]);
    }

    // pow tests
    #[test]
    fn pow_basic() {
        let mut calc = Calculator::new();
        calc.push(2.0);
        calc.push(10.0);
        calc.apply_operator(&Op::Pow).unwrap();
        assert_eq!(calc.stack(), &[1024.0]);
    }

    #[test]
    fn pow_zero_exponent() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        calc.push(0.0);
        calc.apply_operator(&Op::Pow).unwrap();
        assert_eq!(calc.stack(), &[1.0]);
    }

    #[test]
    fn pow_stack_underflow() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        let err = calc.apply_operator(&Op::Pow).unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "^".to_string(),
                required: 2,
                available: 1,
            }
        );
    }

    #[test]
    fn pow_undo() {
        let mut calc = Calculator::new();
        calc.push(2.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Pow).unwrap();
        assert_eq!(calc.stack(), &[8.0]);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[2.0, 3.0]);
    }

    // modulo tests
    #[test]
    fn mod_basic() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Mod).unwrap();
        assert_eq!(calc.stack(), &[1.0]);
    }

    #[test]
    fn mod_no_remainder() {
        let mut calc = Calculator::new();
        calc.push(9.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Mod).unwrap();
        assert_eq!(calc.stack(), &[0.0]);
    }

    #[test]
    fn mod_floating_point() {
        let mut calc = Calculator::new();
        calc.push(5.5);
        calc.push(2.0);
        calc.apply_operator(&Op::Mod).unwrap();
        assert_eq!(calc.stack(), &[1.5]);
    }

    #[test]
    fn mod_by_zero() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(0.0);
        let err = calc.apply_operator(&Op::Mod).unwrap_err();
        assert_eq!(err, CalcError::DivisionByZero);
        assert_eq!(calc.stack(), &[10.0, 0.0]);
    }

    #[test]
    fn mod_stack_underflow() {
        let mut calc = Calculator::new();
        calc.push(5.0);
        let err = calc.apply_operator(&Op::Mod).unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "%".to_string(),
                required: 2,
                available: 1,
            }
        );
    }

    #[test]
    fn mod_undo() {
        let mut calc = Calculator::new();
        calc.push(10.0);
        calc.push(3.0);
        calc.apply_operator(&Op::Mod).unwrap();
        assert_eq!(calc.stack(), &[1.0]);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[10.0, 3.0]);
    }

    // sqrt tests
    #[test]
    fn sqrt_perfect_square() {
        let mut calc = Calculator::new();
        calc.push(9.0);
        calc.sqrt().unwrap();
        assert_eq!(calc.stack(), &[3.0]);
    }

    #[test]
    fn sqrt_non_perfect_square() {
        let mut calc = Calculator::new();
        calc.push(2.0);
        calc.sqrt().unwrap();
        let result = calc.stack()[0];
        assert!((result - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn sqrt_zero() {
        let mut calc = Calculator::new();
        calc.push(0.0);
        calc.sqrt().unwrap();
        assert_eq!(calc.stack(), &[0.0]);
    }

    #[test]
    fn sqrt_negative_input() {
        let mut calc = Calculator::new();
        calc.push(-1.0);
        let err = calc.sqrt().unwrap_err();
        assert_eq!(err, CalcError::NegativeSqrt);
        assert_eq!(calc.stack(), &[-1.0]);
    }

    #[test]
    fn sqrt_empty_stack() {
        let mut calc = Calculator::new();
        let err = calc.sqrt().unwrap_err();
        assert_eq!(
            err,
            CalcError::StackUnderflow {
                operator: "sqrt".to_string(),
                required: 1,
                available: 0,
            }
        );
    }

    #[test]
    fn sqrt_undo() {
        let mut calc = Calculator::new();
        calc.push(16.0);
        calc.sqrt().unwrap();
        assert_eq!(calc.stack(), &[4.0]);
        calc.undo().unwrap();
        assert_eq!(calc.stack(), &[16.0]);
    }

    #[test]
    fn help_does_not_modify_stack() {
        let mut calc = Calculator::new();
        calc.push(1.0);
        calc.push(2.0);
        let quit = calc.process_token(Token::Command(Cmd::Help)).unwrap();
        assert!(!quit);
        assert_eq!(calc.stack(), &[1.0, 2.0]);
    }

    #[test]
    fn help_on_empty_stack() {
        let mut calc = Calculator::new();
        let quit = calc.process_token(Token::Command(Cmd::Help)).unwrap();
        assert!(!quit);
        assert!(calc.stack().is_empty());
    }
}
