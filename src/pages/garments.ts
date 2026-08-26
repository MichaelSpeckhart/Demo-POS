import type { AppState, Garment } from "../types";
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
        <button class="secondary-button" data-action="add-garment">+ Add Garment</button>
      </div>

      <div class="record-editor">
        <div class="list-panel record-list">
          <div class="panel-title">
            <h2>Garments</h2>
            <span>${state.garments.length} records</span>
          </div>
          <div class="record-list-items">
            ${state.garments.map((garment, index) => garmentButton(garment, index, index === state.selectedGarmentIndex)).join("")}
          </div>
        </div>
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
            ${field("garment.id", "Garment ID", selectedGarment?.id ?? "")}
            ${field("garment.description", "Description", selectedGarment?.description ?? "")}
            ${isSpot
              ? field("garment.slotOccupancy", "Slot Occupancy", selectedGarment?.slotOccupancy ?? "")
              : isWhiteConveyors
                ? renderWhiteConveyorsGarmentFields(selectedGarment)
                : renderWinCleanersGarmentFields(selectedGarment)}
          </div>
        </div>
      </div>
    </section>
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
