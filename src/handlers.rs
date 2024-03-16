use anyhow::Result;
use clap::ArgMatches;
use google_calendar3::{
    chrono, hyper::client::HttpConnector, hyper_rustls::HttpsConnector, CalendarHub,
};

pub async fn default(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    args: &ArgMatches,
) -> Result<()> {
    let start_of_today = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let end_of_today = start_of_today + chrono::Duration::try_days(1).unwrap();

    let (_, events) = hub
        .events()
        .list(args.get_one::<String>("id").unwrap()) // safe to unwrap, as the default value is "primary"
        .single_events(true)
        .time_min(start_of_today)
        .time_max(end_of_today)
        .order_by("startTime")
        .doit()
        .await?;
    let events = events.items.unwrap_or_default();

    // println!(
    //     "Events for today ({}):",
    //     chrono::Local::now().format("%Y %B %e, %A")
    // );
    // println!();
    for event in events {
        println!(
            "{}: {}",
            event
                .summary
                .unwrap_or("<No description given>".to_string()),
            event
                .start
                .as_ref()
                .and_then(|start| start.date_time)
                .map(|start| start
                    .with_timezone(&chrono::Local)
                    .format("%H:%M (%Z)")
                    .to_string())
                .unwrap_or("<No start time given>".to_string())
        );
    }

    Ok(())
}

pub async fn calendars(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    args: &ArgMatches,
) -> Result<()> {
    let (_, calendars) = hub.calendar_list().list().show_hidden(true).doit().await?;

    let calendars = calendars.items.unwrap_or_default();

    for calendar in calendars {
        println!(
            "{}",
            match args.get_flag("id") {
                true => calendar.id.unwrap_or_default(),
                false => calendar.summary.unwrap_or_default(),
            }
        );
    }

    Ok(())
}

pub async fn logout() -> Result<()> {
    match std::fs::remove_file("tokencache.json") {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
