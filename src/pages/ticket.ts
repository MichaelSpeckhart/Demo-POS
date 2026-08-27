import type { AppState } from "../types";
import { customerReadyForExport, ticketReadyForExport } from "../exportReadiness";
import { escapeAttribute, escapeHtml, field } from "../ui/html";

export function renderTicketPage(state: AppState) {
  const isSpot = state.settings.posSystem === "spot";
  const isWhiteConveyors = state.settings.posSystem === "whiteconveyors";

  return `
    <section class="page ticket-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">Step 4</p>
          <h1>${isSpot ? "Create Invoice" : "Create Ticket"}</h1>
          <p>${isSpot
            ? "SPOT invoice fields are repeated on each ADDITEM row."
            : isWhiteConveyors
              ? "These fields map directly to the Comp-U-Sort TICKET_CREATE row."
              : "These fields map directly to the WinCleaners TICKET_CREATE row."}</p>
        </div>
        <button class="secondary-button" data-action="add-ticket">Create New ${isSpot ? "Invoice" : "Ticket"}</button>
      </div>

      ${state.ticketDraftActive ? ticketForm(state, isSpot, isWhiteConveyors) : emptyTicket(isSpot)}
    </section>
  `;
}

function ticketForm(state: AppState, isSpot: boolean, isWhiteConveyors: boolean) {
  const label = isSpot ? "Invoice" : "Ticket";
  const customerReady = state.customerAddedToExport && customerReadyForExport(state.customer, state.settings.posSystem);
  const ready = customerReady && ticketReadyForExport(state.ticket, state.settings.posSystem);
  const added = state.ticketAddedToExport && ready;

  return `
    <div class="form-surface">
        <div class="section-title">
          <h2>Assignment</h2>
          <span>Connect this ticket to an existing customer</span>
        </div>
        <label class="field selector-field">
          <span>Customer for this ticket</span>
          <select data-action="select-customer">
            ${customerOptions(state)}
          </select>
        </label>

        ${isSpot ? renderSpotTicketSections(state) : renderTicketSections(state, isWhiteConveyors)}
        <div class="add-to-export-row">
          <button class="${ready && !added ? "primary-button" : "secondary-button"} add-to-export-button" data-action="add-ticket-to-export" ${ready && !added ? "" : "disabled"}>
            ${added ? `${label} Added to Export` : !customerReady ? "Add Customer First" : ready ? `Add ${label} to Export` : "Complete Required Fields"}
          </button>
        </div>
      </div>
  `;
}

function customerOptions(state: AppState) {
  const selectedAccount = state.ticket.customerAccountNumber || state.customer.accountNumber;
  const records = state.databaseSummary?.customers ?? [];
  const seen = new Set<string>();
  const options = records.map((customer) => {
    seen.add(customer.accountNumber);
    const label = `${customer.accountNumber} - ${customer.name.trim() || "Unnamed customer"}`;
    return option(customer.accountNumber, label, customer.accountNumber === selectedAccount);
  });

  if (state.customer.accountNumber && !seen.has(state.customer.accountNumber)) {
    options.unshift(
      option(
        state.customer.accountNumber,
        `${state.customer.accountNumber} - ${state.customer.firstName} ${state.customer.lastName}`.trim(),
        state.customer.accountNumber === selectedAccount
      )
    );
  }

  options.unshift(option("", "No customer selected", !selectedAccount));
  return options.join("");
}

function option(value: string, label: string, selected: boolean) {
  return `
    <option value="${escapeAttribute(value)}" ${selected ? "selected" : ""}>
      ${escapeHtml(label)}
    </option>
  `;
}

function renderSpotTicketSections(state: AppState) {
  return `
    <div class="section-title">
      <h2>Invoice</h2>
      <span>Identifiers used by SPOT ADDITEM rows</span>
    </div>
    <div class="form-grid aligned-grid">
      ${field("ticket.fullInvoiceNumber", "Full Invoice Number", state.ticket.fullInvoiceNumber, "Used as the unique invoice barcode value.")}
      ${field("ticket.displayInvoiceNumber", "Display Invoice Number", state.ticket.displayInvoiceNumber)}
      ${field("ticket.ticketNumber", "Ticket Number", state.ticket.ticketNumber, "Kept for file naming and non-SPOT exports.")}
      ${field("ticket.balanceDue", "Balance Due", state.ticket.balanceDue)}
    </div>

    <div class="section-title">
      <h2>Schedule and Route</h2>
      <span>Dates, comments, and delivery route values</span>
    </div>
    <div class="form-grid aligned-grid">
      ${field("ticket.dropoffDateTime", "Dropoff Date/Time", state.ticket.dropoffDateTime, "Local ISO8601, for example 2025-04-18T14:27:49.")}
      ${field("ticket.promisedDateTime", "Promised Date/Time", state.ticket.promisedDateTime, "Local ISO8601, for example 2025-04-22T17:00:00.")}
      ${field("ticket.comments", "Invoice Comments", state.ticket.comments)}
      ${field("ticket.route", "Route Name", state.ticket.route)}
      ${field("ticket.routeStop", "Route Stop", state.ticket.routeStop)}
    </div>
  `;
}

function renderTicketSections(state: AppState, isWhiteConveyors: boolean) {
  return `
    <div class="section-title">
      <h2>Ticket</h2>
      <span>${isWhiteConveyors ? "Comp-U-Sort TICKET_CREATE identifiers" : "WinCleaners TICKET_CREATE identifiers"}</span>
    </div>
    <div class="form-grid aligned-grid">
      ${field("ticket.ticketNumber", "Ticket Number", state.ticket.ticketNumber)}
      ${field("ticket.readyDate", "Ready Date", state.ticket.readyDate, "MM/DD/YYYY")}
      ${field("ticket.readyTime", "Ready Time", state.ticket.readyTime, "HH:MM:SS AM/PM")}
    </div>

    <div class="section-title">
      <h2>Location Routing</h2>
      <span>Plant, route, and store values written with the ticket</span>
    </div>
    <div class="form-grid aligned-grid">
      ${field("ticket.plant", "Plant", state.ticket.plant)}
      ${field("ticket.route", "Route", state.ticket.route)}
      ${field("ticket.store", "Store", state.ticket.store)}
    </div>
  `;
}

function emptyTicket(isSpot: boolean) {
  const label = isSpot ? "Invoice" : "Ticket";

  return `
    <div class="empty-state">
      <h2>No ${label.toLowerCase()} selected</h2>
      <p>Use the button below to start a blank ${label.toLowerCase()} record.</p>
      <button class="primary-button" data-action="add-ticket">Create New ${label}</button>
    </div>
  `;
}
