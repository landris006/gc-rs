mod handlers;

use anyhow::Result;
use calendar3::{
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::{self, ApplicationSecret},
    CalendarHub,
};
use clap::ArgMatches;
use google_calendar3 as calendar3;
use std::{env, ffi::OsStr, path::Path};

fn bin_name() -> Option<String> {
    env::current_exe()
        .ok()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}

fn data_dir() -> Result<std::path::PathBuf> {
    let data_dir = dirs::data_dir()
        .map(|dir| dir.join("gc-rs"))
        .ok_or_else(|| anyhow::anyhow!("No data directory found"))?;

    std::fs::create_dir_all(&data_dir)?;

    Ok(data_dir)
}

pub async fn get_hub() -> Result<CalendarHub<HttpsConnector<HttpConnector>>> {
    let data_dir = data_dir()?;

    let secret: ApplicationSecret = oauth2::read_application_secret(data_dir.join("secret.json"))
        .await
        .map_err(|_| {
            anyhow::anyhow!(format!(
                "No secret.json found in '{}' directory. Please run '{} setup'.",
                data_dir.to_str().unwrap_or(""),
                bin_name().unwrap_or("gc".to_string())
            ))
        })?;

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk(data_dir.join("tokencache.json"))
    .build()
    .await?;

    let hub = CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    Ok(hub)
}

pub async fn handle(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("setup", args)) => {
            handlers::setup(args).await?;
        }
        _ => {
            let hub = get_hub().await?;

            match matches.subcommand() {
                Some(("logout", _)) => {
                    handlers::logout().await?;
                }
                Some(("calendars", args)) => {
                    handlers::calendars(&hub, args).await?;
                }
                Some(("meet", args)) => {
                    handlers::meet(&hub, args).await?;
                }
                _ => {
                    handlers::default(&hub, &matches).await?;
                }
            }
        }
    }

    Ok(())
}
