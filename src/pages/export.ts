import type { AppState } from "../types";
import { adapters } from "../formatters";
import { resolveOutputFileName } from "../outputFileName";
import { escapeHtml, field, statusMarkup } from "../ui/html";

export function renderExportPage(state: AppState) {
  const isSpot = state.settings.posSystem === "spot";
  const defaultFileName = adapters[state.settings.posSystem].fileName(state.ticket, new Date());
  const outputFileName = resolveOutputFileName(state.settings.outputFileName, defaultFileName);

  return `
    <section class="page export-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">Step 6</p>
          <h1>Write POS File</h1>
          <p>Preview the exact CSV payload before writing it to the selected output folder.</p>
        </div>
        <button class="primary-button" data-action="export">Write Export</button>
      </div>

      ${statusMarkup(state.status, state.statusKind)}

      <div class="form-surface export-config">
        <div class="section-title">
          <h2>Output File</h2>
          <span>Use .csv or .txt; leave blank for the POS default</span>
        </div>
        <div class="form-grid aligned-grid two-column-grid">
          ${field("settings.outputFileName", "File Name", state.settings.outputFileName, `Default: ${defaultFileName}`)}
          <div class="file-name-preview">
            <span>Will write as</span>
            <strong>${escapeHtml(outputFileName || "Invalid file name")}</strong>
          </div>
        </div>
      </div>

      <div class="export-summary">
        <div>
          <span>Customer</span>
          <strong>${escapeHtml(state.customer.firstName)} ${escapeHtml(state.customer.lastName)}</strong>
        </div>
        <div>
          <span>${isSpot ? "Invoice" : "Ticket"}</span>
          <strong>${escapeHtml(isSpot ? state.ticket.displayInvoiceNumber : state.ticket.ticketNumber)}</strong>
        </div>
        <div>
          <span>${isSpot ? "Promised" : "Ready"}</span>
          <strong>${escapeHtml(isSpot ? state.ticket.promisedDateTime : `${state.ticket.readyDate} ${state.ticket.readyTime}`)}</strong>
        </div>
        <div>
          <span>Garments</span>
          <strong>${state.garments.length}</strong>
        </div>
        <div>
          <span>File</span>
          <strong>${escapeHtml(outputFileName || "-")}</strong>
        </div>
      </div>

      <pre class="preview">${escapeHtml(state.preview)}</pre>
    </section>
  `;
}
