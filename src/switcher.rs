use crate::osu_util;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ini::Ini;
use osu_util::restart_osu;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

#[derive(Debug)]
struct AuthDetails {
    username: String,
    password: String,
    server: String,
}

/// Switches osu!'s configuration to replace the authentication details with ones for a different
/// server, if they exist. Afterward, this relaunches osu!.
pub fn switch_servers(osu_dir: &str, target_server: &str) -> Result<()> {
    println!("Using '{osu_dir}' as the target osu! installation!");
    println!("Switching to '{target_server}'");

    let system_username = whoami::username().context("failed getting system username")?;
    let osu_cfg = format!("{osu_dir}/osu!.{system_username}.cfg");
    let osu_exe = format!("{osu_dir}/osu!.exe");
    let osu_db = format!("{osu_dir}/osu!.db");
    let switcher_cfg = format!("{osu_dir}/osu!switcher.ini");

    // Ensure main auth config exists
    if !fs::exists(&*osu_cfg)? {
        println!("Missing osu!.{system_username}.cfg, launching the game normally...");
        clear_logs(&*osu_dir)?;
        restart_osu(&*osu_exe, target_server)?;
        return Ok(());
    }

    // Rename the legacy switcher config to the new file name
    let legacy_cfg = format!("{osu_dir}/server-account-switcher.ini");
    if fs::exists(&*legacy_cfg).unwrap_or(false) {
        fs::rename(&*legacy_cfg, &*switcher_cfg).context("failed migrating old switcher config")?;
    }

    // Create osu!switcher config if not exists
    if !fs::exists(&*switcher_cfg).context("creating osu!switcher config")? {
        File::create(&*switcher_cfg).context("creating osu!switcher config")?;
    }

    // Load configs
    let mut switcher_ini = Ini::load_from_file(&switcher_cfg)
        .with_context(|| format!("failed loading osu!switcher config {switcher_cfg}"))?;
    let mut osu_ini = Ini::load_from_file(&osu_cfg)
        .with_context(|| format!("failed to read osu! config {osu_cfg}"))?;

    // Extract old auth info from osu config
    let old_auth = extract_auth_details(&osu_ini)?;

    clear_logs(&*osu_dir)?;

    // If pending update confirmed, then remove all auth and launch directly
    if !clear_updater(&*osu_dir)? {
        osu_ini
            .with_section(None::<String>)
            .set("Username", "")
            .set("Password", "")
            .set("CredentialEndpoint", "");

        restart_osu(&osu_exe, target_server)?;
        return Ok(());
    }

    if old_auth.server != target_server {
        let new_server = match target_server {
            "osu.ppy.sh" => "",
            server => server,
        };
        let new_auth = match switcher_ini.section(Some(target_server)) {
            None => AuthDetails {
                username: String::from(""),
                password: String::from(""),
                server: String::from(new_server),
            },
            Some(section) => AuthDetails {
                username: section.get("Username").unwrap_or("").to_owned(),
                password: section.get("Password").unwrap_or("").to_owned(),
                server: String::from(new_server),
            },
        };

        edit_db(Path::new(&*osu_db), &*new_auth.username)?;

        osu_ini
            .with_section(None::<String>)
            .set("Username", new_auth.username)
            .set("Password", new_auth.password)
            .set("CredentialEndpoint", new_auth.server);
        osu_ini
            .write_to_file(&osu_cfg)
            .context("failed to write osu! config")?;
    }

    // *Always* save old credentials to switcher config
    switcher_ini
        .with_section(Some(old_auth.server))
        .set("Username", old_auth.username)
        .set("Password", old_auth.password);
    switcher_ini
        .write_to_file(&switcher_cfg)
        .context("failed to write switcher config")?;

    restart_osu(&osu_exe, target_server)?;
    Ok(())
}

/// Extracts authentication details from osu!'s main config.
fn extract_auth_details(osu_config: &Ini) -> Result<AuthDetails> {
    let cfg = osu_config
        .section(None::<String>)
        .context("corrupted osu! config")?;

    let server = match cfg.get("CredentialEndpoint") {
        Some("") | None => "osu.ppy.sh".to_owned(),
        Some(server) => server.to_owned(),
    };

    Ok(AuthDetails {
        server,
        username: cfg.get("Username").unwrap_or("").to_owned(),
        password: cfg.get("Password").unwrap_or("").to_owned(),
    })
}

/// Edits the osu!.db to replace the username stored within.
/// If it doesn't exist, then it is ignored as nothing will happen.
fn edit_db(osu_db: &Path, new_username: &str) -> Result<()> {
    let mut db = osu_db::Listing::from_file(&*osu_db).context("failed to open osu!.db")?;
    db.player_name = Some(new_username.to_owned());
    db.save(&osu_db).context("failed to write osu!.db")?;
    Ok(())
}

/// Clears osu!auth logs
fn clear_logs(osu_dir: &str) -> Result<()> {
    let auth_path = format!("{osu_dir}/Logs/osu!auth.log");

    if fs::exists(&*auth_path)? {
        // I have no clue what this contains, but I have heard about this potentially containing anti-multi-accounting
        // data, which might interfere with switching accounts across servers. Just to be safe, wipe it regardless.
        fs::remove_file(&*auth_path)?;
    }

    Ok(())
}

/// Ask user to delete force updater flag if present.
/// If true returned, then continue switching to a server, otherwise, remove all credentials
/// to allow updater to do its job so that osu! can be safely restarted afterward.
fn clear_updater(osu_dir: &str) -> Result<bool> {
    // If this is present, it causes osu! to relaunch and repair itself,
    // which doesn't preserve the -devserver argument
    let force_update_file = format!("{osu_dir}/.require_update");

    if !fs::exists(&*force_update_file)? {
        return Ok(true);
    }

    print!("Detected a pending osu! repair. Continue [L]aunching or allow [R]epair? ");
    std::io::stdout().flush()?;

    loop {
        match crossterm::event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                kind: KeyEventKind::Press,
                ..
            })) => {
                println!("\nAllowing osu! updater repair to continue...");
                return Ok(false);
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                kind: KeyEventKind::Press,
                ..
            })) => {
                println!("\nCancelling scheduled osu! updater repair...");
                fs::remove_file(force_update_file.as_str())
                    .context("failed deleting osu! force repair flag")?;
            }
            Ok(Event::Key(_)) => {
                println!("Cancelling...");
                exit(1);
            }
            _ => {}
        }
    }
}
