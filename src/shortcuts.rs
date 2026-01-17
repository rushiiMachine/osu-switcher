use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use mslnk::ShellLink;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

/// All the icons of private servers I could find.\
/// They were converted into icon files with ImageMagick:
/// ```shell
/// for file in "./*.{svg,png,jpg,webp};" do
///     magick -background transparent "$file" \
///         -define icon:auto-resize=128,64,48,32,16 \
///         -set option:wd "%[fx:(1/1)>(w/h)?(1/1*h):w]" \
///         -set option:ht "%[fx:(1/1)>(w/h)?h:(w/(1/1))]" \
///         -gravity center -background transparent -extent "%[wd]x%[ht]" \
///         "../${file%.*}.ico";
/// done
/// ```
/// Known missing icons:
/// - `osuwtf.pw`
/// - `nerose.click`
// TODO: add localhost icon
static ICONS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| HashMap::from([
    // @formatter:off
    ("akatsuki.gg", include_bytes!("../assets/akatsuki.gg.ico").as_slice()),
    ("akatsuki.pw", include_bytes!("../assets/akatsuki.pw.ico").as_slice()),
    ("ez-pp.farm", include_bytes!("../assets/ez-pp.farm.ico").as_slice()),
    ("fuquila.net", include_bytes!("../assets/fuquila.net.ico").as_slice()),
    ("gatari.pw", include_bytes!("../assets/gatari.pw.ico").as_slice()),
    ("halcyon.moe", include_bytes!("../assets/halcyon.moe.ico").as_slice()),
    ("kawata.pw", include_bytes!("../assets/kawata.pw.ico").as_slice()),
    ("kokisu.moe", include_bytes!("../assets/kokisu.moe.ico").as_slice()),
    ("lemres.de", include_bytes!("../assets/lemres.de.ico").as_slice()),
    ("mamesosu.net", include_bytes!("../assets/mamesosu.net.ico").as_slice()),
    ("osunolimits.dev", include_bytes!("../assets/osunolimits.dev.ico").as_slice()),
    ("osuokayu.moe", include_bytes!("../assets/osuokayu.moe.ico").as_slice()),
    ("redstar.moe", include_bytes!("../assets/redstar.moe.ico").as_slice()),
    ("ripple.moe", include_bytes!("../assets/ripple.moe.ico").as_slice()),
    ("scosu.net", include_bytes!("../assets/scosu.net.ico").as_slice()),
    ("seventwentyseven.xyz", include_bytes!("../assets/seventwentyseven.xyz.ico").as_slice()),
    ("ussr.pl", include_bytes!("../assets/ussr.pl.ico").as_slice()),
    // @formatter:on
]));

/// Returns all known osu! server domains.
pub fn known_servers() -> Vec<String> {
    let mut servers = vec!["osu.ppy.sh".to_owned()];
    servers.extend(ICONS.keys().map(|s| (*s).to_owned()));
    servers.sort_unstable();
    servers
}

/// Returns the path to the osu! logo to be used as a shortcut icon.
/// This resolves to the osu! executable.
fn osu_server_icon(osu_dir: &Path) -> PathBuf {
    osu_dir.join("osu!.exe")
}

/// Writes a server icon shipped with this executable to the osu! directory
/// to be used as shortcut icons, since they need to be on disk.
fn write_server_icon(osu_dir: &Path, server: &str) -> Result<PathBuf> {
    let icons_dir = osu_dir.join("icons");
    let icon_path = icons_dir.join(format!("{server}.ico"));

    // Fast path
    if server == "ppy.sh" || server == "osu.ppy.sh" {
        return Ok(osu_server_icon(&*osu_dir));
    }

    if let Some(bytes) = ICONS.get(server) {
        fs::create_dir_all(&*icons_dir)
            .with_context(|| format!("failed to create icons directory {icons_dir:?}"))?;
        fs::write(&*icon_path, bytes)
            .with_context(|| format!("failed to write server icon to disk {icons_dir:?}"))?;

        Ok(icon_path)
    } else {
        Ok(osu_server_icon(&*osu_dir))
    }
}

/// Creates a shortcut on the user's Desktop to this osu!switcher binary that triggers an auth
/// switch to a different osu! private server.
fn create_shortcut(osu_dir: &Path, switcher_path: &Path, server: &str) -> Result<()> {
    let home_path = env::var_os("USERPROFILE")
        .context("USERPROFILE environment variable unset")?;
    let home_path = Path::new(&*home_path);

    let desktop_path = home_path.join("Desktop");
    if !fs::exists(&*desktop_path).unwrap_or(false) {
        panic!("user desktop directory does not exist!");
    }

    let name = format!("osu! ({server})");
    let link_path = desktop_path.join(&*format!("{name}.lnk"));
    let args = format!(
        "switch --osu \"{0}\" --server \"{server}\"",
        osu_dir
            .to_str()
            .expect("osu! install directory contains invalid characters")
    );

    let icon_path = write_server_icon(osu_dir, server)?;

    if fs::exists(&*link_path).unwrap_or(false) {
        fs::remove_file(&link_path)
            .with_context(|| format!("failed to delete old shortcut {link_path:?}"))?;
    }

    let mut link = ShellLink::new(switcher_path)
        .with_context(|| format!("failed to create shortcut {switcher_path:?}"))?;
    link.set_arguments(Some(args));
    link.set_icon_location(Some(
        icon_path
            .to_str()
            .expect("icon path contains invalid characters")
            .to_owned(),
    ));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .with_context(|| format!("failed to create shortcut {switcher_path:?}"))?;
    Ok(())
}

/// Installs this switcher in a permanent location and creates the specified server shortcuts.
pub fn install<'a, S>(osu_dir: &Path, servers: S) -> Result<()>
where
    S: IntoIterator<Item=&'a str>,
{
    let this_exe = env::current_exe()
        .context("failed to get path to current running executable")?;
    let localappdata = env::var_os("LOCALAPPDATA")
        .context("LOCALAPPDATA environment variable unset")?;

    // Install self to permanent location
    let installed_exe = if !this_exe.starts_with(&*localappdata) {
        let install_dir = Path::new(&*localappdata).join("osu!switcher");
        let new_exe = install_dir.join("osu!switcher.exe");
        let readme_exe = install_dir.join("README.txt");

        const README_BANNER: &str = "\
        This is the permanent installation location of osu!switcher (https://github.com/rushiiMachine/osu-switcher).\n\
        The 'osu!switcher.exe' executable is referenced by the osu! shortcuts generated onto the desktop.\n\
        ";

        fs::create_dir_all(&*install_dir)
            .with_context(|| format!("failed to create installation dir {install_dir:?}"))?;
        fs::copy(&*this_exe, &*new_exe)
            .with_context(|| format!("failed to copy current executable to installation dir {new_exe:?}"))?;
        fs::write(&*readme_exe, README_BANNER)?;

        new_exe
    } else {
        this_exe
    };

    for server in servers {
        create_shortcut(osu_dir, &*installed_exe, server)?;
    }

    Ok(())
}
