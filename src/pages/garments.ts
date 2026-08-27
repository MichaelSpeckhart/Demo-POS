import type { AppState, Garment } from "../types";
import { garmentReadyForExport, ticketReadyForExport } from "../exportReadiness";
import { escapeAttribute, escapeHtml, field } from "../ui/html";

export function renderGarmentsPage(state: AppState) {
  const selectedGarment = state.garments[state.selectedGarmentIndex] ?? state.garments[0];
  const isSpot = state.settings.posSystem === "spot";
  const isWhiteConveyors = state.settings.posSystem === "whiteconveyors";

  return `
    <section class="page garments-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">Step 5</p>
          <h1>Create Garments</h1>
          <p>${garmentDescription(state.settings.posSystem)}</p>
        </div>
        <button class="secondary-button" data-action="add-garment">Create New Garment</button>
      </div>

      <div class="record-editor">
        <div class="list-panel record-list">
          <div class="panel-title">
            <h2>Garments</h2>
            <span>${state.garments.length} records</span>
          </div>
          <div class="record-list-items">
            ${state.garments.length === 0 ? `<div class="empty-list">No garments added</div>` : state.garments.map((garment, index) => garmentButton(garment, index, index === state.selectedGarmentIndex)).join("")}
          </div>
        </div>
        ${selectedGarment ? garmentForm(state, selectedGarment, isSpot, isWhiteConveyors) : emptyGarment()}
      </div>
    </section>
  `;
}

function garmentForm(
  state: AppState,
  selectedGarment: Garment,
  isSpot: boolean,
  isWhiteConveyors: boolean
) {
  const ticketReady = state.ticketAddedToExport && ticketReadyForExport(state.ticket, state.settings.posSystem);
  const ready = ticketReady && garmentReadyForExport(selectedGarment, state.settings.posSystem);
  const added = Boolean(state.garmentAddedToExport[state.selectedGarmentIndex] && ready);

  return `
    <div class="form-surface record-form">
          <div class="section-title">
            <h2>Ticket Assignment</h2>
            <span>Choose which ticket owns these garment rows</span>
          </div>
          <label class="field selector-field">
            <span>Ticket for these garments</span>
            <select data-action="select-ticket">
              ${ticketOptions(state)}
            </select>
          </label>

          <div class="section-title">
            <h2>Garment Details</h2>
            <span>${isSpot ? "SPOT item fields" : isWhiteConveyors ? "Comp-U-Sort garment fields" : "WinCleaners garment fields"}</span>
          </div>
          <div class="form-grid aligned-grid two-column-grid">
            ${field("garment.id", "Garment ID", selectedGarment.id)}
            ${field("garment.description", "Description", selectedGarment.description)}
            ${isSpot
              ? field("garment.slotOccupancy", "Slot Occupancy", selectedGarment.slotOccupancy)
              : isWhiteConveyors
                ? renderWhiteConveyorsGarmentFields(selectedGarment)
                : renderWinCleanersGarmentFields(selectedGarment)}
          </div>
          <div class="add-to-export-row">
            <button class="${ready && !added ? "primary-button" : "secondary-button"} add-to-export-button" data-action="add-garment-to-export" ${ready && !added ? "" : "disabled"}>
              ${added ? "Garment Added to Export" : !ticketReady ? "Add Ticket First" : ready ? "Add Garment to Export" : "Complete Required Fields"}
            </button>
          </div>
        </div>
  `;
}

function emptyGarment() {
  return `
    <div class="empty-state record-form">
      <h2>No garment selected</h2>
      <p>Use the button below to start a blank garment record.</p>
      <button class="primary-button" data-action="add-garment">Create New Garment</button>
    </div>
  `;
}

function garmentDescription(posSystem: AppState["settings"]["posSystem"]) {
  if (posSystem === "spot") return "Each garment becomes one SPOT ADDITEM row.";
  if (posSystem === "whiteconveyors") return "Each garment becomes one Comp-U-Sort GARMENT_CREATE row.";
  return "Each garment becomes one WinCleaners GARMENT_CREATE row.";
}

function ticketOptions(state: AppState) {
  const records = state.databaseSummary?.tickets ?? [];
  const seen = new Set<string>();
  const options = records.map((ticket) => {
    seen.add(ticket.ticketNumber);
    const display = ticket.displayInvoiceNumber || ticket.fullInvoiceNumber || ticket.ticketNumber;
    return option(ticket.ticketNumber, `${ticket.ticketNumber} - ${display}`, ticket.ticketNumber === state.ticket.ticketNumber);
  });

  if (state.ticket.ticketNumber && !seen.has(state.ticket.ticketNumber)) {
    const display = state.ticket.displayInvoiceNumber || state.ticket.fullInvoiceNumber || state.ticket.ticketNumber;
    options.unshift(option(state.ticket.ticketNumber, `${state.ticket.ticketNumber} - ${display}`, true));
  }

  options.unshift(option("", "No ticket selected", !state.ticket.ticketNumber));
  return options.join("");
}

function option(value: string, label: string, selected: boolean) {
  return `
    <option value="${escapeAttribute(value)}" ${selected ? "selected" : ""}>
      ${escapeHtml(label)}
    </option>
  `;
}

function renderWinCleanersGarmentFields(garment: Garment | undefined) {
  return `
    ${field("garment.servicePrice", "Service Price", garment?.servicePrice ?? "", "OAS ignores this column, but WinCleaners includes it.")}
    ${field("garment.serviceType", "Service Type", garment?.serviceType ?? "")}
    ${field("garment.color", "Color", garment?.color ?? "")}
  `;
}

function renderWhiteConveyorsGarmentFields(garment: Garment | undefined) {
  return `
    ${field("garment.servicePrice", "Service Price", garment?.servicePrice ?? "")}
    ${field("garment.serviceType", "Service Type", garment?.serviceType ?? "", "Required by Comp-U-Sort. Codes must exist in White Conveyors service type setup.")}
    ${field("garment.garmentType", "Garment Type", garment?.garmentType ?? "")}
    ${field("garment.color", "Color", garment?.color ?? "")}
    ${field("garment.fabric", "Fabric", garment?.fabric ?? "")}
  `;
}

function garmentButton(garment: Garment, index: number, selected: boolean) {
  return `
    <button class="garment-button ${selected ? "active" : ""}" data-action="select-garment" data-index="${index}">
      <strong>${escapeHtml(garment.id || `Garment ${index + 1}`)}</strong>
      <span>${escapeHtml(garment.description || "No description")}</span>
    </button>
  `;
}
