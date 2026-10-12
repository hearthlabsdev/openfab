# Drivers Directory

This directory contains the core crates and modular driver packages designed to provide a unified, agnostic API and SDK surface for interacting with diverse manufacturing and fabrication hardware backends.

## Architectural Philosophy

To prevent generic contagion across event loops while handling wildly varying vendor telemetry (from Prusa's transfer states to Bambu's AMS payloads), this architecture uses a **Core + Vendor Extension Model**:
* **Agnostic Core:** Universal telemetry (temperatures, job progress, operational states) lives in strict, non-generic core structs.
* **Encapsulated Extensions:** Vendor-specific properties are captured securely via private, flattened JSON maps and exposed safely through type-checked extension methods (`.extension::<Ext>()`).

---

## Supported Hardware Backends

### 1. 3D Printing Backends
The primary development focus. The infrastructure is well-documented and offers a robust testing ground for standardizing printer telemetry.
* **Prusa Link** (REST/HTTP integration)
* **Bambu Lab** (MQTT/Cloud/Local telemetry)
* **Marlin / OctoPrint** 
* **Klipper / Moonraker** (WebSocket/REST integration)

#### Standardized 3D Printing Features
* **Thermal Management:** Unified bed, nozzle, and chamber temperature reporting and targets.
* **Multi-Material System (MMS):** A brand-agnostic industry standard abstracting multi-spool switching hubs (unifying concepts like Prusa MMU, Bambu AMS, Creality CFS, and Anycubic ACE Pro).
* **Job & Upload Tracking:** Normalized progress percentages, time remaining estimates, and file transfer metrics.

### 2. Vinyl Cutting
* **Cricut** and compatible plotters.

### 3. Laser Cutting
* *Backends under evaluation.*

### 4. Commercial Embroidery
* *Multi-needle machine protocol integration.*

### 5. CNC Routing and Milling
* *G-code / Controller stream integrations.*

---

## Project Overview & Roadmap

The immediate priority is stabilizing the **3D printing ecosystem**, as its telemetry standards provide a reliable baseline for building out the core SDK. Once established, support will systematically expand into laser cutters, vinyl cutters, embroidery machines, and CNC routers.

**The Ultimate Goal:** Simplify the administration of community makerspaces and fabrication labs. By bridging fragmented vendor silos into a single, cohesive API, this project aims to lower the barrier to entry for local machine-aided manufacturing and foster deeper community investment.