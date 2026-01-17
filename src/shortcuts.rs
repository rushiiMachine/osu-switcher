use mslnk::ShellLink;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

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
pub static ICONS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| HashMap::from([
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

/// Returns the path to the osu! logo to be used as a shortcut icon.
/// This resolves to the osu! executable.
fn osu_server_icon(osu_dir: &Path) -> PathBuf {
    osu_dir.join("osu!.exe")
}

/// Writes a server icon shipped with this executable to the osu! directory
/// to be used as shortcut icons, since they need to be on disk.
fn write_server_icon(osu_dir: &Path, server: &str) -> Option<PathBuf> {
    let icons_dir = osu_dir.join("icons");
    let icon_path = icons_dir.join(format!("{server}.ico"));

    if let Some(bytes) = ICONS.get(server) {
        fs::create_dir_all(&*icons_dir)
            .expect(&*format!("failed to create icons directory {icons_dir:?}"));
        fs::write(&*icon_path, bytes).expect(&*format!(
            "failed to write server icon to disk {icons_dir:?}"
        ));
    } else {
        return None;
    }

    Some(icon_path)
}

/// Creates a shortcut on the user's Desktop to this osu!switcher binary that triggers an auth
/// switch to a different osu! private server.  
pub fn create_shortcut(osu_dir: &Path, switcher_path: &Path, server: &str) {
    let home_path = std::env::var("USERPROFILE").expect("Failed to get user home");
    let home_path = Path::new(&*home_path);

    let desktop_path = home_path.join("Desktop");
    if fs::exists(&*desktop_path).unwrap_or(false) {
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

    if fs::exists(&*link_path).unwrap() {
        fs::remove_file(&link_path).expect("Failed to delete old shortcut")
    }

    let icon_path = match server {
        "ppy.sh" | "osu.ppy.sh" => osu_server_icon(&*osu_dir),
        _ => write_server_icon(&*osu_dir, &*server).unwrap_or_else(|| osu_server_icon(&*osu_dir)),
    };

    let mut link = ShellLink::new(switcher_path).expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(
        icon_path
            .to_str()
            .expect("icon path contains invalid characters")
            .to_owned(),
    ));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}
