import type { AppState, Garment, ReceiptPrinterInfo, TicketField } from "../types";
import { escapeAttribute, escapeHtml, field, statusMarkup } from "../ui/html";

const barcodeSupported = new Set(["ticketNumber", "invoiceNumber", "customerIdentifier", "itemList"]);

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
          ${field("settings.receiptPrinterPath", "Printer Path", state.settings.receiptPrinterPath, "Discovered printer, 192.168.192.168, tcp://host[:port], queue name, or COM path.")}
        </div>
        <div class="action-row">
          <button class="secondary-button" data-action="test-receipt-printer">Test Print</button>
          <button class="primary-button" data-action="print-receipt">Print Receipt</button>
        </div>
      </div>

      <div class="form-surface ticket-layout-config">
        <div class="section-title">
          <h2>Ticket Layout</h2>
          <span>Choose what prints, barcode fields, and field order</span>
        </div>

        <div class="ticket-layout-grid">
          <div class="ticket-layout-editor">
            <label class="field compact-field">
              <span>Header Text</span>
              <input data-action="ticket-template-text" data-template-key="headerText" value="${escapeAttribute(state.settings.receiptTicketTemplate.headerText)}" />
            </label>

            <div class="receipt-field-list">
              <div class="receipt-field-head">
                <span>Field</span>
                <span>Barcode</span>
                <span>On/Off</span>
              </div>
              ${state.settings.receiptTicketTemplate.fields.map((ticketField, index, fields) =>
                ticketFieldRow(ticketField, index, fields.length)
              ).join("")}
            </div>

            <label class="field compact-field">
              <span>Footer Text</span>
              <input data-action="ticket-template-text" data-template-key="footerText" value="${escapeAttribute(state.settings.receiptTicketTemplate.footerText)}" />
            </label>
          </div>

          <div class="ticket-preview-panel">
            <div class="panel-title">
              <h2>Preview</h2>
              <span>Receipt output</span>
            </div>
            ${renderReceiptTicketPreview(state)}
          </div>
        </div>
      </div>

      <div class="info-band">
        <strong>${escapeHtml(selectedPrinterTitle(state))}</strong>
        <span>${escapeHtml(selectedPrinterDescription(state))}</span>
      </div>
    </section>
  `;
}

function ticketFieldRow(ticketField: TicketField, index: number, fieldCount: number) {
  const canBarcode = barcodeSupported.has(ticketField.id);
  return `
    <div class="receipt-field-row ${ticketField.enabled ? "" : "disabled"}">
      <div class="receipt-field-moves">
        <button class="field-move-button" data-action="move-ticket-field" data-field-id="${escapeAttribute(ticketField.id)}" data-direction="-1" ${index === 0 ? "disabled" : ""} aria-label="Move ${escapeAttribute(ticketField.label)} up">
          <span class="chevron-icon up"></span>
        </button>
        <button class="field-move-button" data-action="move-ticket-field" data-field-id="${escapeAttribute(ticketField.id)}" data-direction="1" ${index === fieldCount - 1 ? "disabled" : ""} aria-label="Move ${escapeAttribute(ticketField.label)} down">
          <span class="chevron-icon down"></span>
        </button>
      </div>
      <span class="receipt-field-label">${escapeHtml(ticketField.label)}</span>
      <button class="field-icon-toggle ${ticketField.showBarcode ? "active" : ""}" data-action="toggle-ticket-barcode" data-field-id="${escapeAttribute(ticketField.id)}" ${canBarcode ? "" : "disabled"} title="${canBarcode ? "Print barcode" : "Barcode not available for this field"}" aria-label="Toggle barcode for ${escapeAttribute(ticketField.label)}">
        <span class="barcode-icon"></span>
      </button>
      <button class="switch-toggle ${ticketField.enabled ? "active" : ""}" data-action="toggle-ticket-field" data-field-id="${escapeAttribute(ticketField.id)}" aria-label="Toggle ${escapeAttribute(ticketField.label)}"></button>
    </div>
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

