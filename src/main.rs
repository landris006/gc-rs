mod app;
mod handlers;

use app::App;
use clap::{arg, Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::command!()
        .subcommand(
            Command::new("calendars")
                .about("List avaliable calendars")
                .arg(arg!(--id "Display the IDs of the calendars")),
        )
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
