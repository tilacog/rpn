use std::process::Command;

fn run_with_args(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_pol"))
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn run_with_input(input: &str) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_pol"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
            child.wait_with_output()
        })
        .expect("failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn piped_single_expression() {
    let (stdout, _, success) = run_with_input("3 4 +\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[7]");
}

#[test]
fn piped_multi_line() {
    let (stdout, _, success) = run_with_input("3\n4\n+\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[7]");
}

#[test]
fn error_output_on_stderr() {
    let (stdout, stderr, success) = run_with_input("+\n");
    assert!(success);
    assert!(stderr.contains("stack underflow"));
    assert_eq!(stdout.trim(), "[]");
}

#[test]
fn quit_command() {
    let (stdout, _, success) = run_with_input("3 4 + quit\n");
    assert!(success);
    // In pipe mode, quit exits before printing final stack
    assert!(stdout.trim().is_empty());
}

#[test]
fn complex_expression() {
    // 15 7 1 1 + - / 3 * 2 1 1 + + -
    // Step by step:
    //   push 15, 7, 1, 1 → [15, 7, 1, 1]
    //   + → [15, 7, 2]
    //   - → [15, 5]
    //   / → [3]
    //   push 3 → [3, 3]
    //   * → [9]
    //   push 2, 1, 1 → [9, 2, 1, 1]
    //   + → [9, 2, 2]
    //   + → [9, 4]
    //   - → [5]
    let (stdout, _, success) = run_with_input("15 7 1 1 + - / 3 * 2 1 1 + + -\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[5]");
}

#[test]
fn unrecognized_token_error() {
    let (_, stderr, _) = run_with_input("3 foo +\n");
    assert!(stderr.contains("unrecognized token 'foo'"));
}

#[test]
fn division_by_zero_error() {
    let (_, stderr, _) = run_with_input("10 0 /\n");
    assert!(stderr.contains("division by zero"));
}

#[test]
fn floating_point_display() {
    let (stdout, _, success) = run_with_input("3.14\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3.14]");
}

#[test]
fn clear_command() {
    let (stdout, _, success) = run_with_input("1 2 3 clear 42\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[42]");
}

#[test]
fn undo_in_pipe_mode() {
    // 3 4 + undo → reverts the add, stack is [3, 4]
    let (stdout, _, success) = run_with_input("3 4 + undo\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[4 3]");
}

#[test]
fn undo_with_no_history_prints_error() {
    let (_, stderr, success) = run_with_input("undo\n");
    assert!(success);
    assert!(stderr.contains("nothing to undo"));
}

#[test]
fn rotate_left_in_pipe_mode() {
    let (stdout, _, success) = run_with_input("1 2 3 r\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1 3 2]");
}

#[test]
fn rotate_right_in_pipe_mode() {
    let (stdout, _, success) = run_with_input("1 2 3 r-\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[2 1 3]");
}

#[test]
fn rotate_with_count() {
    let (stdout, _, success) = run_with_input("1 2 3 r2\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[2 1 3]");
}

#[test]
fn rotate_and_undo() {
    let (stdout, _, success) = run_with_input("1 2 3 r undo\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3 2 1]");
}

#[test]
fn pop_in_pipe_mode() {
    let (stdout, _, success) = run_with_input("1 2 3 pop\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[2 1]");
}

#[test]
fn pop_on_empty_stack_error() {
    let (_, stderr, success) = run_with_input("pop\n");
    assert!(success);
    assert!(stderr.contains("stack underflow"));
}

#[test]
fn rotate_both_directions_roundtrip() {
    let (stdout, _, success) = run_with_input("1 2 3\nr\nr\nr-\nr-\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3 2 1]");
}

// Mode command integration tests (task 5.4)

#[test]
fn mode_switch_to_vertical() {
    let (stdout, _, success) = run_with_input("1 2 3\nmode vertical\n");
    assert!(success);
    assert_eq!(stdout.trim(), "3. 3\n2. 2\n1. 1");
}

#[test]
fn mode_switch_to_horizontal() {
    let (stdout, _, success) = run_with_input("1 2 3\nmode vertical\nmode horizontal\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3 2 1]");
}

#[test]
fn mode_query_default() {
    let (stdout, _, success) = run_with_input("mode\n");
    assert!(success);
    assert!(stdout.contains("horizontal"));
}

#[test]
fn mode_query_after_switch() {
    let (stdout, _, success) = run_with_input("mode vertical\nmode\n");
    assert!(success);
    assert!(stdout.contains("vertical"));
}

#[test]
fn mode_invalid_argument() {
    let (_, stderr, success) = run_with_input("mode foo\n");
    assert!(success);
    assert!(stderr.contains("invalid display mode"));
    assert!(stderr.contains("foo"));
}

// Mode does not affect stack or undo (task 5.5)

#[test]
fn mode_does_not_affect_stack() {
    let (stdout, _, success) = run_with_input("1 2\nmode vertical\nmode horizontal\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[2 1]");
}

#[test]
fn mode_does_not_affect_undo() {
    // Push 3, switch mode, undo should revert the push, not the mode change
    let (stdout, _, success) = run_with_input("1 2\n3\nmode vertical\nundo\n");
    assert!(success);
    // After undo, stack is [1, 2], vertical mode is still active
    assert_eq!(stdout.trim(), "2. 2\n1. 1");
}

// Help flag tests

#[test]
fn help_long_flag() {
    let (stdout, _, success) = run_with_args(&["--help"]);
    assert!(success);
    assert!(stdout.contains("Usage: pol"));
}

#[test]
fn help_short_flag() {
    let (stdout, _, success) = run_with_args(&["-h"]);
    assert!(success);
    assert!(stdout.contains("Usage: pol"));
}

#[test]
fn help_output_contains_expected_content() {
    let (stdout, _, _) = run_with_args(&["--help"]);
    assert!(stdout.contains("RPN"));
    assert!(stdout.contains("+"));
    assert!(stdout.contains("-"));
    assert!(stdout.contains("*"));
    assert!(stdout.contains("/"));
    assert!(stdout.contains("clear"));
    assert!(stdout.contains("pop"));
    assert!(stdout.contains("quit"));
    assert!(stdout.contains("undo"));
    assert!(stdout.contains("horizontal"));
    assert!(stdout.contains("vertical"));
}

#[test]
fn pow_basic() {
    let (stdout, _, success) = run_with_input("2 10 ^\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1024]");
}

#[test]
fn pow_zero_exponent() {
    let (stdout, _, success) = run_with_input("5 0 ^\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1]");
}

#[test]
fn mod_basic() {
    let (stdout, _, success) = run_with_input("10 3 %\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1]");
}

#[test]
fn mod_by_zero_error() {
    let (_, stderr, success) = run_with_input("10 0 %\n");
    assert!(success);
    assert!(stderr.contains("division by zero"));
}

#[test]
fn sqrt_basic() {
    let (stdout, _, success) = run_with_input("9 sqrt\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3]");
}

#[test]
fn sqrt_negative_error() {
    let (_, stderr, success) = run_with_input("-1 sqrt\n");
    assert!(success);
    assert!(stderr.contains("negative"));
}