export function renderReceiptTicketPreview(state: AppState) {
  const template = state.settings.receiptTicketTemplate;
  const fields = template.fields.filter((ticketField) => ticketField.enabled);

  return `
    <div class="receipt-ticket-preview">
      ${template.headerText.trim() ? `
        <div class="receipt-preview-header">${escapeHtml(template.headerText.trim())}</div>
        <div class="receipt-preview-rule"></div>
      ` : ""}
      ${fields.map((ticketField) => receiptPreviewField(ticketField, state)).join("")}
      ${template.footerText.trim() ? `
        <div class="receipt-preview-rule"></div>
        <div class="receipt-preview-footer">${escapeHtml(template.footerText.trim())}</div>
      ` : ""}
    </div>
  `;
}

function receiptPreviewField(ticketField: TicketField, state: AppState) {
  if (ticketField.id === "itemList") {
    const garments = previewGarments(state.garments);
    if (garments.length === 0) return "";

    return `
      <div class="receipt-preview-group">
        <span>Garments</span>
        ${garments.map((garment) => `
          <p>${escapeHtml(`${garment.id}  ${garment.description}`.trim())}</p>
          ${ticketField.showBarcode && garment.id.trim() ? `<div class="barcode-preview"></div>` : ""}
        `).join("")}
      </div>
    `;
  }

  const value = previewFieldValue(ticketField.id, state);
  if (!value.trim()) return "";

  return `
    <div class="receipt-preview-line">
      <span>${escapeHtml(previewLabel(ticketField.id, ticketField.label))}:</span>
      <strong>${escapeHtml(value)}</strong>
      ${ticketField.showBarcode ? `<div class="barcode-preview"></div>` : ""}
    </div>
  `;
}

function previewFieldValue(id: string, state: AppState) {
  const customerName = `${state.customer.firstName} ${state.customer.lastName}`.trim();
  const readyDate = `${state.ticket.readyDate} ${state.ticket.readyTime}`.trim();
  const itemCount = state.garments.filter((garment) => garment.id.trim() || garment.description.trim()).length;
  const sample: Record<string, string> = {
    customerName: "John Smith",
    customerIdentifier: "01040363",
    customerPhone: "555-0104",
    ticketNumber: "000014684",
    invoiceNumber: "INV-14684",
    balanceDue: "$42.50",
    dropoffDate: "04/03/2026",
    pickupDate: "04/10/2026",
    readyDate: "04/10/2026 5:00 PM",
    numItems: "4 items",
    comments: "Handle with care",
    ticketMessage: "Rack 12, Slot A",
  };
  const values: Record<string, string> = {
    customerName,
    customerIdentifier: state.ticket.customerAccountNumber || state.customer.accountNumber,
    customerPhone: state.customer.phoneNumber,
    ticketNumber: state.ticket.ticketNumber,
    invoiceNumber: state.ticket.displayInvoiceNumber || state.ticket.fullInvoiceNumber,
    balanceDue: state.ticket.balanceDue,
    dropoffDate: state.ticket.dropoffDateTime,
    pickupDate: state.ticket.promisedDateTime,
    readyDate,
    numItems: itemCount > 0 ? `${itemCount} item${itemCount === 1 ? "" : "s"}` : "",
    comments: state.ticket.comments,
    ticketMessage: "",
  };

  return values[id]?.trim() || sample[id] || "";
}

function previewGarments(garments: Garment[]) {
  const current = garments
    .filter((garment) => garment.id.trim() || garment.description.trim())
    .map((garment) => ({
      id: garment.id.trim(),
      description: garment.description.trim(),
    }));

  if (current.length > 0) {
    return current;
  }

  return [
    { id: "T1476237", description: "Ld Bag" },
    { id: "T2003925", description: "LD Shirt Hanger" },
    { id: "T1428942", description: "LD Shirt Hanger" },
    { id: "T1444476", description: "LD Shirt Hanger" },
  ];
}

function previewLabel(id: string, fallback: string) {
  const labels: Record<string, string> = {
    customerName: "Customer",
    customerIdentifier: "Account",
    customerPhone: "Phone",
    ticketNumber: "Ticket",
    invoiceNumber: "Invoice",
    balanceDue: "Balance",
    dropoffDate: "Dropoff",
    pickupDate: "Promised",
    readyDate: "Ready",
    numItems: "Items",
    comments: "Notes",
    ticketMessage: "Message",
  };

  return labels[id] || fallback;
}
