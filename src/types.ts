export type PosSystemId = "wincleaners" | "spot" | "whiteconveyors";
export type ExportOperation =
  | "create"
  | "customerDelete"
  | "ticketDelete"
  | "garmentDelete"
  | "employeeDelete";
export type PageId =
  | "pos"
  | "folders"
  | "printer"
  | "employees"
  | "customer"
  | "ticket"
  | "garments"
  | "export"
  | "database";

export interface AppSettings {
  posSystem: PosSystemId;
  inputDirectory: string;
  inputFileName: string;
  outputDirectory: string;
  outputFileName: string;
  exportOperation: ExportOperation;
  receiptPrinterPath: string;
}

export interface Customer {
  accountNumber: string;
  phoneNumber: string;
  firstName: string;
  lastName: string;
  pin: string;
  address1: string;
  address2: string;
  city: string;
  state: string;
  zipCode: string;
}

export interface Ticket {
  customerAccountNumber: string;
  ticketNumber: string;
  fullInvoiceNumber: string;
  displayInvoiceNumber: string;
  balanceDue: string;
  dropoffDateTime: string;
  promisedDateTime: string;
  comments: string;
  readyDate: string;
  readyTime: string;
  plant: string;
  route: string;
  routeStop: string;
  store: string;
}

export interface Garment {
  id: string;
  description: string;
  slotOccupancy: string;
  servicePrice: string;
  serviceType: string;
  garmentType: string;
  color: string;
  fabric: string;
}

export interface Employee {
  employeeNumber: string;
  employeeName: string;
}

export interface WriteExportRequest {
  posSystem: PosSystemId;
  ticketNumber: string;
  outputDirectory: string;
  fileName: string;
  contents: string;
}

export interface ReceiptPrinterInfo {
  path: string;
  description: string;
}

export interface PosAdapter {
  id: PosSystemId;
  name: string;
  summary: string;
  formatExport(
    customer: Customer,
    ticket: Ticket,
    garments: Garment[],
    employees: Employee[],
    operation: ExportOperation
  ): string;
  fileName(ticket: Ticket, timestamp: Date): string;
}

export interface AppState {
  activePage: PageId;
  settings: AppSettings;
  customer: Customer;
  ticket: Ticket;
  garments: Garment[];
  employees: Employee[];
  customerDraftActive: boolean;
  ticketDraftActive: boolean;
  selectedGarmentIndex: number;
  selectedEmployeeIndex: number;
  preview: string;
  status: string;
  statusKind: "neutral" | "success" | "error";
  databaseSummary: DatabaseSummary | null;
  receiptPrinters: ReceiptPrinterInfo[];
}

export interface DatabaseSummary {
  path: string;
  customerCount: number;
  employeeCount: number;
  ticketCount: number;
  garmentCount: number;
  exportCount: number;
  lastExportPath: string | null;
  lastExportAt: string | null;
  customers: CustomerRecord[];
  employees: EmployeeRecord[];
  tickets: TicketRecord[];
  garments: GarmentRecord[];
  exports: ExportRecord[];
}

export interface CustomerRecord {
  accountNumber: string;
  phoneNumber: string;
  firstName: string;
  lastName: string;
  pin: string;
  address1: string;
  address2: string;
  name: string;
  city: string;
  state: string;
  zipCode: string;
  updatedAt: string;
}

export interface TicketRecord {
  ticketNumber: string;
  customerAccountNumber: string;
  fullInvoiceNumber: string;
  displayInvoiceNumber: string;
  promisedDateTime: string;
  readyDate: string;
  readyTime: string;
  updatedAt: string;
}

export interface GarmentRecord {
  ticketNumber: string;
  garmentId: string;
  description: string;
  slotOccupancy: string;
  serviceType: string;
  garmentType: string;
  color: string;
  fabric: string;
  position: number;
}

export interface EmployeeRecord {
  employeeNumber: string;
  employeeName: string;
  updatedAt: string;
}

export interface ExportRecord {
  id: number;
  posSystem: string;
  ticketNumber: string;
  filePath: string;
  payload: string;
  createdAt: string;
}
