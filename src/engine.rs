use crate::error::CalcError;
use crate::parser::{self, Cmd, Op, Token};

pub struct Calculator {
    stack: Vec<f64>,
    history: Vec<Vec<f64>>,
}

impl Calculator {
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

    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

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
        let n = n.rem_euclid(len as i32) as usize;
        if n == 0 {
            return;
        }
        self.save_snapshot();
        self.stack.rotate_left(n);
    }

    pub fn undo(&mut self) -> Result<(), CalcError> {
        match self.history.pop() {
            Some(previous) => {
                self.stack = previous;
                Ok(())
            }
            None => Err(CalcError::NothingToUndo),
        }
    }

    pub fn apply_operator(&mut self, op: &Op) -> Result<(), CalcError> {
        let op_str = match op {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
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

        if matches!(op, Op::Div) && b == 0.0 {
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
        };

        self.stack.push(result);
        Ok(())
    }

    /// Process a single token. Returns `Ok(true)` if quit was requested.
    /// Mode tokens are ignored by the engine (handled by the caller).
    pub fn process_token(&mut self, token: Token) -> Result<bool, CalcError> {
        match token {
            Token::Number(n) => self.push(n),
            Token::Operator(op) => self.apply_operator(&op)?,
            Token::Command(Cmd::Clear) => self.clear(),
            Token::Command(Cmd::Pop) => self.pop()?,
            Token::Command(Cmd::Quit) => return Ok(true),
            Token::Command(Cmd::Undo) => self.undo()?,
            Token::Command(Cmd::Rotate(n)) => self.rotate(n),
            Token::Mode(_) => {}
        }
        Ok(false)
    }

    /// Process a line of input. Returns `Ok(true)` if quit was requested.
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
}
