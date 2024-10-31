use std::fs::File;
use std::io::{BufRead, Read};
use std::path::Path;
use std::{env, io, panic};

use ini::Ini;
use seahorse::{App, Command, Context, Flag, FlagType};

mod shortcuts;
mod osu_util;

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
        env::set_var("RUST_BACKTRACE", "1");
        app.run(env::args().collect())
    });

    if app_result.is_err() {
        println!("\nAn error has occurred! Please create an issue on this project's Github with the log! ({0}/issues)", env!("CARGO_PKG_REPOSITORY"));
        println!("Press enter to exit...");
        io::stdin().lock().bytes().next();
    };
}

fn configure(_: &Context) {
    println!("This executable will have to remain intact in order for the shortcuts to work!");
    println!("Please ensure its in a permanent spot. (exit now if you need to)\n");

    let stdin = io::stdin();
    let default_osu_path = "%appdata%/Local/osu!";

    let osu_dir = if Path::new(&format!("{default_osu_path}/osu!.exe")).exists() {
        println!("Detected osu! installation at {default_osu_path}");
        default_osu_path.to_string()
    } else {
        println!("Could not detect osu installation! Please enter your osu! directory path below:");
        let mut path = String::new();
        for in_path in stdin.lock().lines() {
            let in_path = in_path.unwrap();

            if !Path::new(&format!("{in_path}/osu!.exe")).exists() {
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

    shortcuts::setup_icons(&osu_dir);

    let this_exe = &env::current_exe().unwrap().to_string_lossy().to_string();
    for server in servers {
        shortcuts::create_shortcut(&osu_dir, &this_exe, &server);
    }

    println!("Created shortcuts! Press enter to exit...");
    stdin.lock().bytes().next();
}

fn switch(ctx: &Context) {
    let osu_dir = ctx.string_flag("osu")
        .expect("The --osu flag is required in order to start osu");
    let server = ctx.string_flag("server")
        .unwrap_or("osu.ppy.sh".to_string());
    println!("Using {osu_dir} as the target osu directory!");
    println!("Switching to {server}!");

    let system_username = whoami::username();
    println!("Running for user {system_username}");

    let osu_cfg = format!("{osu_dir}/osu!.{system_username}.cfg");
    let osu_exe = format!("{osu_dir}/osu!.exe");
    let osu_db = format!("{osu_dir}/osu!.db");
    let switcher_cfg = format!("{osu_dir}/server-account-switcher.ini");

    if !Path::new(&osu_cfg).exists() || !Path::new(&osu_db).exists() {
        println!("Missing osu!.db or osu!.{system_username}.cfg, launching the game normally...");
        osu_util::restart_osu(&osu_exe, &server);
        return;
    }

    if !Path::new(&switcher_cfg).exists() {
        File::create(&switcher_cfg)
            .expect("Failed to create switcher config");
    }

    let mut switcher_ini = Ini::load_from_file(&switcher_cfg)
        .expect("Failed to read switcher config");
    let mut osu_ini = Ini::load_from_file(&osu_cfg)
        .expect(&format!("Failed to read osu!.{system_username}.cfg"));

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
                let new_server = if server == "osu.ppy.sh" { "" } else { &server };

                osu_ini.with_section(None::<String>)
                    .set("Username", new_username)
                    .set("Password", new_password)
                    .set("CredentialEndpoint", new_server);

                osu_util::edit_db(&osu_db, &new_username.to_string());
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

    osu_util::clear_misc(&*osu_dir);
    osu_util::restart_osu(&osu_exe, &server);
}
