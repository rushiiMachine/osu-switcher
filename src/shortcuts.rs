use crate::icons;
use mslnk::ShellLink;
use std::fs;
use std::path::Path;

pub fn create_shortcut(osu_dir: &Path, this_exe: &Path, server: &str) {
    let osu_dir_str = osu_dir.to_str()
        .expect("osu! install directory contains invalid characters");

    let name = format!("osu! ({server})");
    let home_path = std::env::var("USERPROFILE").expect("Failed to get user home");
    let link_path = format!("{home_path}/Desktop/{name}.lnk");
    let args = format!("switch --osu \"{osu_dir_str}\" --server \"{server}\"");

    if fs::exists(&*link_path).unwrap() {
        fs::remove_file(&link_path)
            .expect("Failed to delete old shortcut")
    }

    let icon_path = match server {
        "ppy.sh" | "osu.ppy.sh" => icons::osu_server_icon(&*osu_dir),
        _ => icons::write_server_icon(&*osu_dir, &*server)
            .unwrap_or_else(|| icons::osu_server_icon(&*osu_dir)),
    };
    let icon_path = icon_path.to_str()
        .expect("icon path contains invalid characters")
        .to_string();

    let mut link = ShellLink::new(this_exe)
        .expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(icon_path));
    link.set_name(Some(name.clone()));

    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}
