use std::{env, fs, io, panic, process};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use ini::Ini;
use mslnk::ShellLink;
use seahorse::{App, Command, Context, Flag, FlagType};

fn main() {
    let osu_flag = Flag::new("osu", FlagType::String)
        .description("osu! game directory path");
    let server_flag = Flag::new("server", FlagType::String)
        .description("The target server address (optional). ex: --server akatsuki.pw");

    let switch_cmd = Command::new("switch")
        .description("Switch to a different server account")
        .usage("osu-switcher.exe switch --osu <OSU_DIR> --server <SERVER_ADDRESS>")
        .flag(server_flag)
        .flag(osu_flag.clone())
        .action(switch);

    let configure_cmd = Command::new("configure")
        .description("Create desktop shortcuts for servers")
        .usage("osu-switcher.exe switch --osu <OSU_DIR>")
        .action(configure);

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("osu-switcher.exe <command> [...args]")
        .action(configure)
        .command(switch_cmd)
        .command(configure_cmd);

    let app_result = panic::catch_unwind(|| {
        app.run(env::args().collect())
    });

    if app_result.is_err() {
        println!("\nPress enter to exit...");
        io::stdin().lock().bytes().next();
    };
}

fn configure(_: &Context) {
    println!("This executable will have to remain intact in order for the shortcuts to work!");
    println!("Please ensure its in a permanent spot. (exit now if you need to)\n");

    let username = whoami::username();
    let stdin = io::stdin();
    let default_osu_path = format!("C:/Users/{0}/Appdata/Local/osu!", username);

    let osu_dir = if Path::new(&format!("{0}/osu!.exe", default_osu_path)).exists() {
        println!("Detected osu! installation at {0}", default_osu_path);
        default_osu_path
    } else {
        println!("Could not detect osu installation! Please enter your osu! directory path below:");
        let mut path = String::new();
        for in_path in stdin.lock().lines() {
            let in_path = in_path.unwrap();

            if !Path::new(&format!("{0}/osu!.exe", in_path)).exists() {
                println!("Invalid osu installation! (osu!.exe missing)");
                continue;
            }

            path = in_path;
            break;
        }
        path
    };

    let mut servers = Vec::from(["osu.ppy.sh".to_string()]);
    println!("\nPlease enter the server addresses you want to generate shortcuts for! Press enter after each or to finish.");
    println!("Servers: {0}", servers.join(", "));

    for server in stdin.lock().lines() {
        let server = server.unwrap();
        if server == "" {
            break;
        }

        if !server.contains(".") {
            println!("Invalid server address!");
            continue;
        }

        servers.push(server);
        println!("\nServers: {0}", servers.join(", "))
    }

    let icons: HashMap<&str, &[u8]> = HashMap::from([
        ("osu.ppy.sh.ico", include_bytes!("../assets/osu.ppy.sh.ico").as_slice()),
        ("akatsuki.pw.ico", include_bytes!("../assets/akatsuki.pw.ico").as_slice()),
        ("kurikku.pw.ico", include_bytes!("../assets/kurikku.pw.ico").as_slice()),
        ("ez-pp.farm.ico", include_bytes!("../assets/ez-pp.farm.ico").as_slice()),
        ("lemres.de.ico", include_bytes!("../assets/lemres.de.ico").as_slice()),
        ("kawata.pw.ico", include_bytes!("../assets/kawata.pw.ico").as_slice()),
        ("gatari.pw.ico", include_bytes!("../assets/gatari.pw.ico").as_slice()),
        ("ussr.pl.ico", include_bytes!("../assets/ussr.pl.ico").as_slice()),
        ("ripple.moe.ico", include_bytes!("../assets/ripple.moe.ico").as_slice()),
    ]);

    setup_icons(&osu_dir, &icons);

    let desktop_path = format!("C:/Users/{0}/Desktop", username);
    let this_exe = &env::current_exe().unwrap().to_string_lossy().to_string();
    for server in servers {
        create_shortcut(&desktop_path, &osu_dir, &this_exe, &server, &icons);
    }

    println!("Created shortcuts! Press enter to exit...");
    stdin.lock().bytes().next();
}

fn create_shortcut(desktop_path: &String, osu_dir: &String, this_exe: &String, server: &String, icons: &HashMap<&str, &[u8]>) {
    let name = format!("osu! ({0})", server);
    let link_path = format!("{0}/{1}.lnk", desktop_path, name);
    let args = format!("switch --osu \"{0}\" --server \"{1}\"", osu_dir, server);

    if Path::new(&link_path).exists() {
        fs::remove_file(&link_path)
            .expect("Failed to delete old shortcut")
    }

    let icon_path = if icons.contains_key(&*format!("{0}.ico", server)) {
        format!("{0}/icons/{1}.ico", osu_dir, server)
    } else {
        format!("{0}/icons/osu.ppy.sh.ico", osu_dir)
    };

    let mut link = ShellLink::new(this_exe)
        .expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(icon_path));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}

