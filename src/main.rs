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
        .subcommand(
            Command::new("meet")
                .about("Open Google Meet link")
                .arg(
                    Arg::new("event-index")
                        .value_name("INDEX")
                        .help("Optionally specify the index of the event to open (defaults to the ongoing or next event)")
                )
                .arg(
                    Arg::new("calendar-id")
                        .long("calendar-id")
                        .default_value("primary")
                        .help("Specify the ID of the calendar to use"),
                )
        )
        .arg(
            Arg::new("calendar-id")
                .long("calendar-id")
                .default_value("primary")
                .help("Specify the ID of the calendar to use"),
        )
        .get_matches();

    let app = App::new().await?;

    app.handle(matches).await
}
