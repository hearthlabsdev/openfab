use crate::devices::PrinterDiscoveryEventORM;
use crate::errors::OpenFabErr;
use openfab::IppTxtRecords;
use openfab::discovery::IppDiscovery;
use openfab::discovery::verify::verify_ipp_printer;
use openfab::utils::unix_now;
use ormlite::Model;
use ormlite::postgres::PgPool;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

async fn mark_queue_advertised(pool: &PgPool, uid: Uuid) {}

/// Commands sent to the mDNS advertiser task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvertiseCmd {
    Advertise {
        instance: String,
        hostname: String,
        txt: IppTxtRecords,
        secure: bool,
    },
}

pub struct DiscoveryRouter {
    pool: PgPool,
    advertiser: Sender<AdvertiseCmd>,
}

impl DiscoveryRouter {
    /// Start the discovery → verification → classification → advertise loop.
    ///
    /// Responsibilities:
    /// - Browse DNS-SD for IPP/IPPS services
    /// - Attempt IPP handshake (Get-Printer-Attributes)
    /// - Ignore unreachable or invalid printers
    /// - Persist printers and discovery events
    /// - Create capability queues if missing
    /// - Trigger advertisement for new queues
    pub async fn run_discovery_router(
        pool: PgPool,
        advertise_tx: Sender<AdvertiseCmd>,
    ) -> Result<(), OpenFabErr> {
        let discovery = IppDiscovery::new()?;
        discovery.start()?;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            for discovered in discovery.printers() {
                let mut conn = pool.acquire().await?;
                // 1. Attempt IPP verification (authoritative step)
                let info = match verify_ipp_printer(&discovered).await {
                    Ok(info) => info,
                    Err(err) => {
                        let event = PrinterDiscoveryEventORM {
                            uid: Uuid::new_v4(),
                            mdns_instance: discovered.name.clone(),
                            host: discovered.host.clone(),
                            discovered_at: unix_now(),
                            port: discovered.port.into(),
                            txt: Some(format!("{:?}", discovered.txt)),
                            success: true,
                            error: Some(err.to_string()),
                        };
                        event.insert(&mut *conn).await?;

                        // Soft failure: discovery is ephemeral
                        tracing::debug!(
                            printer = %discovered.name,
                            "IPP verification failed: {err}"
                        );
                        continue;
                    }
                };

                // 2. Persist / update printer record
                /*let printer_uid = upsert_printer(&pool, &discovered, &info).await?;

                // 3. Classify printer based on verified attributes
                let Some(class) = classify_verified(&info) else {
                    tracing::debug!(
                        printer = %discovered.name,
                        "Unable to classify printer"
                    );
                    continue;
                };

                let rp = class.rp();
                let ty = class.ty();

                // 4. Ensure queue exists (capability-level)
                let queue = ensure_queue(&pool, &rp, &ty).await?;

                // 5. Ensure printer is a member of the queue
                ensure_membership(&pool, printer_uid, queue.uid).await?;

                // 6. Advertise queue if newly created or newly active
                if queue.advertised == false {
                    let txt = IppTxtRecords::new()
                        .rp(&queue.rp)
                        .ty(&queue.name)
                        .product("(OpenMPD Virtual Queue)")
                        .pdl(info.pdl.join(","))
                        .note("Automatically generated capability queue");

                    advertise_tx
                        .send(AdvertiseCmd::Advertise {
                            instance: format!("OpenMPD {}", queue.name),
                            hostname: "openmpd.local.".to_string(),
                            txt,
                            secure: info.secure,
                        })
                        .await?;

                    mark_queue_advertised(&pool, queue.uid).await?;
                }*/
            }
        }
    }
}
