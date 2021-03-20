# `cargo-win`

Cargo subcommand to run `cargo` on Windows from within a WSL shell.

This is a useful tool for a bit of a niche use-case. I prefer to write Rust
from a Linux environment, but use Windows as daily driver for various other
reasons. This usually works very well by just using the Windows Subsystem
for Linux, but there is a major exception: Any graphical applications are
currently not supported inside WSL.

As a workaround, using this tool, I can still develop and store such projects
inside WSL, but then run them with a Windows version of `cargo`, `rustc`, etc.,
all without ever leaving the WSL shell.

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

## Installation

Install using `cargo install`, either by cloning yourself and using `cargo install --path`
or by using `cargo install --git`. For the moment, no crates.io version of this exists.

## How it works

It currently creates a target directory in `<temp folder>\cargo-win\<workspace-name>` for
any workspace from which it is executed, and then uses the Windows-WSL interopability features
to execute the Windows `cargo` in the current working directory, but with a `CARGO_TARGET_DIR`
override set to that path.

By using the same path for a workspace every time, `cargo` can re-use dependencies already
compiled and so forth. However the above path is also never cleaned up, so if you care about
not leaving old directories around there forever, you'll have to clean it up yourself.

The subfolder in a temp folder is used instead of e.g. simply a `target-win` folder next to
the `target` folder inside the workspace, because `rustc` unfortunately currently crashes
if incremental compilation is on and the target folder is inside a WSL path.
(https://github.com/rust-lang/rust/issues/66513)

## Issues & Feature Requests

If you encounter any issues using this tool, or need any additional configuration options to
make effective use of it (there are currently none at all), please feel free to open an issue
(or a PR)! This currently does what I personally need it to do, but I would love to make it
useful to others too.

## License

This project is licensed under the MIT license, see LICENSE.md.
