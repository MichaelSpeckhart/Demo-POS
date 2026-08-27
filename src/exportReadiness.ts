import type { Customer, Garment, PosSystemId, Ticket } from "./types";

export function customerReadyForExport(customer: Customer, posSystem: PosSystemId) {
  return Boolean(
    customer.accountNumber.trim() &&
      customer.phoneNumber.trim() &&
      customer.firstName.trim() &&
      customer.lastName.trim() &&
      (posSystem !== "spot" || customer.pin.trim())
  );
}

export function ticketReadyForExport(ticket: Ticket, posSystem: PosSystemId) {
  if (!ticket.ticketNumber.trim()) {
    return false;
  }

  if (posSystem === "spot") {
    return Boolean(
      ticket.fullInvoiceNumber.trim() &&
        ticket.displayInvoiceNumber.trim() &&
        ticket.dropoffDateTime.trim() &&
        ticket.promisedDateTime.trim()
    );
  }

  return Boolean(ticket.readyDate.trim() && ticket.readyTime.trim());
}

export function garmentReadyForExport(garment: Garment, posSystem: PosSystemId) {
  if (!garment.id.trim() || !garment.description.trim()) {
    return false;
  }

  if (posSystem === "spot") {
    return Boolean(garment.slotOccupancy.trim());
  }

  if (posSystem === "whiteconveyors") {
    return Boolean(garment.serviceType.trim());
  }

  return true;
}

export function employeeReadyForExport(employee: { employeeNumber: string; employeeName: string }) {
  return Boolean(employee.employeeNumber.trim() && employee.employeeName.trim());
}
