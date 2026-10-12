import { LitElement as e, html as t } from "lit";
import { customElement as n, property as r, state as i } from "lit/decorators.js";
import { repeat as a } from "lit/directives/repeat.js";
//#region \0rolldown/runtime.js
var o = (e, t, n) => () => {
	if (n) throw n[0];
	try {
		return e && (t = e(e = 0)), t;
	} catch (e) {
		throw n = [e], e;
	}
}, s = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports);
//#endregion
//#region \0@oxc-project+runtime@0.153.0/helpers/esm/decorate.js
function c(e, t, n, r) {
	var i = arguments.length, a = i < 3 ? t : r === null ? r = Object.getOwnPropertyDescriptor(t, n) : r, o;
	if (typeof Reflect == "object" && typeof Reflect.decorate == "function") a = Reflect.decorate(e, t, n, r);
	else for (var s = e.length - 1; s >= 0; s--) (o = e[s]) && (a = (i < 3 ? o(a) : i > 3 ? o(t, n, a) : o(t, n)) || a);
	return i > 3 && a && Object.defineProperty(t, n, a), a;
}
var l = o((() => {})), u, d = o((() => {
	l(), u = class extends e {
		constructor(...e) {
			super(...e), this.printerId = "", this.streamUrl = "", this.connectionState = "Connecting", this.errorMessage = "", this.shouldReconnect = !0;
		}
		connectedCallback() {
			super.connectedCallback(), this.shouldReconnect = !0, this.connectWebSocket();
		}
		disconnectedCallback() {
			this.shouldReconnect = !1, this.ws?.close(), super.disconnectedCallback();
		}
		connectWebSocket() {
			if (!this.streamUrl && !this.printerId) return;
			let e = window.location.protocol === "https:" ? "wss:" : "ws:";
			this.streamUrl ||= `/devices/${this.printerId}/status`;
			let t = this.streamUrl.startsWith("ws") ? this.streamUrl : `${e}//${window.location.host}${this.streamUrl}`;
			this.ws = new WebSocket(t);
			let n = 3e3, r = setTimeout(() => {
				this.ws.readyState !== WebSocket.OPEN && (console.warn(`WebSocket connection timed out after ${n}ms`), this.shouldReconnect = !1, this.errorMessage = "Connection timed out: Printer stream is unreachable.", this.connectionState = "Disconnected", this.ws.close());
			}, n);
			this.ws.onopen = () => {
				clearTimeout(r), this.connectionState = "Connected", this.errorMessage = "";
			}, this.ws.onmessage = (e) => {
				let t = JSON.parse(e.data);
				"Err" in t ? (console.error("Backend error received:", t.Err), this.shouldReconnect = !1, this.errorMessage = t.Err, this.ws?.close(1e3, "Fatal error from backend")) : "StatusUpdate" in t ? this.printerStatus = t.StatusUpdate : "PrintStarted" in t || "JobUpdate" in t;
			}, this.ws.onerror = (e) => {
				console.error("WebSocket connection failed (Device likely not found):", e), this.connectionState = "Disconnected", this.errorMessage = "Unable to connect to printer stream (Device not found or offline).";
			}, this.ws.onclose = () => {
				this.connectionState = "Disconnected", this.shouldReconnect && setTimeout(() => this.connectWebSocket(), 3e3);
			};
		}
		formatTime(e) {
			if (e <= 0) return "Unknown";
			let t = Math.floor(e / 3600), n = Math.floor(e % 3600 / 60), r = e % 60;
			return t > 0 ? `${t}h ${n}m` : `${n}m ${r}s`;
		}
		getVendorExtras(e) {
			let t = [
				"bed_temp",
				"target_bed_temp",
				"nozzle_temp",
				"target_nozzle_temp",
				"chamber_temp"
			];
			return Object.entries(e).filter(([e]) => !t.includes(e));
		}
		render() {
			return t`

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
            ${this.cameraUrl ? t`<img src="${this.cameraUrl}" alt="Printer Live Camera" class="printer-feed" />` : t`<div class="camera-placeholder">Camera unavailable or disabled</div>`}
          </div>

          <div class="telemetry-container">
            <div class="card">
              <h3>Job Progress</h3>
              ${this.jobStatus ? t`
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

                ${Object.keys(this.jobStatus.extras || {}).length > 0 ? t`
                  <details class="extras-dropdown">
                    <summary>Vendor Job Details</summary>
                    <ul>
                      ${a(Object.entries(this.jobStatus.extras), ([e]) => e, ([e, n]) => t`
                        <li><code>${e}</code>:${JSON.stringify(n)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ""}
              ` : t`<p class="muted">Waiting for job data...</p>`}
            </div>

            <div class="card">
              <h3>Temperatures</h3>
              ${this.printerStatus ? t`
                <div class="temp-grid">
                  <div class="temp-box">
                    <span class="label">Nozzle</span>
                    <span class="value">${this.printerStatus.nozzle_temp.toFixed(1)}°C</span>${this.printerStatus.target_nozzle_temp ? t`<span class="target">Target: ${this.printerStatus.target_nozzle_temp}°C</span>` : ""}
                  </div>
                  <div class="temp-box">
                    <span class="label">Bed</span>
                    <span class="value">${this.printerStatus.bed_temp.toFixed(1)}°C</span>${this.printerStatus.target_bed_temp ? t`<span class="target">Target: ${this.printerStatus.target_bed_temp}°C</span>` : ""}
                  </div>
                  ${this.printerStatus.chamber_temp !== void 0 && this.printerStatus.chamber_temp !== null ? t`
                    <div class="temp-box">
                      <span class="label">Chamber</span>
                      <span class="value">${this.printerStatus.chamber_temp.toFixed(1)}°C</span>
                    </div>
                  ` : ""}
                </div>

                ${this.getVendorExtras(this.printerStatus).length > 0 ? t`
                  <details class="extras-dropdown">
                    <summary>Hardware Extras</summary>
                    <ul>
                      ${a(this.getVendorExtras(this.printerStatus), ([e]) => e, ([e, n]) => t`
                        <li><code>${e}</code>:${JSON.stringify(n)}</li>
                      `)}
                    </ul>
                  </details>
                ` : ""}
              ` : t`<p class="muted">Waiting for temperature telemetry...</p>`}
            </div>
          </div>
        </div>
      </div>
    `;
		}
	}, c([r({
		type: String,
		attribute: "printer-id"
	})], u.prototype, "printerId", void 0), c([r({
		type: String,
		attribute: "stream-url"
	})], u.prototype, "streamUrl", void 0), c([r({
		type: String,
		attribute: "camera-url"
	})], u.prototype, "cameraUrl", void 0), c([i()], u.prototype, "printerStatus", void 0), c([i()], u.prototype, "jobStatus", void 0), c([i()], u.prototype, "connectionState", void 0), c([i()], u.prototype, "errorMessage", void 0), u = c([n("device-view")], u);
})), f = /* @__PURE__ */ s((() => {
	d();
}));
//#endregion
export default f();
