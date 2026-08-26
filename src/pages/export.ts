import type { AppState } from "../types";
import { adapters } from "../formatters";
import { resolveOutputFileName } from "../outputFileName";
import { escapeAttribute, escapeHtml, field, statusMarkup } from "../ui/html";

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
          ${state.settings.posSystem === "whiteconveyors" ? transactionField(state) : ""}
          ${state.settings.posSystem === "whiteconveyors" ? deleteTargetField(state) : ""}
          ${field("settings.outputFileName", "File Name", state.settings.outputFileName, `Default: ${defaultFileName}`)}
          <div class="file-name-preview">
            <span>Will write as</span>
            <strong>${escapeHtml(outputFileName || "Invalid file name")}</strong>
          </div>
        </div>
      </div>

      <div class="export-summary">
        <div>
          <span>Transaction</span>
          <strong>${escapeHtml(transactionLabel(state.settings.exportOperation))}</strong>
        </div>
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

function transactionField(state: AppState) {
  return `
    <label class="field selector-field">
      <span>Transaction</span>
      <select data-action="select-export-operation">
        ${transactionOption("create", "Create / Update Records", state.settings.exportOperation)}
        ${transactionOption("customerDelete", "Delete Customer", state.settings.exportOperation)}
        ${transactionOption("ticketDelete", "Delete Ticket", state.settings.exportOperation)}
        ${transactionOption("garmentDelete", "Delete Garment", state.settings.exportOperation)}
        ${transactionOption("employeeDelete", "Delete Employee", state.settings.exportOperation)}
      </select>
    </label>
  `;
}

function deleteTargetField(state: AppState) {
  if (state.settings.exportOperation === "customerDelete") {
    return selectField("Customer to Delete", "select-delete-customer", customerDeleteOptions(state));
  }

  if (state.settings.exportOperation === "ticketDelete") {
    return selectField("Ticket to Delete", "select-delete-ticket", ticketDeleteOptions(state));
  }

  if (state.settings.exportOperation === "garmentDelete") {
    return selectField("Garment to Delete", "select-delete-garment", garmentDeleteOptions(state));
  }

  if (state.settings.exportOperation === "employeeDelete") {
    return selectField("Employee to Delete", "select-delete-employee", employeeDeleteOptions(state));
  }

  return "";
}

function selectField(label: string, action: string, options: string) {
  return `
    <label class="field selector-field">
      <span>${label}</span>
      <select data-action="${escapeAttribute(action)}">
        ${options}
      </select>
    </label>
  `;
}

function customerDeleteOptions(state: AppState) {
  const rows = state.databaseSummary?.customers ?? [];
  const selected = state.customer.accountNumber;
  const options = rows.map((customer) => {
    const name = customer.name.trim();
    const label = name ? `${customer.accountNumber} - ${name}` : customer.accountNumber;
    return genericOption(customer.accountNumber, label, customer.accountNumber === selected);
  });

  return [
    genericOption("", rows.length === 0 ? "No customers in database" : "Choose a customer", !selected),
    ...options,
  ].join("");
}

function ticketDeleteOptions(state: AppState) {
  const rows = state.databaseSummary?.tickets ?? [];
  const selected = state.ticket.ticketNumber;
  const options = rows.map((ticket) => {
    const display = ticket.displayInvoiceNumber || ticket.fullInvoiceNumber || ticket.ticketNumber;
    const label = `${ticket.customerAccountNumber} / ${display}`;
    return genericOption(ticket.ticketNumber, label, ticket.ticketNumber === selected);
  });

  return [
    genericOption("", rows.length === 0 ? "No tickets in database" : "Choose a ticket", !selected),
    ...options,
  ].join("");
}

function garmentDeleteOptions(state: AppState) {
  const rows = state.databaseSummary?.garments ?? [];
  const selected = state.garments[state.selectedGarmentIndex];
  const selectedValue = selected ? `${state.ticket.ticketNumber}::${selected.id}` : "";
  const options = rows.map((garment) => {
    const value = `${garment.ticketNumber}::${garment.garmentId}`;
    const label = `${garment.ticketNumber} / ${garment.garmentId}${garment.description ? ` - ${garment.description}` : ""}`;
    return genericOption(value, label, value === selectedValue);
  });

  return [
    genericOption("", rows.length === 0 ? "No garments in database" : "Choose a garment", !selectedValue),
    ...options,
  ].join("");
}

function employeeDeleteOptions(state: AppState) {
  const rows = state.databaseSummary?.employees ?? [];
  const selected = state.employees[state.selectedEmployeeIndex]?.employeeNumber ?? "";
  const options = rows.map((employee) => {
    const label = employee.employeeName ? `${employee.employeeNumber} - ${employee.employeeName}` : employee.employeeNumber;
    return genericOption(employee.employeeNumber, label, employee.employeeNumber === selected);
  });

  return [
    genericOption("", rows.length === 0 ? "No employees in database" : "Choose an employee", !selected),
    ...options,
  ].join("");
}

function genericOption(value: string, label: string, selected: boolean) {
  return `
    <option value="${escapeAttribute(value)}" ${selected ? "selected" : ""}>
      ${escapeHtml(label)}
    </option>
  `;
}

function transactionOption(value: AppState["settings"]["exportOperation"], label: string, selected: string) {
  return `
    <option value="${escapeAttribute(value)}" ${value === selected ? "selected" : ""}>
      ${escapeHtml(label)}
    </option>
  `;
}

function transactionLabel(operation: AppState["settings"]["exportOperation"]) {
  if (operation === "customerDelete") return "CUSTOMER_DELETE";
  if (operation === "ticketDelete") return "TICKET_DELETE";
  if (operation === "garmentDelete") return "GARMENT_DELETE";
  if (operation === "employeeDelete") return "EMPLOYEE_DELETE";
  return "Create / Update";
}
