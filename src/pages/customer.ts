import type { AppState } from "../types";
import { customerReadyForExport } from "../exportReadiness";
import { field } from "../ui/html";

export function renderCustomerPage(state: AppState) {
  const isSpot = state.settings.posSystem === "spot";
  const isWhiteConveyors = state.settings.posSystem === "whiteconveyors";

  return `
    <section class="page customer-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">Step 3</p>
          <h1>Create Customer</h1>
          <p>${isSpot
            ? "SPOT repeats customer details on every ADDITEM row."
            : isWhiteConveyors
              ? "These fields map directly to the Comp-U-Sort CUSTOMER_CREATE row."
              : "These fields map directly to the WinCleaners CUSTOMER_CREATE row."}</p>
        </div>
        <button class="secondary-button" data-action="add-customer">Create New Customer</button>
      </div>

      ${state.customerDraftActive ? customerForm(state, isSpot) : emptyCustomer()}
    </section>
  `;
}

function customerForm(state: AppState, isSpot: boolean) {
  const ready = customerReadyForExport(state.customer, state.settings.posSystem);
  const added = state.customerAddedToExport && ready;

  return `
    <div class="form-surface">
        <div class="section-title">
          <h2>Customer Profile</h2>
          <span>Identity and contact fields</span>
        </div>
        <div class="form-grid aligned-grid">
          ${field(
            "customer.accountNumber",
            "Account Number",
            state.customer.accountNumber,
            state.settings.posSystem === "wincleaners" ? "Padded to at least 8 digits for CustomerID." : ""
          )}
          ${field("customer.phoneNumber", "Phone Number", state.customer.phoneNumber)}
          ${field("customer.firstName", "First Name", state.customer.firstName)}
          ${field("customer.lastName", "Last Name", state.customer.lastName)}
          ${isSpot ? field("customer.pin", "PIN Number", state.customer.pin) : ""}
        </div>

        <div class="section-title">
          <h2>Address</h2>
          <span>Optional customer location details</span>
        </div>
        <div class="form-grid aligned-grid">
          ${field("customer.address1", "Address 1", state.customer.address1)}
          ${field("customer.address2", "Address 2", state.customer.address2)}
          ${field("customer.city", "City", state.customer.city)}
          ${field("customer.state", "State", state.customer.state)}
          ${field("customer.zipCode", "Zip Code", state.customer.zipCode)}
        </div>
        <div class="add-to-export-row">
          <button class="${ready && !added ? "primary-button" : "secondary-button"} add-to-export-button" data-action="add-customer-to-export" ${ready && !added ? "" : "disabled"}>
            ${added ? "Customer Added to Export" : ready ? "Add Customer to Export" : "Complete Required Fields"}
          </button>
        </div>
      </div>
  `;
}

function emptyCustomer() {
  return `
    <div class="empty-state">
      <h2>No customer selected</h2>
      <p>Use the button below to start a blank customer record.</p>
      <button class="primary-button" data-action="add-customer">Create New Customer</button>
    </div>
  `;
}
