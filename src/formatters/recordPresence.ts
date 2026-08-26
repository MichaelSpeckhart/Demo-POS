import type { Customer, Employee, Garment, Ticket } from "../types";

function hasAnyValue(values: string[]) {
  return values.some((value) => value.trim());
}

export function hasCustomerData(customer: Customer) {
  return hasAnyValue([
    customer.accountNumber,
    customer.phoneNumber,
    customer.firstName,
    customer.lastName,
    customer.pin,
    customer.address1,
    customer.address2,
    customer.city,
    customer.state,
    customer.zipCode,
  ]);
}

export function hasTicketData(ticket: Ticket) {
  return hasAnyValue([
    ticket.ticketNumber,
    ticket.fullInvoiceNumber,
    ticket.displayInvoiceNumber,
    ticket.balanceDue,
    ticket.dropoffDateTime,
    ticket.promisedDateTime,
    ticket.comments,
    ticket.readyDate,
    ticket.readyTime,
    ticket.plant,
    ticket.route,
    ticket.routeStop,
    ticket.store,
  ]);
}

export function hasGarmentData(garment: Garment) {
  return hasAnyValue([
    garment.id,
    garment.description,
    garment.slotOccupancy,
    garment.servicePrice,
    garment.serviceType,
    garment.garmentType,
    garment.color,
    garment.fabric,
  ]);
}

export function hasEmployeeData(employee: Employee) {
  return hasAnyValue([
    employee.employeeNumber,
    employee.employeeName,
  ]);
}

export function hasExportData(
  customer: Customer,
  ticket: Ticket,
  garments: Garment[],
  employees: Employee[]
) {
  return (
    hasCustomerData(customer) ||
    hasTicketData(ticket) ||
    garments.some(hasGarmentData) ||
    employees.some(hasEmployeeData)
  );
}
