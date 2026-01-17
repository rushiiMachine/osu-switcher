use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, process};

pub fn edit_db(osu_db: &String, username: &String) {
    let mut db = osu_db::Listing::from_file(&osu_db)
        .expect("Failed to open osu!.db");
    db.player_name = Some(username.to_owned());
    db.save(&osu_db)
        .expect("Failed to save osu!.db");
}

pub fn restart_osu(osu_exe: &String, server: &String) {
    let powershell_cmd: &str = "\
        $p = Get-Process -Name osu! -ErrorAction SilentlyContinue; \
        if (!$p) { Exit 0; }; \
        Stop-Process -Force -InputObject $p -ErrorAction Stop; \
        Wait-Process -InputObject $p";

    match powershell_script::run(powershell_cmd) {
        Err(e) => panic!("Failed to kill osu!:\n{e}"),
        _ => println!("Killed running osu!.exe, restarting..."),
    }

    let server = if server == "osu.ppy.sh" { "" } else { server };
    process::Command::new("cmd")
        .args(&[
            "/C",
            "start", osu_exe,
            "-devserver", server,
        ])
        .spawn()
        .expect("Failed to start osu");
}

/// Clear miscellaneous files that might be an issue when relaunching
pub fn clear_misc(osu_dir: &str) {
    // I have no clue what this contains, but I have heard about this potentially containing anti-multi-accounting
    // data, which might interfere with switching accounts across servers. Just to be safe, wipe it regardless.
    let _ = fs::remove_file(&*format!("{osu_dir}/Logs/osu!auth.log"));

    // If this is present, it causes osu! to relaunch and repair itself, which doesn't preserve -devserver
    let force_update_file = format!("{osu_dir}/.require_update");

    // Check if user wants to continue if .require_update exists
    if fs::exists(&*force_update_file).unwrap_or(false) {
        print!("Detected a pending osu! repair! Continue [L]aunching or allow [R]epair? ");
        std::io::stdout().flush().unwrap();

        loop {
            match crossterm::event::read() {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    kind: KeyEventKind::Press,
                    ..
                })) => {
                    // TODO: don't switch accounts in this case
                    println!("\nAllowing osu! updater repair to continue...");
                    break;
                }
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    kind: KeyEventKind::Press,
                    ..
                })) => {
                    println!("\nCancelling scheduled osu! updater repair...");
                    let _ = fs::remove_file(force_update_file.as_str());
                    break;
                }
                Ok(Event::Key(_)) => {
                    println!("Cancelling...");
                    exit(1);
                }
                _ => {}
            }
        }
    }
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
