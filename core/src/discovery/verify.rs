//! This module provides verification for discovered IppPrinters

use crate::discovery::{IppDiscoveryError, IppPrinter};
use crate::ipputils::*;
use ipp::prelude::*;
use serde_derive::{Deserialize, Serialize};

/// Structured info about a verified printer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IppPrinterInfo {
    /// the name of the discovered printer
    pub name: String,
    /// the host that runs this printer (hostname)
    pub host: String,
    /// the port on which the IPP server is running.
    pub port: u16,
    /// the printer model/type
    pub ty: Option<String>,
    /// the accepted printer domain languages, eg application/pdf, image/jpg, image/png etc
    pub pdl: Vec<String>,
    /// further printer product information
    pub product: Option<String>,
    /// the raw text records
    pub txt: Option<String>,
}

/// Attempt to verify an IPP printer by performing a Get-Printer-Attributes request.
/// On failure (network, protocol, timeout, invalid response) returns an error.
pub async fn verify_ipp_printer(printer: &IppPrinter) -> Result<IppPrinterInfo, IppDiscoveryError> {
    // Construct IPP URI
    let scheme = if printer.secure { "ipps" } else { "ipp" };
    let uri: Uri = format!(
        "{}://{}:{}/{}",
        scheme,
        printer.host,
        printer.port,
        printer
            .records()
            .path()
            .ok_or(IppDiscoveryError::MissingRoute)?
    )
    .parse()?;

    // Create IPP client
    let client = AsyncIppClient::new(uri.clone());

    let operation = IppOperationBuilder::get_printer_attributes(uri.clone()).build();
    // Get printer attributes
    let attrs: AttrMap = client.send(operation).await?.attributes().into();

    // Extract relevant fields
    let printer_info = IppPrinterInfo {
        name: attrs
            .get_str(DelimiterTag::PrinterAttributes, "printer-name")
            .ok_or(IppDiscoveryError::MissingAttribute(
                "printer-name".to_string(),
            ))?
            .to_string(),
        host: printer.host.clone(),
        port: printer.port,
        ty: attrs
            .get_str(DelimiterTag::PrinterAttributes, "printer-type")
            .map(|s| s.to_string()),
        pdl: attrs
            .get_strings(DelimiterTag::PrinterAttributes, "document-format-supported")
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        product: attrs
            .get(DelimiterTag::PrinterAttributes, "printer-make-and-model")
            .map(|s| s.to_string()),
        txt: Some(format!("{:?}", attrs)), // optionally store raw attrs
    };

    Ok(printer_info)
}

/*
/// Classify a verified printer into a logical queue/type
pub fn classify_verified(info: &IppPrinterInfo) -> Option<PrinterClass> {
    // Minimal example: classify based on product string
    let ty = info.ty.clone().or_else(|| {
        info.product.as_ref().map(|p| {
            if p.to_lowercase().contains("laser") {
                "laser-printer".to_string()
            } else if p.to_lowercase().contains("3d") {
                "3d-printer".to_string()
            } else {
                "generic-printer".to_string()
            }
        })
    })?;

    Some(PrinterClass::new(ty, format!("Queue for {}", info.name)))
}

/// Logical printer class representation
#[derive(Debug, Clone)]
pub struct PrinterClass {
    pub ty: String,
    pub name: String,
}

impl PrinterClass {
    pub fn new(ty: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            ty: ty.into(),
            name: name.into(),
        }
    }

    pub fn rp(&self) -> &str {
        &self.ty
    }

    pub fn ty(&self) -> &str {
        &self.ty
    }
}
*/
