use crate::icons;
use mslnk::ShellLink;
use std::fs;
use std::path::Path;

pub fn create_shortcut(osu_dir: &String, this_exe: &String, server: &String) {
    let name = format!("osu! ({server})");
    let home_path = std::env::var("USERPROFILE").expect("Failed to get user home");
    let link_path = format!("{home_path}/Desktop/{name}.lnk");
    let args = format!("switch --osu \"{osu_dir}\" --server \"{server}\"");

    if Path::new(&link_path).exists() {
        fs::remove_file(&link_path)
            .expect("Failed to delete old shortcut")
    }

    let icon_path = icons::write_server_icon(&*osu_dir, &*server)
        .unwrap_or_else(|| icons::osu_server_icon(&*osu_dir));

    let mut link = ShellLink::new(this_exe)
        .expect("Failed to initialize a shortcut");
    link.set_arguments(Some(args));
    link.set_icon_location(Some(icon_path));
    link.set_name(Some(name.clone()));

    println!("Creating shortcut at {link_path}");
    link.create_lnk(link_path)
        .expect("Failed to create shortcut")
}
