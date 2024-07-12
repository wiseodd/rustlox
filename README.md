# Yet Another Lox Interpreter

Another Rust implementation of Lox, the language introduced in
Crafting Interpreters by Robert Nystrom.

> [!TIP]
> **What makes this unique:** Nothing! This Lox interpreter is just
> a vessel I use to learn Rust. Expect non-idiomatic and unsafe Rust here and there!
> Nevertheless, this might be useful to people who also follows the book with
> Rust.

> [!IMPORTANT]
> Unlike other Rust implementations out there, I don't use the Visitor pattern
> used in the book. It's all full on pattern matching.

## Setup

1. Install Rust
2. **REPL:** `cargo run`
3. **Executing `.lox` script:** `cargo run -- your_script.lox`

> [!TIP]
> Check out `playground` dir. It contains some example Lox scripts.
> Run via `cargo run -- playground/{script_name}.lox`

## References

1. Nystrom, Robert. _Crafting interpreters_. Genever Benning, 2021. [[Link]](https://craftinginterpreters.com/)
