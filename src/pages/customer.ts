import type { AppState } from "../types";
import { field, pageHeader } from "../ui/html";

export function renderCustomerPage(state: AppState) {
  const isSpot = state.settings.posSystem === "spot";
  const isWhiteConveyors = state.settings.posSystem === "whiteconveyors";

  return `
    <section class="page customer-page">
      ${pageHeader(
        "Step 3",
        "Create Customer",
        isSpot
          ? "SPOT repeats customer details on every ADDITEM row."
          : isWhiteConveyors
            ? "These fields map directly to the Comp-U-Sort CUSTOMER_CREATE row."
            : "These fields map directly to the WinCleaners CUSTOMER_CREATE row."
      )}

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
      </div>
    </section>
  `;
}
