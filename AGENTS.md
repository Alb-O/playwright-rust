# common

```bash
nix develop -c cargo build
nix develop -c cargo test
nix develop -c cargo clippy
```

# format

`nix fmt` (uses treefmt)

# structure

```
crates/
  cli/         # pw-cli binary and commands
  core/        # pw-rs library (public API)
  runtime/     # Playwright server communication
  protocol/    # Wire protocol types
extension/     # Browser extension (wasm)
```

# commit

Use conventional commit style with bullet point descriptive messages.

# testing

- integration tests go in `crates/cli/tests/`
- prefer `data:` URLs to avoid network dependencies
- clear context store between tests for isolation
- use JSON format in tests for assertions: `run_pw(&["-f", "json", ...])`
- always run `cargo test ...` on relevant packages/specific tests after making changes
