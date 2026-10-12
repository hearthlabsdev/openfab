import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { repeat } from 'lit/directives/repeat.js';

export interface PrinterStatus {
  bed_temp: number;
  target_bed_temp?: number | null;
  nozzle_temp: number;
  target_nozzle_temp?: number | null;
  chamber_temp?: number | null;
  [key: string]: any;
}

export interface JobStatus {
  time_remaining_secs: number;
  progress: number;
  state: string;
  [key: string]: any;
}

// Define TypeScript interfaces matching your Rust structs/enums
type Event = 
    | { Connected: string }
    | { Disconnected: string }
    | { PrintStarted: JobStatus }
    | { JobUpdate: JobStatus }
    | { PrintPaused: JobStatus }
    | { PrintResumed: JobStatus }
    | { PrintStopped: [JobStatus, PrintError | null] }
    | { FileUploaded: FileInfo }
    | { StatusUpdate: PrinterStatus }
    | { Err: string };

    @customElement('device-view')
export class DeviceView extends LitElement {
  @property({ type: String, attribute: 'printer-id' }) printerId = '';
  @property({ type: String, attribute: 'stream-url' }) streamUrl = '';
  @property({ type: String, attribute: 'camera-url' }) cameraUrl?: string;

  @state() private printerStatus?: PrinterStatus;
  @state() private jobStatus?: JobStatus;
  @state() private connectionState: 'Connecting' | 'Connected' | 'Disconnected' = 'Connecting';
  @state() private errorMessage?: string = '';

  private ws?: WebSocket;
  private shouldReconnect = true;

  connectedCallback() {
    super.connectedCallback();
    this.shouldReconnect = true;
    this.connectWebSocket();
  }

  disconnectedCallback() {
    this.shouldReconnect = false;
    this.ws?.close();
    super.disconnectedCallback();
  }

  private connectWebSocket() {
    if (!this.streamUrl && !this.printerId) return;
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    
    if (!this.streamUrl) {
      this.streamUrl = `/devices/${this.printerId}/status`;
    }

    const fullUrl = this.streamUrl.startsWith('ws') 
      ? this.streamUrl 
      : `${proto}//${window.location.host}${this.streamUrl}`;

    this.ws = new WebSocket(fullUrl);

    const timeoutMs = 3000; // 3 seconds timeout
    const connectionTimer = setTimeout(() => {
      // If the socket hasn't reached OPEN state (readyState 1) yet, abort it
      if (this.ws.readyState !== WebSocket.OPEN) {
        console.warn(`WebSocket connection timed out after ${timeoutMs}ms`);
        this.shouldReconnect = false; // Stop further retries if desired, or leave true to keep trying
        this.errorMessage = "Connection timed out: Printer stream is unreachable.";
        this.connectionState = 'Disconnected';
        this.ws.close(); // Force close the pending connection
      }
    }, timeoutMs);

    this.ws.onopen = () => {
      clearTimeout(connectionTimer);
      this.connectionState = 'Connected';
      this.errorMessage = '';
    };

    this.ws.onmessage = (event) => {
      const data: Event = JSON.parse(event.data);

      if ('Err' in data) {
        console.error("Backend error received:", data.Err);
        this.shouldReconnect = false; // Stop retrying on fatal backend errors
        this.errorMessage = data.Err;
        this.ws?.close(1000, "Fatal error from backend");
      } else if ('StatusUpdate' in data) {
        this.printerStatus = data.StatusUpdate;
      } else if ('PrintStarted' in data || 'JobUpdate' in data) {
        // Handle job updates as needed based on variant structure
      }
    };

    this.ws.onerror = (error) => {
      console.error("WebSocket connection failed (Device likely not found):", error);
      this.connectionState = 'Disconnected';
      this.errorMessage = "Unable to connect to printer stream (Device not found or offline).";
    };

    this.ws.onclose = () => {
      this.connectionState = 'Disconnected';
      
      // Only retry if we haven't explicitly halted reconnection
      if (this.shouldReconnect) {
        setTimeout(() => this.connectWebSocket(), 3000);
      }
    };
  }

