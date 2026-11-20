## 🛠️ Build
Umpire is a Rust project. You can use either Cargo or Nix.

### Cargo
```sh
cargo build --release   # the binary is target/release/um
cargo run -- examples/demo
```

### Nix
```sh
nix develop   # a shell with cargo, clippy, rustfmt and g++
nix build     # the binary is result/bin/um
```
> [!NOTE]
> Nix only sees files that Git knows about. Run `git add` on new files before `nix build`, or use `nix build path:.`.

## 📝 Tests
Unit tests live next to the code in `src/`. The end-to-end tests in `tests/cli.rs` run the real `um` binary on small C++ programs, so they need `g++` (or `$CXX`).
```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```
`um add` needs a terminal, so only its parts that do not read from a terminal are covered by automated tests.

`examples/demo` contains a sample task with passing and failing test cases:
```sh
cargo run -- examples/demo --output
```
