use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use colored::Colorize;
use google_calendar3::{
    api::Event, chrono, hyper::client::HttpConnector, hyper_rustls::HttpsConnector, CalendarHub,
};

pub async fn meet(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    args: &ArgMatches,
) -> Result<()> {
    let events = get_events(
        hub,
        args.get_one::<String>("calendar-id").map(|id| id.as_str()),
    )
    .await?;

    let event_index = args.get_one::<usize>("event-index").map(|index| index - 1);

    if let Some(index) = event_index {
        let event = events
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("No event found at index {}", index + 1))?;

        let url = extract_link(event)
            .ok_or_else(|| anyhow::anyhow!("No Google Meet link found for event"))?;

        println!(
            "Opening Google Meet link for '{}' ({})",
            event.summary.as_ref().unwrap_or(&"".to_string()),
            url
        );
        return Ok(open::that(url)?);
    }

    let ongoing_event = events.iter().find(|event| {
        let has_started: bool = event
            .start
            .as_ref()
            .and_then(|start| start.date_time)
            .map(|start| start <= chrono::Utc::now())
            .unwrap_or(false);
        let has_ended: bool = event
            .end
            .as_ref()
            .and_then(|end| end.date_time)
            .map(|end| end <= chrono::Utc::now())
            .unwrap_or(false);

        has_started && !has_ended
    });

    if let Some(ongoing_event) = ongoing_event {
        let url = extract_link(ongoing_event)
            .ok_or_else(|| anyhow::anyhow!("No Google Meet link found for ongoing event"))?;

        println!(
            "Opening Google Meet link for ongoing event '{}' ({})",
            ongoing_event.summary.as_ref().unwrap_or(&"".to_string()),
            url
        );
        return Ok(open::that(url)?);
    }

    let next_event = events
        .iter()
        .find(|event| {
            event
                .start
                .as_ref()
                .and_then(|start| start.date_time)
                .map(|start| start > chrono::Utc::now())
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("No upcoming events found"))?;

    let url = extract_link(next_event).ok_or_else(|| {
        anyhow::anyhow!("No ongoing or upcoming events found, specify the index of the event")
    })?;

    println!(
        "Opening Google Meet link for next event '{}' ({})",
        next_event.summary.as_ref().unwrap_or(&"".to_string()),
        url
    );
    Ok(open::that(url)?)
}

fn extract_link(event: &Event) -> Option<&str> {
    let url = event.hangout_link.as_ref().or_else(|| {
        event.conference_data.as_ref().and_then(|data| {
            data.entry_points.as_ref().and_then(|entry_points| {
                entry_points
                    .first()
                    .and_then(|entry_point| entry_point.uri.as_ref())
            })
        })
    });

    url.map(|url| url.as_str())
}

pub async fn default(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    args: &ArgMatches,
) -> Result<()> {
    let events = get_events(
        hub,
        args.get_one::<String>("calendar-id").map(|id| id.as_str()),
    )
    .await?;

    for (index, event) in events.into_iter().enumerate() {
        let has_started: bool = event
            .start
            .as_ref()
            .and_then(|start| start.date_time)
            .map(|start| start <= chrono::Utc::now())
            .unwrap_or(false);
        let has_ended: bool = event
            .end
            .as_ref()
            .and_then(|end| end.date_time)
            .map(|end| end <= chrono::Utc::now())
            .unwrap_or(false);

        let is_ongoing = has_started && !has_ended;

        let is_close = !has_started
            && event
                .start
                .as_ref()
                .and_then(|start| start.date_time)
                .map(|start| {
                    start <= chrono::Utc::now() + chrono::Duration::try_minutes(30).unwrap()
                })
                .unwrap_or(false);

        let mut out = format!(
            "{}. {}: {}",
            index + 1,
            event
                .summary
                .unwrap_or("<No description given>".to_string()),
            event
                .start
                .as_ref()
                .and_then(|start| start.date_time)
                .map(|start| start
                    .with_timezone(&chrono::Local)
                    .format("%H:%M")
                    .to_string())
                .unwrap_or("<No start time given>".to_string())
        );

        let is_recurring = event
            .recurring_event_id
            .map(|id| !id.is_empty())
            .unwrap_or(false);

        if is_recurring {
            out.push_str(" (recurring)");
        }

        if is_ongoing {
            out.push_str(" (ONGOING)");
            println!("{}", out.red());
            continue;
        }

        if is_close {
            println!("{}", out.yellow());
            continue;
        }

        if has_ended {
            println!("{}", out.dimmed());
            continue;
        }

        println!("{}", out);
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

async fn get_events(
    hub: &CalendarHub<HttpsConnector<HttpConnector>>,
    calendar_id: Option<&str>,
) -> Result<Vec<Event>> {
    let start_of_today = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let end_of_today = start_of_today + chrono::Duration::try_days(1).unwrap();

    let (_, events) = hub
        .events()
        .list(calendar_id.unwrap_or("primary"))
        .single_events(true)
        .time_min(start_of_today)
        .time_max(end_of_today)
        .order_by("startTime")
        .doit()
        .await?;
    let events = events.items.unwrap_or_default();

    Ok(events)
}

pub async fn setup(args: &ArgMatches) -> Result<()> {
    let path = args.get_one::<PathBuf>("file").expect("file is required");
    let file = std::fs::read_to_string(path)?;

    let data_dir = crate::data_dir()?;

    std::fs::write(data_dir.join("secret.json"), file)?;

    println!(
        "Credentials file copied to '{}'",
        data_dir.join("secret.json").to_str().unwrap_or("")
    );

    Ok(())
}
