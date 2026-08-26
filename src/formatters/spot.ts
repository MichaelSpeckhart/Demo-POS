import type { Customer, Garment, PosAdapter, Ticket } from "../types";
import { hasGarmentData } from "./recordPresence";

export const spotAdapter: PosAdapter = {
  id: "spot",
  name: "SPOT",
  summary: "One ADDITEM CSV row per garment",
  formatExport(customer, ticket, garments) {
    const sentAt = localIsoDateTime(new Date());
    const exportGarments = garments.filter(hasGarmentData);
    if (exportGarments.length === 0) {
      return "";
    }

    const rows = exportGarments.map((garment) =>
      addItemRow(customer, ticket, garment, String(exportGarments.length), sentAt)
    );

    return rows.map(toCsvRow).join("\n") + "\n";
  },
  fileName(ticket, timestamp) {
    return `spot_${safeName(ticket.displayInvoiceNumber || ticket.ticketNumber)}_${fileTimestamp(timestamp)}.csv`;
  },
};

function addItemRow(
  customer: Customer,
  ticket: Ticket,
  garment: Garment,
  itemCount: string,
  sentAt: string
) {
  return [
    "ADDITEM",
    ticket.fullInvoiceNumber,
    ticket.displayInvoiceNumber || ticket.ticketNumber,
    itemCount,
    garment.slotOccupancy,
    ticket.balanceDue,
    customer.accountNumber,
    customer.firstName,
    customer.lastName,
    "", // Column 10 is omitted in the SPOT doc; keep it empty so phone lands in column 11.
    customer.phoneNumber,
    garment.id,
    garment.description,
    ticket.dropoffDateTime,
    ticket.promisedDateTime,
    ticket.comments,
    sentAt,
    customer.pin,
    customer.address1,
    customer.address2,
    customer.city,
    customer.state,
    customer.zipCode,
    ticket.route,
    ticket.routeStop,
  ];
}

function toCsvRow(values: string[]) {
  return values.map(csvEscape).join(",");
}

function csvEscape(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
}

function localIsoDateTime(date: Date) {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const min = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}T${hh}:${min}:${ss}`;
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
  return value.trim().replace(/[^a-z0-9_-]/gi, "_") || "invoice";
}
