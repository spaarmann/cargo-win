# `cargo-win`

Cargo subcommand to run `cargo` on Windows from within a WSL shell.

`cargo-win` is a small tool for a bit of a niche use-case: Developing Rust projects inside the
Windows Subsystem for Linux, but needing to compile and/or run the project on Windows itself. This
comes up for example if you prefer to work in a Linux environment but develop an application
involving a GUI or other features currently not supported in WSL.

To work around those limitations, using this tool you can develop and store such projects inside
WSL, but then run them with a Windows version of `cargo`, `rustc`, etc., all without ever leaving
the WSL shell.

## Usage

```
$ cargo win <cargo subcommand> [args]
```

This will execute `cargo subcommand [args]` on Windows, using the Windows `cargo`.
Examples:
```
$ cargo win run --release
$ cargo win test
```

`cargo-win` will also attempt to detect the current `rustup` toolchain in use and pass a
more-or-less-equivalent `RUSTUP_TOOLCHAIN` environment variable to the Windows side. This cannot use
the exact same toolchain: If `nightly-x86_64-unknown-linux-gnu` is in use on Linux, that cannot be
used on Windows. Instead, `RUSTUP_TOOLCHAIN` will be set to just `nightly` on Windows. This does not
guarantee that the exact same nightly version is used however.

## Installation

Install using `cargo install`, either by cloning yourself and using `cargo install --path`
or by using `cargo install --git`. For the moment, no crates.io version of this exists.

## How it works

`cargo-win` currently creates a target directory in `<temp folder>\cargo-win\<workspace-name>` for
any workspace from which it is executed, and then uses the Windows-WSL interopability features
to execute the Windows `cargo` in the current working directory, but with a `CARGO_TARGET_DIR`
override set to that path.

By using the same path for a workspace every time, `cargo` can re-use dependencies already
compiled and so forth. However the above path is also never cleaned up, so if you care about
not leaving old directories around there forever, you'll have to clean it up yourself.

The subfolder in a temp folder is used instead of e.g. simply a `target-win` folder next to
the `target` folder inside the workspace, because `rustc` unfortunately currently crashes
if incremental compilation is on and the target folder is inside a WSL path.
(https://github.com/rust-lang/rust/issues/49773)

## Issues & Feature Requests

If you encounter any issues using this tool, or need any additional configuration options or
features to make effective use of it, please feel free to open an issue (or a PR)!

## License

Licensed under either
- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0.html), or
- MIT License ([LICENSE-MIT](./LICENSE-MIT) or https://spdx.org/licenses/MIT.html)
at your option.
