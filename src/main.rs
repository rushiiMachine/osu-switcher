use crate::tui::start_tui;
use seahorse::{ActionError, ActionResult, App, Command, Context, Flag, FlagType};
use std::{env, panic};

mod osu_util;
mod shortcuts;
mod switcher;
mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let osu_flag = Flag::new("osu", FlagType::String).description("osu! game directory path");
    let server_flag = Flag::new("server", FlagType::String)
        .description("The target server address (optional). ex: --server akatsuki.pw");

    let switch_cmd = Command::new("switch")
        .description("Switch to a different server account")
        .usage("osu-switcher.exe switch --osu <OSU_DIR> --server <SERVER_ADDRESS>")
        .flag(server_flag)
        .flag(osu_flag.clone())
        .action_with_result(switch);

    let configure_cmd = Command::new("configure")
        .description("Create desktop shortcuts for servers")
        .usage("osu-switcher.exe switch --osu <OSU_DIR>")
        .action(|_| start_tui());

    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("osu-switcher.exe <command> [...args]")
        .action(|_| start_tui())
        .command(switch_cmd)
        .command(configure_cmd);

    let app_result = panic::catch_unwind(|| app.run(env::args().collect()));

    if app_result.is_err() {
        println!(
            "\nAn error has occurred! Please create an issue on this \
        project's Github with the log! ({0}/issues)",
            env!("CARGO_PKG_REPOSITORY")
        );
    };

    Ok(())
}

fn switch(ctx: &Context) -> ActionResult {
    let osu_dir = match ctx.string_flag("osu") {
        Ok(s) => s,
        Err(_) => {
            return Err(ActionError {
                message: "The --osu flag is required in order to start osu".to_owned(),
            });
        }
    };
    let server = ctx
        .string_flag("server")
        .unwrap_or("osu.ppy.sh".to_string());

    switcher::switch_servers(&*osu_dir, &*server).unwrap();
    Ok(())
}