  private formatTime(seconds: number): string {
    if (seconds <= 0) return 'Unknown';
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) return `${hrs}h ${mins}m`;
    return `${mins}m ${secs}s`;
  }

  private getVendorExtras(status: PrinterStatus): [string, any][] {
    const ignoredKeys = ['bed_temp', 'target_bed_temp', 'nozzle_temp', 'target_nozzle_temp', 'chamber_temp'];
    return Object.entries(status).filter(([key]) => !ignoredKeys.includes(key));
  }

  render() {
    return html`

      <div class="printer-dashboard">

        <div class="dashboard-header">
          <h2>Printer: ${this.printerId}</h2>
          <span class="badge badge-${this.connectionState.toLowerCase()}">
            ${this.connectionState}
          </span>
        </div>

      

        <div class="dashboard-grid">

          <div class="camera-container">
            <h3>Live Feed</h3>
            ${this.cameraUrl 
              ? html`<img src="${this.cameraUrl}" alt="Printer Live Camera" class="printer-feed" />`
              : html`<div class="camera-placeholder">Camera unavailable or disabled</div>`
            }
          </div>

          <div class="telemetry-container">
            <div class="card">
              <h3>Job Progress</h3>
              ${this.jobStatus ? html`
                <div class="metric-row">
                  <span>State:</span> <strong>${this.jobStatus.state}</strong>
                </div>
                <div class="metric-row">
                  <span>Progress:</span> <strong>${this.jobStatus.progress}%</strong>
                </div>
                <div class="progress-bar-bg">
                  <div class="progress-bar-fill" style="width: ${this.jobStatus.progress}%"></div>
                </div>
                <div class="metric-row">
                  <span>Time Remaining:</span> <strong>${this.formatTime(this.jobStatus.time_remaining_secs)}</strong>
                </div>

                ${Object.keys(this.jobStatus.extras || {}).length > 0 ? html`
                  <details class="extras-dropdown">
                    <summary>Vendor Job Details</summary>
                    <ul>
                      ${repeat(Object.entries(this.jobStatus.extras), ([k]) => k, ([k, v]) => html`
                        <li><code>${k}</code>:${JSON.stringify(v)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ''}
              ` : html`<p class="muted">Waiting for job data...</p>`}
            </div>

            <div class="card">
              <h3>Temperatures</h3>
              ${this.printerStatus ? html`
                <div class="temp-grid">
                  <div class="temp-box">
                    <span class="label">Nozzle</span>
                    <span class="value">${this.printerStatus.nozzle_temp.toFixed(1)}°C</span>${this.printerStatus.target_nozzle_temp ? html`<span class="target">Target: ${this.printerStatus.target_nozzle_temp}°C</span>` : ''}
                  </div>
                  <div class="temp-box">
                    <span class="label">Bed</span>
                    <span class="value">${this.printerStatus.bed_temp.toFixed(1)}°C</span>${this.printerStatus.target_bed_temp ? html`<span class="target">Target: ${this.printerStatus.target_bed_temp}°C</span>` : ''}
                  </div>
                  ${this.printerStatus.chamber_temp !== undefined && this.printerStatus.chamber_temp !== null ? html`
                    <div class="temp-box">
                      <span class="label">Chamber</span>
                      <span class="value">${this.printerStatus.chamber_temp.toFixed(1)}°C</span>
                    </div>
                  ` : ''}
                </div>

                ${this.getVendorExtras(this.printerStatus).length > 0 ? html`
                  <details class="extras-dropdown">
                    <summary>Hardware Extras</summary>
                    <ul>
                      ${repeat(this.getVendorExtras(this.printerStatus), ([k]) => k, ([k, v]) => html`
                        <li><code>${k}</code>:${JSON.stringify(v)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ''}
              ` : html`<p class="muted">Waiting for temperature telemetry...</p>`}
            </div>
          </div>
        </div>
      </div>
    `;
  }
}