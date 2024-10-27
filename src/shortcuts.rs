use mslnk::ShellLink;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;

static ICONS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| HashMap::from([
    ("osu.ppy.sh.ico", include_bytes!("../assets/osu.ppy.sh.ico").as_slice()),
    ("akatsuki.pw.ico", include_bytes!("../assets/akatsuki.pw.ico").as_slice()),
    ("kurikku.pw.ico", include_bytes!("../assets/kurikku.pw.ico").as_slice()),
    ("ez-pp.farm.ico", include_bytes!("../assets/ez-pp.farm.ico").as_slice()),
    ("lemres.de.ico", include_bytes!("../assets/lemres.de.ico").as_slice()),
    ("kawata.pw.ico", include_bytes!("../assets/kawata.pw.ico").as_slice()),
    ("gatari.pw.ico", include_bytes!("../assets/gatari.pw.ico").as_slice()),
    ("ussr.pl.ico", include_bytes!("../assets/ussr.pl.ico").as_slice()),
    ("ripple.moe.ico", include_bytes!("../assets/ripple.moe.ico").as_slice()),
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
            let path = format!("{osu_dir}/icons/{icon}");
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

    let icon_path = if ICONS.contains_key(&*format!("{server}.ico")) {
        format!("{osu_dir}/icons/{server}.ico")
    } else {
        format!("{osu_dir}/icons/osu.ppy.sh.ico")
    };

    let mut link = ShellLink::new(this_exe)
        .expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(icon_path));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}
