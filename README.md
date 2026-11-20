# ⚖️ Umpire

Umpire is a hassle free multithreaded helper for competitive programming. It compiles your C++ code, runs it against your test cases, and shows you a verdict for each one.

![Demo Video](asset/demo.gif)

## Installation
Umpire runs on Linux and macOS. It calls `g++` (or the compiler in `$CXX`), so make sure a C++ compiler is installed.

### Nix
Try it without installing:
```sh
nix run github:udontur/umpire
```
Or add it to your flake. The package provides both `um` and `umpire`:
```nix
inputs.umpire.packages."${system}".default
```

### Cargo
Install [Rust](https://rustup.rs/), then run:
```sh
cargo install --git https://github.com/udontur/umpire
```
This installs the `um` command.

## Usage
See the [documentation](./docs/run.md) for every option.

Organize your code directory as follows:
```sh
task-a/
├─ main.cpp      # The file name must be "main.cpp"
└─ testcase/     # The exact name
   ├─ 1.in
   └─ 1.out
```

Judge the code in the **current** directory, or in a specific folder:
```sh
um
um task-a
```

Create a test case by pasting it into the terminal:
```sh
um add
```

Delete all test cases and start over:
```sh
um clear
```

## Development
See [docs/development.md](./docs/development.md).
