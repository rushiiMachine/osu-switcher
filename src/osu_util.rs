use std::process;

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
    // If this is present, it causes osu! to relaunch and repair itself, which doesn't preserve -devserver
    let force_update_file = format!("{osu_dir}/.require_update");

    // I have no clue what this contains, but I have heard about this potentially containing anti-multi-accounting
    // data, which might interfere with switching accounts across servers. Just to be safe, wipe it regardless.
    let osu_auth_logs = format!("{osu_dir}/Logs/osu!auth.log");

    let _ = std::fs::remove_file(force_update_file.as_str());
    let _ = std::fs::remove_file(osu_auth_logs.as_str());
}
