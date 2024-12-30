use std::io::{BufRead, Read, Write};
use std::process;
use std::process::exit;

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
    let _ = std::fs::remove_file(&*format!("{osu_dir}/Logs/osu!auth.log"));

    // If this is present, it causes osu! to relaunch and repair itself, which doesn't preserve -devserver
    let force_update_file = format!("{osu_dir}/.require_update");

    // Check if user wants to continue if .require_update exists
    if std::fs::exists(&*force_update_file).unwrap_or(false) {
        print!("Detected a pending osu! repair! Continue [L]aunching or allow [R]epair? ");
        std::io::stdout().flush().unwrap();

        let line = std::io::stdin().lock().lines().next().unwrap().unwrap();
        match line.to_ascii_uppercase().as_bytes() {
            [b'R'] => {}
            [b'L'] => {
                let _ = std::fs::remove_file(force_update_file.as_str());
            }
            _ => {
                println!("Cancelling... Press enter to exit");
                std::io::stdin().lock().bytes().next();
                exit(1);
            }
        }
    }
}
