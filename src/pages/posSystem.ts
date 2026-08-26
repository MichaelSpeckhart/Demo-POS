import type { AppState } from "../types";
import { pageHeader } from "../ui/html";
import CleanCloudLogo from "../assets/CleanCloudLogo.svg";
import AppLogo from "../assets/Logo1.png";
import SMRTSystemsLogo from "../assets/SMRTSystemsLogo.svg";
import XplorSpotLogo from "../assets/XplorSpotLogo.svg";

const posLogos: Record<string, string> = {
  cleancloud: CleanCloudLogo,
  smrt: SMRTSystemsLogo,
  spot: XplorSpotLogo,
  whiteconveyors: AppLogo,
};

const outputDetails: Record<string, { title: string; description: string }> = {
  spot: {
    title: "SPOT output",
    description: "One ADDITEM row per garment. Customer and invoice values repeat on each row.",
  },
  wincleaners: {
    title: "WinCleaners output",
    description: "One CUSTOMER_CREATE row, one TICKET_CREATE row, then one GARMENT_CREATE row per item.",
  },
  whiteconveyors: {
    title: "White Conveyors output",
    description: "Writes Comp-U-Sort POS.txt transactions for customer, ticket, and garment records.",
  },
};

export function renderPosSystemPage(state: AppState) {
  return `
    <section class="page pos-page">
      ${pageHeader(
        "Step 1",
        "Choose POS System",
        "The selected POS controls how customers, tickets, and garments are written to disk."
      )}

      <div class="pos-grid">
        ${posCard("spot", "SPOT", "One ADDITEM row per garment", true, state.settings.posSystem === "spot")}
        ${posCard("wincleaners", "WinCleaners", "Customer, ticket, and garment CSV rows", true, state.settings.posSystem === "wincleaners")}
        ${posCard("whiteconveyors", "White Conveyors", "Comp-U-Sort POS.txt transactions", true, state.settings.posSystem === "whiteconveyors")}
        ${posCard("cleancloud", "CleanCloud", "Coming later", false, false)}
        ${posCard("smrt", "SMRT", "Coming later", false, false)}
      </div>

      <div class="info-band">
        <strong>${outputDetails[state.settings.posSystem].title}</strong>
        <span>${outputDetails[state.settings.posSystem].description}</span>
      </div>
    </section>
  `;
}

function posCard(id: string, name: string, description: string, available: boolean, active: boolean) {
  const logo = posLogos[id];

  return `
    <button
      class="pos-card ${active ? "active" : ""}"
      data-action="select-pos"
      data-pos="${id}"
      ${available ? "" : "disabled"}
    >
      ${logo
        ? `<img class="pos-logo" src="${logo}" alt="${name} logo" />`
        : `<span class="pos-name">${name}</span>`}
      <small>${description}</small>
      ${available ? "" : "<em>Coming soon</em>"}
    </button>
  `;
}
