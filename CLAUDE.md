# typed-lucide

The README is the spec: if the code and the README disagree, the code is wrong.

## The generated files

`src/icon.rs` (the `Icon` enum, names, bodies) and `src/meta.rs` (tags,
categories, the `Category` enum, the search data) are written by the generator
and carry a `// GENERATED …` first line. **Never edit them.** Everything
hand-written lives in `src/lib.rs`, `src/search.rs` and `src/bin/icon-search.rs`.

## Regenerating

```bash
cargo run -p generate -- 1.40.0                       # downloads the tag from GitHub
cargo run -p generate -- 1.40.0 --from-dir <icons>    # a local lucide icons/ directory
```

It rewrites both generated files, sets `version` in the root `Cargo.toml` to the
tag, and runs `rustfmt` over what it wrote. Output is sorted by icon name, so a
second run on the same tag leaves `git diff` empty.

**The crate version is the Lucide version.** One crate release per Lucide
release; nothing else moves it.

## Tests

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo fmt --all --check
cargo package
```

All four must exit 0. `cargo package` proves the crate stands without the
generator and without a network.

## Publishing

Manual, always: `cargo publish` and the git tag are a person's act. CI never
publishes. The `lucide` workflow only opens a pull request when Lucide ships a
release newer than the crate version.
