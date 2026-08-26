import type { Customer, Employee, Garment, PosAdapter, Ticket } from "../types";
import {
  hasCustomerData,
  hasEmployeeData,
  hasGarmentData,
  hasTicketData,
} from "./recordPresence";

export const whiteConveyorsAdapter: PosAdapter = {
  id: "whiteconveyors",
  name: "White Conveyors",
  summary: "Comp-U-Sort POS.txt customer, ticket, and garment transactions",
  formatExport(customer, ticket, garments, employees) {
    const now = new Date();
    const date = formatDate(now);
    const time = formatTime(now);
    const accountNumber = customer.accountNumber.trim();
    const rows: string[][] = [
      ...employees
        .filter(hasEmployeeData)
        .map((employee) => employeeCreateRow(employee, date, time)),
    ];

    if (hasCustomerData(customer)) {
      rows.push(customerCreateRow(customer, accountNumber, date, time));
    }

    if (hasTicketData(ticket)) {
      rows.push(ticketCreateRow(accountNumber, ticket, date, time));
    }

    rows.push(
      ...garments
        .filter(hasGarmentData)
        .map((garment) => garmentCreateRow(accountNumber, ticket, garment, date, time))
    );

    return rows.length === 0 ? "" : rows.map(toQuotedCsvRow).join("\r\n") + "\r\n";
  },
  fileName() {
    return "POS.txt";
  },
};

function employeeCreateRow(employee: Employee, date: string, time: string) {
  return [
    "EMPLOYEE_CREATE",
    employee.employeeNumber,
    employee.employeeName,
    date,
    time,
  ];
}

function customerCreateRow(customer: Customer, accountNumber: string, date: string, time: string) {
  return [
    "CUSTOMER_CREATE",
    accountNumber,
    customer.phoneNumber,
    customer.lastName,
    customer.firstName,
    customer.address1,
    customer.address2,
    customer.city,
    customer.state,
    customer.zipCode,
    date,
    time,
  ];
}

function ticketCreateRow(accountNumber: string, ticket: Ticket, date: string, time: string) {
  return [
    "TICKET_CREATE",
    accountNumber,
    ticket.ticketNumber,
    ticket.readyDate,
    ticket.readyTime,
    ticket.plant,
    ticket.route,
    ticket.store,
    date,
    time,
  ];
}

function garmentCreateRow(
  accountNumber: string,
  ticket: Ticket,
  garment: Garment,
  date: string,
  time: string
) {
  return [
    "GARMENT_CREATE",
    accountNumber,
    ticket.ticketNumber,
    garment.id,
    garment.description,
    garment.servicePrice,
    garment.serviceType,
    garment.garmentType,
    garment.color,
    garment.fabric,
    date,
    time,
  ];
}

function toQuotedCsvRow(values: string[]) {
  return values.map((value) => `"${value.replace(/"/g, '""')}"`).join(",");
}

function formatDate(date: Date) {
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${month}/${day}/${date.getFullYear()}`;
}

function formatTime(date: Date) {
  let hours = date.getHours();
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");
  const suffix = hours >= 12 ? "PM" : "AM";
  hours %= 12;
  if (hours === 0) hours = 12;
  return `${String(hours).padStart(2, "0")}:${minutes}:${seconds} ${suffix}`;
}
