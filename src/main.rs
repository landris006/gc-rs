mod app;
mod handlers;

use anyhow::Result;
use app::App;
use clap::{arg, Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap::command!()
        .subcommand(
            Command::new("calendars")
                .about("List avaliable calendars")
                .arg(arg!(--id "Display the IDs of the calendars")),
        )
        .subcommand(Command::new("logout").about("Delete the cached access tokens"))
        .arg(
            Arg::new("id")
                .long("id")
                .default_value("primary")
                .help("Specify the ID of the calendar to use (defaults to 'primary')"),
        )
        .get_matches();

    let app = App::new().await?;

    app.handle(matches).await
}
