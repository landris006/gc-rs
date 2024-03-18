use std::path::PathBuf;

use anyhow::Result;
use clap::{arg, value_parser, Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap::command!()
        .subcommand(
            Command::new("setup")
                .arg(Arg::new("file").value_name("FILE").value_parser(
                    value_parser!(PathBuf)
                ).required(true).help("The path to the credentials file"))
                .about("Copies your Google credentials file to the data directory")
        )
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
                        .value_parser(value_parser!(usize))
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

    gc_rs::handle(matches).await
}
