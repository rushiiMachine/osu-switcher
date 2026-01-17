use color_eyre::eyre::Context;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{fs, process};

/// Forcefully restarts osu! and launches it with a specified server.
pub fn restart_osu(osu_exe: &str, server: &str) -> color_eyre::Result<()> {
    let powershell_cmd: &str = "\
        $p = Get-Process -Name osu! -ErrorAction SilentlyContinue; \
        if (!$p) { Exit 0; }; \
        Stop-Process -Force -InputObject $p -ErrorAction Stop; \
        Wait-Process -InputObject $p";

    powershell_script::run(powershell_cmd).context("failed to kill osu!")?;
    println!("Killed running osu!.exe, restarting...");

    // Remap special server domains
    let server = match server {
        "akatsuki.pw" => "akatsuki.gg", // Moved to new domain
        "osu.ppy.sh" => "",             // Empty argument defaults to Bancho
        server => server,
    };

    process::Command::new("cmd")
        .args(&["/C", "start", osu_exe, "-devserver", server])
        .spawn()
        .context("failed to start osu!")?;

    Ok(())
}

/// Flattens the input osu! installation directory path if it is actually the osu! executable.
pub fn flatten_osu_installation(mut path: &'_ Path) -> Cow<'_, Path> {
    if let Some(file_name) = path.file_name() {
        if file_name == "osu!.exe" {
            if let Some(parent) = path.parent() {
                path = parent;
            }
        }
    }

    path.into()
}

/// Checks whether the specified osu! stable installation directory exists.
pub fn check_osu_installation(dir: &Path) -> bool {
    // OpenTK.dll is checked to ensure this isn't an osu! lazer installation
    fs::exists(dir.join("osu!.exe")).unwrap_or(false)
        && fs::exists(dir.join("OpenTK.dll")).unwrap_or(false)
}

/// Attempts to retrieve an osu! stable installation based on the associated osu! stable application
/// to open `*.osz` files with from the Windows registry.
pub fn find_osu_installation() -> Option<PathBuf> {
    let reg_open_command = windows_registry::CLASSES_ROOT
        .open("osustable.File.osz\\Shell\\Open\\Command")
        .and_then(|key| key.get_string(""));

    if let Ok(open_cmd) = reg_open_command {
        if let Some(osu_exe) = open_cmd.split("\"").skip(1).next() {
            let osu_exe = Path::new(osu_exe);
            let osu_dir = flatten_osu_installation(osu_exe);

            return if check_osu_installation(&*osu_dir) {
                Some(osu_dir.into_owned())
            } else {
                None
            };
        }
    }
    None
}
