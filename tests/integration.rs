use std::process::Command;

fn run_with_input(input: &str) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
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
fn complex_rpn_expression() {
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
    assert_eq!(stdout.trim(), "[3 4]");
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
    assert_eq!(stdout.trim(), "[2 3 1]");
}

#[test]
fn rotate_right_in_pipe_mode() {
    let (stdout, _, success) = run_with_input("1 2 3 r-\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3 1 2]");
}

#[test]
fn rotate_with_count() {
    let (stdout, _, success) = run_with_input("1 2 3 r2\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[3 1 2]");
}

#[test]
fn rotate_and_undo() {
    let (stdout, _, success) = run_with_input("1 2 3 r undo\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1 2 3]");
}

#[test]
fn rotate_both_directions_roundtrip() {
    let (stdout, _, success) = run_with_input("1 2 3\nr\nr\nr-\nr-\n");
    assert!(success);
    assert_eq!(stdout.trim(), "[1 2 3]");
}
