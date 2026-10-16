import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { repeat } from 'lit/directives/repeat.js';


export const styles = css`
  :host {
    display: block;
    width: 100%;
  }

  .printer-dashboard {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px;
    max-width: 1400px;
    margin: 0 auto;
    color: var(--text);
    background-color: var(--background);
    box-sizing: border-box;
  }

  /* Header & Connection Status Badge */
  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    padding-bottom: 16px;
  }

  .dashboard-header h2 {
    margin: 0;
    color: var(--text-strong);
    font-size: 1.5rem;
    font-weight: 600;
  }

  .badge {
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.85rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .badge-connected {
    background-color: color-mix(in srgb, #28a745 20%, var(--surface));
    color: #28a745;
    border: 1px solid color-mix(in srgb, #28a745 40%, transparent);
  }

  .badge-connecting {
    background-color: color-mix(in srgb, #ffc107 20%, var(--surface));
    color: #d39e00;
    border: 1px solid color-mix(in srgb, #ffc107 40%, transparent);
  }

  .badge-disconnected {
    background-color: color-mix(in srgb, #dc3545 20%, var(--surface));
    color: #dc3545;
    border: 1px solid color-mix(in srgb, #dc3545 40%, transparent);
  }

  /* Main Grid Layout */
  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }

  @media (max-width: 1024px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }

  /* Cards & Containers */
  .card, .camera-container {
    background-color: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .card h3, .camera-container h3 {
    margin-top: 0;
    margin-bottom: 16px;
    color: var(--text-strong);
    font-size: 1.1rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 8px;
  }

  /* Camera Stream Box */
  .camera-container {
    display: flex;
    flex-direction: column;
  }

  .printer-feed {
    width: 100%;
    height: auto;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    border-radius: 6px;
    background-color: var(--background);
  }

  .camera-placeholder {
    width: 100%;
    aspect-ratio: 16 / 9;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--background);
    color: var(--text-muted);
    border: 1px dashed var(--border);
    border-radius: 6px;
    font-size: 0.95rem;
  }

  /* Telemetry Stack */
  .telemetry-container {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .metric-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 0.95rem;
  }

  .metric-row span {
    color: var(--text-muted);
  }

  .metric-row strong {
    color: var(--text-strong);
  }

  /* Progress Bar */
  .progress-bar-bg {
    width: 100%;
    height: 8px;
    background-color: var(--background);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 16px;
    border: 1px solid var(--border-subtle);
  }

  .progress-bar-fill {
    height: 100%;
    background-color: var(--accent);
    transition: width 0.3s ease;
  }

  /* Temperature Grid */
  .temp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 12px;
    margin-bottom: 16px;
  }

  .temp-box {
    background-color: var(--background);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .temp-box .label {
    font-size: 0.8rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
  }

  .temp-box .value {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-strong);
  }

  .temp-box .target {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 4px;
  }

  /* Extras Dropdowns */
  .extras-dropdown {
    margin-top: 12px;
    font-size: 0.9rem;
    border-top: 1px solid var(--border-subtle);
    padding-top: 8px;
  }

  .extras-dropdown summary {
    cursor: pointer;
    color: var(--accent);
    font-weight: 500;
  }

  .extras-dropdown ul {
    margin: 8px 0 0 0;
    padding-left: 20px;
    color: var(--text-muted);
  }

  .extras-dropdown li {
    margin-bottom: 4px;
  }

  .extras-dropdown code {
    background-color: var(--background);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-subtle);
    color: var(--text-strong);
  }

  .muted {
    color: var(--text-muted);
    font-style: italic;
    margin: 0;
  }

  .camera-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.printer-feed {
  width: 100%;
  max-width: 480px;
  border-radius: 4px;
}

.printer-controls {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.control-btn {
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 600;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.start-btn {
  background-color: #28a745;
  color: white;
}

.start-btn:hover:not(:disabled) {
  background-color: #218838;
}

.stop-btn {
  background-color: #dc3545;
  color: white;
}

.stop-btn:hover:not(:disabled) {
  background-color: #c82333;
}
`;

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
  static styles = [styles];
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
      console.log("event: " + JSON.stringify(event.data));
      if ('Err' in data) {
        console.error("Backend error received:", data.Err);
        this.shouldReconnect = false; // Stop retrying on fatal backend errors
        this.errorMessage = data.Err;
        this.ws?.close(1000, "Fatal error from backend");
      } else if ('StatusUpdate' in data) {
        this.printerStatus = data.StatusUpdate;
      } else if ('PrintStarted' in data) {
        this.jobStatus = data.PrintStarted;
      } else if ('JobUpdate' in data) {
        this.jobStatus = data.JobUpdate;
      } else if ('PrintPaused' in data) {
        this.jobStatus = data.PrintPaused;
      } else if ('PrintResumed' in data) {
        this.jobStatus = data.PrintResumed;
      } else if ('PrintStopped' in data) {
        // PrintStopped variant carries a tuple: [JobStatus, Option<PrintError>]
        const [job, error] = data.PrintStopped;
        this.jobStatus = job;
        if (error) {
          console.error("Print stopped due to error:", error);
          this.errorMessage = `Print stopped: ${error}`; // Or customize based on your PrintError structure
        }
      } else if ('Connected' in data) {
        console.log("Device connected:", data.Connected);
      } else if ('Disconnected' in data) {
        console.log("Device disconnected:", data.Disconnected);
      } else if ('FileUploaded' in data) {
        console.log("File uploaded:", data.FileUploaded);
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

  private getJobVendorExtras(job: JobStatus): [string, any][] {
    const ignoredKeys = ['time_remaining_secs', 'progress', 'state'];
    return Object.entries(job).filter(([key]) => !ignoredKeys.includes(key));
  }

  render() {
    return html`
      <div class="printer-dashboard page">
        <div class="dashboard-header">
          <h2>Printer: ${this.printerId}</h2>
          <span class="badge badge-${this.connectionState.toLowerCase()}">
            ${this.connectionState}
          </span>
        </div>

        ${this.errorMessage ? html`
          <div class="error-banner" role="alert">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <span>${this.errorMessage}</span>
          </div>
        ` : ''}

        <div class="dashboard-grid">
          <div class="camera-container">
            <h3>Live Feed</h3>
            ${this.cameraUrl 
              ? html`<img src="${this.cameraUrl}" alt="Printer Live Camera" class="printer-feed" />`
              : html`<div class="camera-placeholder">Camera unavailable or disabled</div>`
            }

            <div class="printer-controls">
            <button 
              class="control-btn start-btn" 
              @click="${this.startPrint}"
              ?disabled="${this.isPrinting}">
              Start Print
            </button>
            <button 
              class="control-btn stop-btn" 
              @click="${this.stopPrint}"
              ?disabled="${!this.isPrinting}">
              Stop Print
            </button>
            <button class="btn btn-primary" @click="${this.uploadFile}">
              Upload File
            </button>
          </div>
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

                ${this.getJobVendorExtras(this.jobStatus).length > 0 ? html`
                  <details class="extras-dropdown">
                    <summary>Vendor Job Details</summary>
                    <ul>
                      ${repeat(this.getJobVendorExtras(this.jobStatus), ([k]) => k, ([k, v]) => html`
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