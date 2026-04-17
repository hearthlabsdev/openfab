//! # Components
//! 1. DNS-SD advertises queues.
//! 2. IPP attributes describe capabilities.
//! 3. Device selection is an internal policy decision

#[macro_use]
extern crate rocket;

use openmpd::discovery::AdvertiseCmd;
use openmpd_core::IppTxtRecords;
use openmpd_core::advertise::{IppAdvertiser, IppBroadcastBuilder};

use rocket::tokio;
use tokio::sync::mpsc;

#[launch]
async fn rocket() -> _ {
    // Channel for sending advertise commands
    let (tx, mut rx) = mpsc::channel::<AdvertiseCmd>(16);

    // Spawn the advertiser task
    tokio::spawn(async move {
        let mut advertiser = match IppAdvertiser::new() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Failed to start IPP advertiser: {e}");
                return;
            }
        };

        // Example: advertise a default virtual printer on startup
        let default_txt = IppTxtRecords::new()
            .rp("ipp/print")
            .ty("Open Maker Virtual Printer")
            .product("(OpenMPD Virtual Printer)")
            .pdl("application/pdf,image/pwg-raster")
            .note("Development / virtual printer");

        let _ = IppBroadcastBuilder::new("OpenMPD Printer", "openmpd.local.")
            .records(default_txt)
            .secure(false)
            .broadcast(&mut advertiser);

        // Event loop: react to new advertise commands
        while let Some(cmd) = rx.recv().await {
            match cmd {
                AdvertiseCmd::Advertise {
                    instance,
                    hostname,
                    txt,
                    secure,
                } => {
                    if let Err(e) = IppBroadcastBuilder::new(&instance, &hostname)
                        .records(txt)
                        .secure(secure)
                        .broadcast(&mut advertiser)
                    {
                        eprintln!("Failed to advertise printer {instance}: {e}");
                    }
                }
            }
        }

        // When rx closes, advertiser drops → services are unpublished
    });

    // Build Rocket
    rocket::build().manage(tx)
}
