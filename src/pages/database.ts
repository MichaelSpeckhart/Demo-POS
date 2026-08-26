import type {
  AppState,
  CustomerRecord,
  DatabaseSummary,
  EmployeeRecord,
  ExportRecord,
  GarmentRecord,
  TicketRecord,
} from "../types";
import { escapeAttribute, escapeHtml, pageHeader, statusMarkup } from "../ui/html";

export function renderDatabasePage(state: AppState) {
  return `
    <section class="page database-page">
      ${pageHeader(
        "Storage",
        "SQLite Database",
        "Employees, customers, tickets, garments, and export history are persisted locally."
      )}

      ${statusMarkup(state.status, state.statusKind)}
      ${state.databaseSummary ? summaryMarkup(state.databaseSummary) : emptyMarkup()}
    </section>
  `;
}

function summaryMarkup(summary: DatabaseSummary) {
  return `
    <div class="database-layout">
      <section class="db-panel">
        <div class="section-title">
          <h2>Overview</h2>
          <span>Current record counts</span>
        </div>
        <div class="db-stats">
          ${stat("Customers", summary.customerCount)}
          ${stat("Employees", summary.employeeCount)}
          ${stat("Tickets", summary.ticketCount)}
          ${stat("Garments", summary.garmentCount)}
          ${stat("Exports", summary.exportCount)}
        </div>
      </section>

      <section class="db-panel">
        <div class="section-title">
          <h2>Storage</h2>
          <span>Local SQLite file and most recent write</span>
        </div>
        <div class="db-path-grid">
          <div class="db-path">
            <span>Database file</span>
            <code>${escapeHtml(summary.path)}</code>
          </div>
          <div class="db-path">
            <span>Last export</span>
            <code>${escapeHtml(summary.lastExportPath ?? "No exports yet")}</code>
            ${summary.lastExportAt ? `<small>Recorded at epoch ${escapeHtml(summary.lastExportAt)}</small>` : ""}
          </div>
        </div>
      </section>

      <section class="db-panel">
        <div class="section-title">
          <h2>Operational Records</h2>
          <span>Editable source data used for POS exports</span>
        </div>
        <div class="db-tables">
          ${customerTable(summary.customers)}
          ${employeeTable(summary.employees)}
          ${ticketTable(summary.tickets)}
          ${garmentTable(summary.garments)}
        </div>
      </section>

      <section class="db-panel">
        <div class="section-title">
          <h2>Export History</h2>
          <span>Payloads written to disk</span>
        </div>
        ${exportTable(summary.exports)}
      </section>
    </div>
  `;
}

function stat(label: string, value: number) {
  return `
    <div>
      <span>${label}</span>
      <strong>${value}</strong>
    </div>
  `;
}

function customerTable(rows: CustomerRecord[]) {
  return `
    <div class="db-section">
      <div class="db-section-heading">
        <h2>Customers</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyTable() : `
        <div class="db-table-wrap">
          <table class="db-table">
            <thead>
              <tr>
                <th>Account</th>
                <th>Name</th>
                <th>Phone</th>
                <th>City</th>
                <th>State</th>
                <th>Updated</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${escapeHtml(row.accountNumber)}</td>
                  <td>${escapeHtml(row.name.trim() || "-")}</td>
                  <td>${escapeHtml(row.phoneNumber || "-")}</td>
                  <td>${escapeHtml(row.city || "-")}</td>
                  <td>${escapeHtml(row.state || "-")}</td>
                  <td>${escapeHtml(row.updatedAt || "-")}</td>
                  <td>
                    <button type="button" class="danger-button" data-action="delete-customer" data-account-number="${escapeAttribute(row.accountNumber)}">
                      Delete
                    </button>
                  </td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </div>
  `;
}

function employeeTable(rows: EmployeeRecord[]) {
  return `
    <div class="db-section">
      <div class="db-section-heading">
        <h2>Employees</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyTable() : `
        <div class="db-table-wrap">
          <table class="db-table">
            <thead>
              <tr>
                <th>Employee</th>
                <th>Name</th>
                <th>Updated</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${escapeHtml(row.employeeNumber)}</td>
                  <td>${escapeHtml(row.employeeName || "-")}</td>
                  <td>${escapeHtml(row.updatedAt || "-")}</td>
                  <td>
                    <button type="button" class="danger-button" data-action="delete-employee" data-employee-number="${escapeAttribute(row.employeeNumber)}">
                      Delete
                    </button>
                  </td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </div>
  `;
}

