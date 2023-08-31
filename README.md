# Monkey Programming Language Interpreter in Rust

![Monkey Interpreter](https://interpreterbook.com/img/monkey_logo-d5171d15.png)

This repository contains an implementation of the Monkey programming language interpreter in Rust. The Monkey programming language is a simple, dynamically-typed language, and this project is based on the concepts presented in the book "Writing An Interpreter In Go" by Thorsten Ball. The goal of this project is to provide a functional interpreter for the Monkey language using the Rust programming language.

## Features

- Lexical analysis (Tokenization)
- Syntax analysis (Parsing)
- Abstract Syntax Tree (AST) representation
- Evaluation of expressions and statements
- Basic built-in functions
- REPL (Read-Eval-Print Loop) for interactive usage

## Getting Started

Follow these instructions to get a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

- Rust programming language (https://www.rust-lang.org/tools/install)
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:

```bash
git clone https://github.com/ElZafro/interpreter-book.git
cd interpreter-book
```

2. Build the project using Cargo:

```bash
cargo build
```

3. Run the tests to ensure everything is working correctly:

```bash
cargo test
```

### Usage

Right now there is no way to run a preexisting script, you have to use the REPL.

#### Running Scripts

*In development*

#### Using the REPL

To use the REPL for interactive experimentation, simply run:

```bash
cargo run --release
```

This will start the Monkey interpreter in REPL mode, allowing you to enter and evaluate expressions and statements interactively.

## Acknowledgments

- The Monkey programming language and the ideas behind this project are based on the book "Writing An Interpreter In Go" by Thorsten Ball.