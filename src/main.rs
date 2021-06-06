use std::env;
use std::process::{self, Command, ExitStatus};

use cargo_metadata::MetadataCommand;

// Unfortunately, both camino and std make the assumption that when running on Linux (which we
// are), `(Utf8)Path` & `(Utf8)PathBuf` only refer to Linux paths, and don't support modifying a Windows
// path using these types.
// So instead we unfortunately treat these paths as just Strings when we need to modify them.
// TODO: It'd be nice to find a crate or something that actually supports this use-case
// properly.

fn main() {
    let status = run();
    match status.code() {
        Some(code) => process::exit(code),
        None => process::exit(-1),
    }
}

fn run() -> ExitStatus {
    // When the user executes `cargo win <some command> <more arguments>`,
    // we get passed `["/path/to/cargo-win", "win", <some command>, <more arguments>...]` as args.
    let mut args = env::args();

    // The first two are not interesting, skip them:
    args.next();
    args.next();

    let cargo_args = args.collect::<Vec<_>>();

    if cargo_args.len() == 0 {
        panic!("Must provide cargo subcommand to execute!");
    }

    let target_dir = find_target_dir();
    let rustup_toolchain = get_active_rustup_toolchain();

    let mut command = Command::new("cargo.exe");

    command
        .env("WSLENV", {
            // This instructs WSL to pass the CARGO_TARGET_DIR and RUSTUP_TOOLCHAIN environment
            // variables we set next on to Windows.
            let mut s = env::var("WSLENV").unwrap_or_else(|_| String::new());
            s.push_str(":CARGO_TARGET_DIR/w");
            if rustup_toolchain.is_some() {
                s.push_str(":RUSTUP_TOOLCHAIN/w");
            }
            s
        })
        .env("CARGO_TARGET_DIR", target_dir);

    if let Some(toolchain) = rustup_toolchain.as_ref() {
        command.env("RUSTUP_TOOLCHAIN", toolchain);
    }

    let status = command
        .args(&cargo_args)
        .status()
        .expect("Could not execute cargo command on Windows!`");

    status
}

// Grab a target directory. This doesn't need to exist, or necessarily stick around, and the
// user should not worry too much about it. We still use a predictable path because if we have
// already created one, re-using it allows cargo to not recompile all dependencies etc.
fn find_target_dir() -> String {
    let metadata_command = MetadataCommand::new();
    let metadata = metadata_command
        .exec()
        .expect("Could not get cargo metadata!");

    let workspace_name = metadata
        .workspace_root
        .file_name()
        .expect("Could not find workspace name!");

    let mut target_dir = find_temp_dir().to_string();

    if *target_dir.as_bytes().last().unwrap() != b'\\' {
        target_dir.push('\\');
    }

    target_dir.push_str("cargo-win\\");
    target_dir.push_str(workspace_name);
    target_dir.push_str("\\");

    target_dir
}

fn find_temp_dir() -> String {
    // std::env::temp_dir on Windows returns:
    // "the value of, in order, the TMP, TEMP, USERPROFILE environment variable if any are set and
    // not the empty string. Otherwise, temp_dir returns the path of the Windows directory."
    // We'll try getting TMP and TEMP here and just error out otherwise, not too interested in
    // spewing garbage into USERPROFILE or the Windows directory.

    return try_env("TMP")
        .or_else(|| try_env("TEMP"))
        .expect("Could not get a Windows Temp directory!");

    fn try_env(env: &str) -> Option<String> {
        let output = Command::new("cmd.exe")
            .args(&["/C", &format!("echo %{}%", env)])
            .current_dir("/mnt/c")
            .output()
            .expect("Could not execute `cmd.exe /C \"echo %TMP%\"`");
        let mut out_string =
            String::from_utf8(output.stdout).expect("Did not get valid UTF-8 output from cmd.exe!");

        out_string.truncate(out_string.trim_end().len());

        if out_string.len() > 0 {
            Some(out_string)
        } else {
            None
        }
    }
}

fn get_active_rustup_toolchain() -> Option<String> {
    let output = Command::new("rustup")
        .args(&["show"])
        .current_dir(".")
        .output()
        .ok()?;

    let output =
        String::from_utf8(output.stdout).expect("Did not get valid UTF-8 output from rustup!");

    // Default host: x86_64-unknown-linux-gnu
    let host_regex = regex::Regex::new("Default host: (.*)").unwrap();

    let host = host_regex
        .captures(&output)
        .expect("Could not parse rustup output, host regex did not match!")
        .get(1)
        .expect("Could not parse rustup output, could not get host capture!")
        .as_str();

    // active toolchain
    // ----------------
    //
    // nightly-x86_64-unknown-linux-gnu
    let active_toolchain_regex =
        regex::Regex::new(r"\nactive toolchain\n-+\n\n([a-zA-Z0-9\-_]*)\s").unwrap();
    let active_toolchain = active_toolchain_regex
        .captures(&output)
        .expect("Could not parse rustup output, toolchain regex did not match!")
        .get(1)
        .expect("Could not parse rustup output, could not get toolchain capture!")
        .as_str();

    // This toolchain contains an OS-specific bit which should obviously be different for the
    // windows version, so we cut it out, going from e.g. "nightly-x86_64-unknown-linux-gnu" to
    // just "nightly" which rustup will accept as toolchain.
    let mut toolchain = active_toolchain.replace(host, "");
    toolchain = toolchain.trim_matches('-').to_string();

    Some(toolchain)
}
