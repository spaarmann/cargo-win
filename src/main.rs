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

    let mut cargo_args = args.next().expect("Must provide subcommand to execute!");
    for arg in args {
        cargo_args.push_str(&format!(" \"{}\")", arg));
    }

    let target_dir = find_target_dir();
    let win_path_to_current_dir = get_win_path_to_current_dir();

    let status = Command::new("cmd.exe")
        .current_dir("/mnt/c")
        .args(&[
            "/C",
            &format!(
                "pushd {} && set CARGO_TARGET_DIR={}&& cargo {}",
                win_path_to_current_dir, target_dir, cargo_args
            ),
        ])
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

fn get_win_path_to_current_dir() -> String {
    // Files within WSL distros can be accessed from Windows by using a path like
    // `\\wsl$\<distro name>\<linux path>`.
    // The distro name can be found via the `WSL_DISTRO_NAME` env variable.
    let distro_name = env::var("WSL_DISTRO_NAME").expect("WSL_DISTRO_NAME not set!");
    let current_dir = env::current_dir().expect("Could not get current directory!");

    // This isn't nice, but it's probably good enough for the moment:
    let current_dir = current_dir.to_string_lossy().replace("/", "\\");

    format!("\\\\wsl$\\{}{}", distro_name, current_dir).to_string()
}
