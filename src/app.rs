use anyhow::Result;
use calendar3::{
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::{self, ApplicationSecret},
    CalendarHub,
};
use clap::ArgMatches;
use google_calendar3 as calendar3;

use crate::handlers;

pub struct App {
    hub: CalendarHub<HttpsConnector<HttpConnector>>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let secret: ApplicationSecret = oauth2::read_application_secret("secret.json").await?;

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("tokencache.json")
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

        Ok(Self { hub })
    }

    pub async fn handle(&self, matches: ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("logout", _)) => {
                handlers::logout().await?;
            }
            Some(("calendars", args)) => {
                handlers::calendars(&self.hub, args).await?;
            }
            Some(("meet", args)) => {
                handlers::meet(&self.hub, args).await?;
            }
            _ => {
                handlers::default(&self.hub, &matches).await?;
            }
        }

        Ok(())
    }
}
