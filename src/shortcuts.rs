use mslnk::ShellLink;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;

/// All the icons of private servers I could find.\
/// They were converted into icon files with ImageMagick:
/// ```shell
/// for file in "./*.{svg,png,jpg,webp};" do
///     magick -background transparent "$file" -define icon:auto-resize=128,64,48,32,16 -set option:wd "%[fx:(1/1)>(w/h)?(1/1*h):w]" -set option:ht "%[fx:(1/1)>(w/h)?h:(w/(1/1))]" -gravity center -background transparent -extent "%[wd]x%[ht]" "../${file%.*}.ico";
/// done
/// ```
/// Known missing icons:
/// - `osuwtf.pw`
/// - `nerose.click`
static ICONS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| HashMap::from([
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
]));

pub fn setup_icons(osu_dir: &String) {
    let icons_path = format!("{osu_dir}/icons");
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

    for (icon, bytes) in ICONS.iter() {
        if files.is_empty() || !files.contains(&icon.to_string()) {
            let path = format!("{osu_dir}/icons/{icon}.ico");
            let path = Path::new(&path);

            let mut file = File::create(path)
                .expect("Failed to create icon");

            file.write_all(bytes)
                .expect("Failed to write icon");
        }
    }
}

pub fn create_shortcut(desktop_path: &String, osu_dir: &String, this_exe: &String, server: &String) {
    let name = format!("osu! ({server})");
    let link_path = format!("{desktop_path}/{name}.lnk");
    let args = format!("switch --osu \"{osu_dir}\" --server \"{server}\"");

    if Path::new(&link_path).exists() {
        fs::remove_file(&link_path)
            .expect("Failed to delete old shortcut")
    }

    let icon_path = if ICONS.contains_key(server.as_str()) {
        format!("{osu_dir}\\icons\\{server}.ico")
    } else {
        format!("{osu_dir}\\osu!.exe")
    };

    let mut link = ShellLink::new(this_exe)
        .expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(icon_path));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}
