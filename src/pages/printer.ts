import type { AppState, ReceiptPrinterInfo } from "../types";
import { escapeAttribute, escapeHtml, field, statusMarkup } from "../ui/html";

export function renderPrinterPage(state: AppState) {
  return `
    <section class="page printer-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">Receipt Printer</p>
          <h1>Connect Printer</h1>
          <p>Send ESC/POS receipts to a printer queue, USB receipt printer, network printer, or serial device.</p>
        </div>
        <button class="secondary-button" data-action="scan-printers">Scan</button>
      </div>

      ${statusMarkup(state.status, state.statusKind)}

      <div class="form-surface printer-config">
        <div class="section-title">
          <h2>Printer Connection</h2>
          <span>Select a discovered printer or enter a path manually</span>
        </div>
        <div class="form-grid aligned-grid two-column-grid">
          ${printerSelect(state)}
          ${field("settings.receiptPrinterPath", "Printer Path", state.settings.receiptPrinterPath, "Queue name, VID:PID, IP address, or serial path.")}
        </div>
        <div class="action-row">
          <button class="secondary-button" data-action="test-receipt-printer">Test Print</button>
          <button class="primary-button" data-action="print-receipt">Print Receipt</button>
        </div>
      </div>

      <div class="info-band">
        <strong>${escapeHtml(selectedPrinterTitle(state))}</strong>
        <span>${escapeHtml(selectedPrinterDescription(state))}</span>
      </div>
    </section>
  `;
}

function printerSelect(state: AppState) {
  const options = [
    printerOption("", state.receiptPrinters.length === 0 ? "No printers scanned" : "Choose a printer", !state.settings.receiptPrinterPath),
    ...state.receiptPrinters.map((printer) =>
      printerOption(printer.path, printerLabel(printer), printer.path === state.settings.receiptPrinterPath)
    ),
  ];

  return `
    <label class="field selector-field">
      <span>Discovered Printer</span>
      <select data-action="select-receipt-printer">
        ${options.join("")}
      </select>
    </label>
  `;
}

function printerOption(value: string, label: string, selected: boolean) {
  return `
    <option value="${escapeAttribute(value)}" ${selected ? "selected" : ""}>
      ${escapeHtml(label)}
    </option>
  `;
}

function printerLabel(printer: ReceiptPrinterInfo) {
  return `${printer.description} (${printer.path})`;
}

function selectedPrinterTitle(state: AppState) {
  return state.settings.receiptPrinterPath || "No receipt printer selected";
}

function selectedPrinterDescription(state: AppState) {
  const selected = state.receiptPrinters.find((printer) => printer.path === state.settings.receiptPrinterPath);
  if (selected) {
    return selected.description;
  }

  return "Scan for printers or type a connection path manually.";
}
