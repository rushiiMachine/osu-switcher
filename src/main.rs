use std::{env, process};
use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use ini::Ini;
use seahorse::{App, Command, Context, Flag, FlagType};

fn main() {
    let args: Vec<String> = env::args().collect();

    let osu_flag = Flag::new("osu", FlagType::String)
        .description("osu! game directory path");
    let server_flag = Flag::new("server", FlagType::String)
        .description("The target server address (optional). ex: --server akatsuki.pw");

    let switch_cmd = Command::new("switch")
        .description("Switch to a different server account")
        .usage("cli switch --osu <OSU_DIR> --server <SERVER_ADDRESS>")
        .flag(server_flag)
        .flag(osu_flag.clone())
        .action(switch);

    let configure_cmd = Command::new("configure")
        .description("Create desktop shortcuts for servers")
        .usage("cli switch --osu <OSU_DIR>")
        .flag(osu_flag)
        .action(configure);

    App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("cli <command> [...args]")
        .action(configure)
        .command(switch_cmd)
        .command(configure_cmd)
        .run(args);
}

fn configure(_: &Context) {
    sleep(Duration::from_secs(3))
}

fn switch(ctx: &Context) {
    let osu = ctx.string_flag("osu").unwrap();
    let server = ctx.string_flag("server").unwrap_or("<bancho>".to_string());
    println!("Using {0} as the target osu directory!", osu);
    println!("Switching to {0}!", server);

    let system_username = whoami::username();
    println!("Running for user {0}", system_username);

    let osu_cfg = format!("{0}/osu!.{1}.cfg", osu, system_username);
    let osu_exe = format!("{0}/osu!.exe", osu);
    let osu_db = format!("{0}/osu!.db", osu);
    let switcher_cfg = format!("{0}/server-account-switcher.ini", osu);

    if !Path::new(&osu_cfg).exists() || !Path::new(&osu_db).exists() {
        println!("Missing osu!.db or osu!.{0}.cfg, launching the game normally...", system_username);
        launch_osu(&osu_exe, &server);
        return;
    }

    if !Path::new(&switcher_cfg).exists() {
        File::create(&switcher_cfg).unwrap();
    }
    let mut switcher_ini = Ini::load_from_file(&switcher_cfg).unwrap();
    let mut osu_ini = Ini::load_from_file(&osu_cfg).unwrap();

    // rust trickery
    // .section() returns an immutable reference,
    // as long as its in scope I cannot borrow as a mutable reference using .with_section later
    let (old_server, current_username, current_password) = {
        let cfg = osu_ini.section(None::<String>).unwrap();
        let old_server = cfg.get("CredentialEndpoint").unwrap().to_string();
        (
            if old_server != "" { old_server } else { "<bancho>".to_string() },
            cfg.get("Username").unwrap().to_string(),
            cfg.get("Password").unwrap().to_string(),
        )
    };

    if old_server != server.as_str() {
        match switcher_ini.section(Some(&server)) {
            None => {
                osu_ini
                    .with_section(None::<String>)
                    .set("Password", "");
            }
            Some(section) => {
                let new_username = section.get("Username").unwrap();
                let new_password = section.get("Password").unwrap();

                osu_ini.with_section(None::<String>)
                    .set("Username", new_username)
                    .set("Password", new_password)
                    .set("CredentialEndpoint", &server);

                edit_db(&osu_db, &osu_exe, &server, &new_username.to_string());
            }
        }

        switcher_ini
            .with_section(Some(old_server))
            .set("Username", current_username)
            .set("Password", current_password);

        osu_ini.write_to_file(&osu_cfg).unwrap();
        switcher_ini.write_to_file(&switcher_cfg).unwrap();
    }

    launch_osu(&osu_exe, &server);
}

fn edit_db(osu_db: &String, osu_exe: &String, server: &String, username: &String) {
    let mut db = match osu_db::Listing::from_file(&osu_db) {
        Ok(db) => db,
        Err(t) => {
            println!("{0}", t);
            println!("Corrupted osu!.db, launching game normally...");
            launch_osu(&osu_exe, &server);
            return;
        }
    };
    db.player_name = Some(username.to_owned());
    db.save(&osu_db).unwrap();
}

fn launch_osu(osu_exe: &String, server: &String) {
    if !Path::new(&osu_exe).exists() {
        // TODO: windows alert
        println!("Missing game exe! Is this even the correct directory?")
    }

    let output = process::Command::new("taskkill")
        .args(&[
            "/IM",
            "osu!.exe"
        ])
        .output().unwrap();

    if output.stdout.starts_with("SUCCESS".as_bytes()) {
        println!("Killed running osu!.exe, restarting...");
        sleep(Duration::from_secs(1));
    }

    process::Command::new("cmd")
        .args(&[
            "/C", "start", "",
            osu_exe,
            "-devserver",
            if server == "<bancho>" { "" } else { server },
        ])
        .spawn().unwrap();
}