fn switch(ctx: &Context) {
    let osu_dir = ctx.string_flag("osu")
        .expect("The --osu flag is required in order to start osu");
    let server = ctx.string_flag("server")
        .unwrap_or("osu.ppy.sh".to_string());
    println!("Using {0} as the target osu directory!", osu_dir);
    println!("Switching to {0}!", server);

    let system_username = whoami::username();
    println!("Running for user {0}", system_username);

    let osu_cfg = format!("{0}/osu!.{1}.cfg", osu_dir, system_username);
    let osu_exe = format!("{0}/osu!.exe", osu_dir);
    let osu_db = format!("{0}/osu!.db", osu_dir);
    let switcher_cfg = format!("{0}/server-account-switcher.ini", osu_dir);

    if !Path::new(&osu_cfg).exists() || !Path::new(&osu_db).exists() {
        println!("Missing osu!.db or osu!.{0}.cfg, launching the game normally...", system_username);
        launch_osu(&osu_exe, &server);
        return;
    }

    if !Path::new(&switcher_cfg).exists() {
        File::create(&switcher_cfg)
            .expect("Failed to create switcher config");
    }

    let mut switcher_ini = Ini::load_from_file(&switcher_cfg)
        .expect("Failed to read switcher config");
    let mut osu_ini = Ini::load_from_file(&osu_cfg)
        .expect(&format!("Failed to read osu!.{}.cfg config", system_username));

    // rust trickery
    // .section() returns an immutable reference,
    // as long as its in scope I cannot borrow as a mutable reference using .with_section later
    let (old_server, current_username, current_password) = {
        let cfg = osu_ini.section(None::<String>)
            .expect("Corrupted osu user config");

        let old_server = cfg.get("CredentialEndpoint")
            .unwrap_or("")
            .to_string();

        (
            if old_server != "" { old_server } else { "osu.ppy.sh".to_string() },
            cfg.get("Username").unwrap_or("").to_string(),
            cfg.get("Password").unwrap_or("").to_string(),
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
                let new_username = section.get("Username").unwrap_or("");
                let new_password = section.get("Password").unwrap_or("");

                osu_ini.with_section(None::<String>)
                    .set("Username", new_username)
                    .set("Password", new_password)
                    .set("CredentialEndpoint", &server);

                edit_db(&osu_db, &new_username.to_string());
            }
        }

        switcher_ini
            .with_section(Some(old_server))
            .set("Username", current_username)
            .set("Password", current_password);

        osu_ini.write_to_file(&osu_cfg)
            .expect("Failed to save osu user config");
        switcher_ini.write_to_file(&switcher_cfg)
            .expect("Failed to save switcher config");
    }

    launch_osu(&osu_exe, &server);
}

fn edit_db(osu_db: &String, username: &String) {
    let mut db = osu_db::Listing::from_file(&osu_db)
        .expect("Failed to open osu!.db");
    db.player_name = Some(username.to_owned());
    db.save(&osu_db)
        .expect("Failed to save osu!.db");
}

fn launch_osu(osu_exe: &String, server: &String) {
    let output = process::Command::new("taskkill")
        .args(&[
            "/IM",
            "osu!.exe"
        ])
        .output()
        .expect("Failed to kill osu");

    if output.stdout.starts_with("SUCCESS".as_bytes()) {
        println!("Killed running osu!.exe, restarting...");
        sleep(Duration::from_secs(1));
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

fn setup_icons(osu_dir: &String, icons: &HashMap<&str, &[u8]>) {
    let icons_path = format!("{}/icons", osu_dir);
    let icons_path = Path::new(&icons_path);

    let files: Vec<String> = if icons_path.exists() {
        fs::read_dir(osu_dir)
            .unwrap()
            .filter(|d| d.is_ok())
            .map(|d| d.unwrap().file_name().into_string().unwrap())
            .collect()
    } else {
        fs::create_dir(icons_path)
            .expect("Failed to create icons dir");
        vec![]
    };

    for (icon, bytes) in icons {
        if files.is_empty() || !files.contains(&icon.to_string()) {
            let path = format!("{0}/icons/{1}", osu_dir, icon);
            let path = Path::new(&path);

            let mut file = File::create(path)
                .expect("Failed to create icon");

            file.write_all(bytes)
                .expect("Failed to write icon");
        }
    }
}
