import type { Garment, PosAdapter, Ticket } from "../types";
import { hasCustomerData, hasGarmentData, hasTicketData } from "./recordPresence";

export const winCleanersAdapter: PosAdapter = {
  id: "wincleaners",
  name: "WinCleaners",
  summary: "CUSTOMER_CREATE, TICKET_CREATE, and GARMENT_CREATE CSV rows",
  formatExport(customer, ticket, garments, _employees, _operation) {
    const now = new Date();
    const date = formatDate(now);
    const time = formatTime(now);
    const customerId = formatCustomerId(customer.accountNumber);
    const rows: string[][] = [];

    if (hasCustomerData(customer)) {
      rows.push([
        "CUSTOMER_CREATE",
        customerId,
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
      ]);
    }

    if (hasTicketData(ticket)) {
      rows.push(ticketCreateRow(customerId, ticket, date, time));
    }

    rows.push(
      ...garments
        .filter(hasGarmentData)
        .map((garment) => garmentCreateRow(customerId, ticket, garment, date, time))
    );

    return rows.length === 0 ? "" : rows.map(toCsvRow).join("\n") + "\n";
  },
  fileName(ticket, timestamp) {
    return `wincleaners_${safeName(ticket.ticketNumber)}_${fileTimestamp(timestamp)}.csv`;
  },
};

function ticketCreateRow(customerId: string, ticket: Ticket, date: string, time: string) {
  return [
    "TICKET_CREATE",
    customerId,
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
  customerId: string,
  ticket: Ticket,
  garment: Garment,
  date: string,
  time: string
) {
  return [
    "GARMENT_CREATE",
    customerId,
    ticket.ticketNumber,
    garment.id,
    garment.description,
    garment.servicePrice,
    garment.serviceType,
    garment.color,
    date,
    time,
  ];
}

function formatCustomerId(accountNumber: string) {
  const trimmed = accountNumber.trim();
  if (/^\d+$/.test(trimmed)) {
    return trimmed.padStart(8, "0");
  }
  return trimmed;
}

function toCsvRow(values: string[]) {
  return values.map(csvEscape).join(",");
}

function csvEscape(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
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

function fileTimestamp(date: Date) {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const min = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");
  return `${yyyy}${mm}${dd}_${hh}${min}${ss}`;
}

function safeName(value: string) {
  return value.trim().replace(/[^a-z0-9_-]/gi, "_") || "ticket";
}