function ticketTable(rows: TicketRecord[]) {
  return `
    <div class="db-section">
      <div class="db-section-heading">
        <h2>Tickets</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyTable() : `
        <div class="db-table-wrap">
          <table class="db-table">
            <thead>
              <tr>
                <th>Ticket</th>
                <th>Customer</th>
                <th>Full Invoice</th>
                <th>Display Invoice</th>
                <th>Promised</th>
                <th>Ready</th>
                <th>Updated</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${escapeHtml(row.ticketNumber)}</td>
                  <td>${escapeHtml(row.customerAccountNumber)}</td>
                  <td>${escapeHtml(row.fullInvoiceNumber || "-")}</td>
                  <td>${escapeHtml(row.displayInvoiceNumber || "-")}</td>
                  <td>${escapeHtml(row.promisedDateTime || "-")}</td>
                  <td>${escapeHtml(`${row.readyDate} ${row.readyTime}`.trim() || "-")}</td>
                  <td>${escapeHtml(row.updatedAt || "-")}</td>
                  <td>
                    <button type="button" class="danger-button" data-action="delete-ticket" data-ticket-number="${escapeAttribute(row.ticketNumber)}">
                      Delete
                    </button>
                  </td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </div>
  `;
}

function garmentTable(rows: GarmentRecord[]) {
  return `
    <div class="db-section">
      <div class="db-section-heading">
        <h2>Garments</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyTable() : `
        <div class="db-table-wrap">
          <table class="db-table">
            <thead>
              <tr>
                <th>Ticket</th>
                <th>Garment</th>
                <th>Description</th>
                <th>Slot</th>
                <th>Service</th>
                <th>Type</th>
                <th>Color</th>
                <th>Fabric</th>
                <th>Position</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${escapeHtml(row.ticketNumber)}</td>
                  <td>${escapeHtml(row.garmentId)}</td>
                  <td>${escapeHtml(row.description || "-")}</td>
                  <td>${escapeHtml(row.slotOccupancy || "-")}</td>
                  <td>${escapeHtml(row.serviceType || "-")}</td>
                  <td>${escapeHtml(row.garmentType || "-")}</td>
                  <td>${escapeHtml(row.color || "-")}</td>
                  <td>${escapeHtml(row.fabric || "-")}</td>
                  <td>${row.position}</td>
                  <td>
                    <button
                      type="button"
                      class="danger-button"
                      data-action="delete-garment"
                      data-ticket-number="${escapeAttribute(row.ticketNumber)}"
                      data-garment-id="${escapeAttribute(row.garmentId)}"
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </div>
  `;
}

function exportTable(rows: ExportRecord[]) {
  return `
    <div class="db-section">
      <div class="db-section-heading">
        <h2>Exports</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyTable() : `
        <div class="db-table-wrap">
          <table class="db-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>POS</th>
                <th>Ticket</th>
                <th>File</th>
                <th>Created</th>
                <th>Payload</th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${row.id}</td>
                  <td>${escapeHtml(row.posSystem)}</td>
                  <td>${escapeHtml(row.ticketNumber)}</td>
                  <td><code>${escapeHtml(row.filePath)}</code></td>
                  <td>${escapeHtml(row.createdAt)}</td>
                  <td><pre class="db-payload">${escapeHtml(row.payload)}</pre></td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </div>
  `;
}

function emptyTable() {
  return `<div class="empty-table">No records yet.</div>`;
}

function emptyMarkup() {
  return `
    <div class="info-band">
      <strong>Database initializing</strong>
      <span>The SQLite database will be created in the app data directory.</span>
    </div>
  `;
}
