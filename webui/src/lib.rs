#![allow(unused_imports)]
#![allow(unused_variables)]
pub mod ui;
pub mod api;
pub mod admin;
pub mod accounts;
pub mod config;
pub mod dashboard;
pub mod devices;
pub mod prints;
pub mod library;

pub mod discovery;
pub mod errors;
pub mod routes;
pub mod utils;

use crate::discovery::AdvertiseCmd;
use openfab::IppTxtRecords;
use openfab::advertise::{IppAdvertiser, IppBroadcastBuilder};

use rocket::tokio;
use tokio::sync::mpsc;

/// can this server handle livekit orchestration for per machine live viewing.
#[cfg(feature = "livekit")]
pub mod livekit;

use ormlite::Model;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub struct VirtualPrinter {}

impl VirtualPrinter {
    pub fn new() -> Self {
        VirtualPrinter {}
    }

    pub fn spawn_advertiser(&self) -> mpsc::Sender<AdvertiseCmd> {
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
                .pdl("test/x.gcode,image/svg+xml,model/stl,model/obj")
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
        tx
    }

    /// Spawns a VirtualPrinter (IPP Server) that listens for print requests as a means of saving files received from 3D Scanners, 2D Scanners, or user devices.
    /// The virtual printer acts as a printer loopback device that captures print jobs and saves them to the library instead of sending them to a physical printer. This allows users to "print" scanned objects directly into their library for easy access and management.
    pub fn spawn_scanner_endpoint(tx: &mpsc::Sender<AdvertiseCmd>) {
        unimplemented!("Scanner endpoint not implemented yet");
    }
}
