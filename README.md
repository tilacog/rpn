# rpn

An RPN calculator in Rust with a REPL and pipe mode. The stack persists across lines.

## Install

```bash
cargo install --path .
```

## Usage

### REPL mode

In a terminal, you get a prompt and the stack after each line:

```
> 3 4 +
[7]
> 10 *
[70]
> 2 /
[35]
```

### Pipe mode

Piped input runs silently and prints the final stack:

```bash
echo "3 4 +" | rpn
# [7]

printf "15 7 1 1 + - /\n3 * 2 1 1 + + -\n" | rpn
# [5]
```

## Tokens

| Token | Description |
|-------|-------------|
| Numbers | Integers and floats (e.g. `42`, `3.14`, `-5`) |
| `+` `-` `*` `/` | Arithmetic operators (binary, pop two, push result) |
| `clear` | Clear the stack |
| `pop` | Remove the top element |
| `undo` | Revert the last operation |
| `r`, `r2`, `r-`, `r-2` | Rotate the stack left/right by N positions (default 1) |
| `mode` | Show current display mode |
| `mode horizontal` | Switch to horizontal display (default, top-first: `[3 2 1]`) |
| `mode vertical` | Switch to vertical display with indices (`0. 3`, `1. 2`, `2. 1`) |
| `quit` | Exit the calculator |

## Build and test

```bash
cargo build
cargo test
```

## License

GPL-3.0
