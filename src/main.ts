import { invoke } from "@tauri-apps/api/core";
import { adapters } from "./formatters";
import { hasCustomerData, hasExportData, hasGarmentData, hasTicketData } from "./formatters/recordPresence";
import { renderCustomerPage } from "./pages/customer";
import { renderDatabasePage } from "./pages/database";
import { renderEmployeesPage } from "./pages/employees";
import { renderExportPage } from "./pages/export";
import { renderFoldersPage } from "./pages/folders";
import { renderGarmentsPage } from "./pages/garments";
import { renderPosSystemPage } from "./pages/posSystem";
import { renderPrinterPage } from "./pages/printer";
import { renderTicketPage } from "./pages/ticket";
import { resolveOutputFileName } from "./outputFileName";
import AppLogo from "./assets/Logo1.png";
import type {
  AppSettings,
  AppState,
  Customer,
  CustomerRecord,
  Employee,
  EmployeeRecord,
  ExportOperation,
  Garment,
  GarmentRecord,
  PageId,
  ReceiptPrinterInfo,
  Ticket,
  TicketRecord,
  WriteExportRequest,
} from "./types";

const defaultSettings: AppSettings = {
  posSystem: "spot",
  inputDirectory: "",
  inputFileName: "",
  outputDirectory: "",
  outputFileName: "",
  exportOperation: "create",
  receiptPrinterPath: "",
};

function defaultCustomer(): Customer {
  return {
    accountNumber: "",
    phoneNumber: "",
    firstName: "",
    lastName: "",
    pin: "",
    address1: "",
    address2: "",
    city: "",
    state: "",
    zipCode: "",
  };
}

function defaultTicket(customerAccountNumber = ""): Ticket {
  return {
    customerAccountNumber,
    ticketNumber: "",
    fullInvoiceNumber: "",
    displayInvoiceNumber: "",
    balanceDue: "",
    dropoffDateTime: "",
    promisedDateTime: "",
    comments: "",
    readyDate: "",
    readyTime: "",
    plant: "",
    route: "",
    routeStop: "",
    store: "",
  };
}

function defaultGarments(): Garment[] {
  return [];
}

function blankGarment(): Garment {
  return {
    id: "",
    description: "",
    slotOccupancy: "",
    servicePrice: "",
    serviceType: "",
    garmentType: "",
    color: "",
    fabric: "",
  };
}

function defaultEmployees(): Employee[] {
  return [];
}

function blankEmployee(): Employee {
  return {
    employeeNumber: "",
    employeeName: "",
  };
}

const state: AppState = {
  activePage: "pos",
  settings: { ...defaultSettings },
  customer: defaultCustomer(),
  ticket: defaultTicket(),
  garments: defaultGarments(),
  employees: defaultEmployees(),
  customerDraftActive: false,
  ticketDraftActive: false,
  selectedGarmentIndex: 0,
  selectedEmployeeIndex: 0,
  preview: "",
  status: "",
  statusKind: "neutral",
  databaseSummary: null,
  receiptPrinters: [],
};

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("Missing #app root");
}

const appRoot = app;
let workspaceSaveTimer: number | undefined;
let deleteHandlerBound = false;

const pages: { id: PageId; label: string; render: (state: AppState) => string }[] = [
  { id: "pos", label: "POS System", render: renderPosSystemPage },
  { id: "folders", label: "Folders", render: renderFoldersPage },
  { id: "printer", label: "Printer", render: renderPrinterPage },
  { id: "employees", label: "Employees", render: renderEmployeesPage },
  { id: "customer", label: "Customer", render: renderCustomerPage },
  { id: "ticket", label: "Ticket", render: renderTicketPage },
  { id: "garments", label: "Garments", render: renderGarmentsPage },
  { id: "export", label: "Export", render: renderExportPage },
  { id: "database", label: "Database", render: renderDatabasePage },
];

function render() {
  const adapter = adapters[state.settings.posSystem];
  state.preview = adapter.formatExport(
    state.customer,
    state.ticket,
    state.garments,
    state.employees,
    state.settings.exportOperation
  );
  const activePage = pages.find((page) => page.id === state.activePage) ?? pages[0];

  appRoot.innerHTML = `
    <div class="shell">
      <aside class="sidebar">
        <div class="brand">
          <img src="${AppLogo}" alt="White logo" />
          <div>
            <span>Demo POS</span>
            <strong>Export Simulator</strong>
          </div>
        </div>
        <nav class="steps">
          ${pages.map((page) => navButton(page.id, page.label, page.id === state.activePage)).join("")}
        </nav>
        <div class="sidebar-note">
          <strong>${adapter.name}</strong>
          <span>${adapter.summary}</span>
        </div>
      </aside>
      <main class="content">${activePage.render(state)}</main>
    </div>
  `;

  bindEvents();
}

function navButton(id: PageId, label: string, active: boolean) {
  return `
    <button class="${active ? "active" : ""}" data-action="navigate" data-page="${id}">
      ${label}
    </button>
  `;
}

function bindEvents() {
  document.querySelectorAll<HTMLButtonElement>("[data-action='navigate']").forEach((button) => {
    button.addEventListener("click", () => {
      const page = button.dataset.page as PageId | undefined;
      if (page) {
        state.activePage = page;
        state.status = "";
        render();
        if (page === "database") {
          void refreshDatabaseSummary();
        }
      }
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-export-operation']").forEach((select) => {
    select.addEventListener("change", () => {
      const operation = select.value;
      if (isExportOperation(operation)) {
        state.settings.exportOperation = operation;
        state.status = "";
        resetWorkspaceDraft();
        render();
        void saveSettings();
      }
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-delete-customer']").forEach((select) => {
    select.addEventListener("change", () => {
      selectCustomerDeleteTarget(select.value);
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-delete-ticket']").forEach((select) => {
    select.addEventListener("change", () => {
      selectTicketDeleteTarget(select.value);
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-delete-garment']").forEach((select) => {
    select.addEventListener("change", () => {
      selectGarmentDeleteTarget(select.value);
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-delete-employee']").forEach((select) => {
    select.addEventListener("change", () => {
      selectEmployeeDeleteTarget(select.value);
    });
  });

  document.querySelectorAll<HTMLInputElement>("[data-bind]").forEach((input) => {
    input.addEventListener("input", () => {
      updateValue(input.dataset.bind ?? "", input.value);
      updateLivePreview();
      const bindPath = input.dataset.bind ?? "";
      if (bindPath.startsWith("settings.")) {
        void saveSettings();
      } else {
        queueCurrentRecordSave(bindPath);
      }
    });
    input.addEventListener("change", () => {
      const bindPath = input.dataset.bind ?? "";
      updateValue(bindPath, input.value);
      updateLivePreview();
      if (bindPath.startsWith("settings.")) {
        void saveSettings();
      } else {
        void saveCurrentRecord(bindPath).then(refreshDatabaseSummary);
      }
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-receipt-printer']").forEach((select) => {
    select.addEventListener("change", () => {
      state.settings.receiptPrinterPath = select.value;
      state.status = "";
      render();
      void saveSettings();
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-customer']").forEach((select) => {
    select.addEventListener("change", async () => {
      const accountNumber = select.value;
      if (!accountNumber) {
        state.ticket.customerAccountNumber = "";
        state.customer = defaultCustomer();
        state.customerDraftActive = false;
        state.status = "";
        render();
        return;
      }
      state.ticket.customerAccountNumber = accountNumber;
      const customer = await invoke<Customer | null>("load_customer", { accountNumber });
      if (customer) {
        state.customer = customer;
        state.customerDraftActive = true;
      }
      state.status = "";
      render();
      await saveWorkspace();
      await refreshDatabaseSummary();
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-ticket']").forEach((select) => {
    select.addEventListener("change", async () => {
      const ticketNumber = select.value;
      if (!ticketNumber) {
        state.ticket = defaultTicket(state.customer.accountNumber);
        state.ticketDraftActive = false;
        state.garments = [];
        state.selectedGarmentIndex = 0;
        state.status = "";
        render();
        return;
      }
      await loadTicketWorkspace(ticketNumber);
      state.status = "";
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='add-customer']").forEach((button) => {
    button.addEventListener("click", () => {
      state.customer = defaultCustomer();
      state.customerDraftActive = true;
      if (!state.ticketDraftActive) {
        state.ticket = defaultTicket();
      }
      state.status = "";
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='add-ticket']").forEach((button) => {
    button.addEventListener("click", () => {
      state.ticket = defaultTicket(state.customer.accountNumber);
      state.ticketDraftActive = true;
      state.status = "";
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='select-pos']").forEach((button) => {
    button.addEventListener("click", () => {
      const pos = button.dataset.pos;
      if (pos === "wincleaners" || pos === "spot" || pos === "whiteconveyors") {
        state.settings.posSystem = pos;
        if (pos !== "whiteconveyors") {
          state.settings.exportOperation = "create";
        }
        state.status = "";
        render();
        void saveSettings();
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='choose-folder']").forEach((button) => {
    button.addEventListener("click", async () => {
      const key = button.dataset.folderKey as keyof Pick<AppSettings, "inputDirectory" | "outputDirectory"> | undefined;
      if (!key) return;

      try {
        const selected = await invoke<string | null>("choose_folder", {
          title: key === "outputDirectory" ? "Choose POS output folder" : "Choose POS input folder",
        });
        if (selected) {
          state.settings[key] = selected;
          state.status = "";
          render();
          await saveSettings();
        }
      } catch (error) {
        setStatus(`Folder picker failed: ${String(error)}`, "error");
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='select-garment']").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedGarmentIndex = Number(button.dataset.index ?? 0);
      state.status = "";
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='select-employee']").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedEmployeeIndex = Number(button.dataset.index ?? 0);
      state.status = "";
      render();
    });
  });

  document.querySelector<HTMLButtonElement>("[data-action='add-garment']")?.addEventListener("click", () => {
    state.garments.push(blankGarment());
    state.selectedGarmentIndex = state.garments.length - 1;
    state.status = "";
    render();
  });

  document.querySelector<HTMLButtonElement>("[data-action='add-employee']")?.addEventListener("click", () => {
    state.employees.push(blankEmployee());
    state.selectedEmployeeIndex = state.employees.length - 1;
    state.status = "";
    render();
  });

  document.querySelector<HTMLButtonElement>("[data-action='export']")?.addEventListener("click", () => {
    void exportFile();
  });

  document.querySelector<HTMLButtonElement>("[data-action='scan-printers']")?.addEventListener("click", () => {
    void scanReceiptPrinters();
  });

  document.querySelector<HTMLButtonElement>("[data-action='test-receipt-printer']")?.addEventListener("click", () => {
    void testReceiptPrinter();
  });

  document.querySelector<HTMLButtonElement>("[data-action='print-receipt']")?.addEventListener("click", () => {
    void printReceipt();
  });

}

function bindGlobalDeleteHandler() {
  if (deleteHandlerBound) return;
  deleteHandlerBound = true;

  document.addEventListener(
    "click",
    (event) => {
      const target = event.target;
      if (!(target instanceof Element)) return;

      const button = target.closest<HTMLButtonElement>(".danger-button");
      if (!button) return;

      event.preventDefault();
      event.stopPropagation();

      const action = button.dataset.action;
      if (action === "delete-customer") {
        const accountNumber = button.dataset.accountNumber;
        if (accountNumber) void deleteCustomer(accountNumber);
        return;
      }
      if (action === "delete-ticket") {
        const ticketNumber = button.dataset.ticketNumber;
        if (ticketNumber) void deleteTicket(ticketNumber);
        return;
      }
      if (action === "delete-garment") {
        const ticketNumber = button.dataset.ticketNumber;
        const garmentId = button.dataset.garmentId;
        if (ticketNumber && garmentId) void deleteGarment(ticketNumber, garmentId);
        return;
      }
      if (action === "delete-employee") {
        const employeeNumber = button.dataset.employeeNumber;
        if (employeeNumber) void deleteEmployee(employeeNumber);
      }
    },
    true
  );
}

function updateValue(path: string, value: string) {
  if (path.startsWith("settings.")) {
    const key = path.replace("settings.", "") as keyof Pick<
      AppSettings,
      "inputDirectory" | "inputFileName" | "outputDirectory" | "outputFileName" | "receiptPrinterPath"
    >;
    state.settings[key] = value;
    return;
  }

  if (path.startsWith("customer.")) {
    const key = path.replace("customer.", "") as keyof Customer;
    state.customerDraftActive = true;
    state.customer[key] = value;
    if (key === "accountNumber") {
      state.ticket.customerAccountNumber = value;
    }
    return;
  }

  if (path.startsWith("ticket.")) {
    const key = path.replace("ticket.", "") as keyof Ticket;
    state.ticketDraftActive = true;
    state.ticket[key] = value;
    return;
  }

  if (path.startsWith("garment.")) {
    const key = path.replace("garment.", "") as keyof Garment;
    const garment = state.garments[state.selectedGarmentIndex];
    if (garment) {
      garment[key] = value;
    }
  }

  if (path.startsWith("employee.")) {
    const key = path.replace("employee.", "") as keyof Employee;
    const employee = state.employees[state.selectedEmployeeIndex];
    if (employee) {
      employee[key] = value;
    }
  }
}

function selectCustomerDeleteTarget(accountNumber: string) {
  const row = state.databaseSummary?.customers.find((customer) => customer.accountNumber === accountNumber);
  if (!row) {
    resetWorkspaceDraft();
    render();
    return;
  }

  state.customer = customerFromRecord(row);
  state.ticket = defaultTicket(row.accountNumber);
  state.garments = [];
  state.employees = [];
  state.customerDraftActive = true;
  state.ticketDraftActive = false;
  state.selectedGarmentIndex = 0;
  state.selectedEmployeeIndex = 0;
  state.status = "";
  render();
}

function selectTicketDeleteTarget(ticketNumber: string) {
  const row = state.databaseSummary?.tickets.find((ticket) => ticket.ticketNumber === ticketNumber);
  if (!row) {
    resetWorkspaceDraft();
    render();
    return;
  }

  setTicketDeleteTarget(row);
  state.garments = [];
  state.employees = [];
  state.status = "";
  render();
}

function selectGarmentDeleteTarget(value: string) {
  const [ticketNumber, garmentId] = value.split("::");
  const row = state.databaseSummary?.garments.find(
    (garment) => garment.ticketNumber === ticketNumber && garment.garmentId === garmentId
  );
  if (!row) {
    resetWorkspaceDraft();
    render();
    return;
  }

  const ticket = state.databaseSummary?.tickets.find((record) => record.ticketNumber === row.ticketNumber);
  if (ticket) {
    setTicketDeleteTarget(ticket);
  } else {
    state.customer = defaultCustomer();
    state.ticket = defaultTicket();
    state.ticket.ticketNumber = row.ticketNumber;
    state.ticketDraftActive = true;
  }

  state.garments = [garmentFromRecord(row)];
  state.employees = [];
  state.selectedGarmentIndex = 0;
  state.selectedEmployeeIndex = 0;
  state.status = "";
  render();
}

function selectEmployeeDeleteTarget(employeeNumber: string) {
  const row = state.databaseSummary?.employees.find((employee) => employee.employeeNumber === employeeNumber);
  if (!row) {
    resetWorkspaceDraft();
    render();
    return;
  }

  state.customer = defaultCustomer();
  state.ticket = defaultTicket();
  state.garments = [];
  state.employees = [employeeFromRecord(row)];
  state.customerDraftActive = false;
  state.ticketDraftActive = false;
  state.selectedGarmentIndex = 0;
  state.selectedEmployeeIndex = 0;
  state.status = "";
  render();
}

function setTicketDeleteTarget(row: TicketRecord) {
  const customer = state.databaseSummary?.customers.find(
    (record) => record.accountNumber === row.customerAccountNumber
  );
  state.customer = customer ? customerFromRecord(customer) : defaultCustomer();
  state.customer.accountNumber = row.customerAccountNumber;
  state.ticket = ticketFromRecord(row);
  state.customerDraftActive = true;
  state.ticketDraftActive = true;
  state.selectedGarmentIndex = 0;
  state.selectedEmployeeIndex = 0;
}

function customerFromRecord(row: CustomerRecord): Customer {
  return {
    accountNumber: row.accountNumber,
    phoneNumber: row.phoneNumber,
    firstName: row.firstName,
    lastName: row.lastName,
    pin: row.pin,
    address1: row.address1,
    address2: row.address2,
    city: row.city,
    state: row.state,
    zipCode: row.zipCode,
  };
}

function ticketFromRecord(row: TicketRecord): Ticket {
  return {
    customerAccountNumber: row.customerAccountNumber,
    ticketNumber: row.ticketNumber,
    fullInvoiceNumber: row.fullInvoiceNumber,
    displayInvoiceNumber: row.displayInvoiceNumber,
    balanceDue: "",
    dropoffDateTime: "",
    promisedDateTime: row.promisedDateTime,
    comments: "",
    readyDate: row.readyDate,
    readyTime: row.readyTime,
    plant: "",
    route: "",
    routeStop: "",
    store: "",
  };
}

function garmentFromRecord(row: GarmentRecord): Garment {
  return {
    id: row.garmentId,
    description: row.description,
    slotOccupancy: row.slotOccupancy,
    servicePrice: "",
    serviceType: row.serviceType,
    garmentType: row.garmentType,
    color: row.color,
    fabric: row.fabric,
  };
}

function employeeFromRecord(row: EmployeeRecord): Employee {
  return {
    employeeNumber: row.employeeNumber,
    employeeName: row.employeeName,
  };
}

async function exportFile() {
  const validationError = validateExport();
  if (validationError) {
    setStatus(validationError, "error");
    state.activePage = "export";
    return;
  }

  const adapter = adapters[state.settings.posSystem];
  const timestamp = new Date();
  const fileName = resolveOutputFileName(
    state.settings.outputFileName,
    adapter.fileName(state.ticket, timestamp)
  );
  if (!fileName) {
    setStatus("Choose a valid output file name.", "error");
    state.activePage = "export";
    return;
  }

  const request: WriteExportRequest = {
    posSystem: state.settings.posSystem,
    ticketNumber: state.ticket.ticketNumber,
    outputDirectory: state.settings.outputDirectory,
    fileName,
    contents: adapter.formatExport(
      state.customer,
      state.ticket,
      state.garments,
      state.employees,
      state.settings.exportOperation
    ),
  };

  try {
    if (isEmployeeOnlyCreateExport()) {
      await saveWhiteConveyorsEmployees();
    } else if (!isDeleteExportMode()) {
      await saveWorkspace();
    }
    const path = await invoke<string>("write_export_file", { request });
    await refreshDatabaseSummary();
    resetWorkspaceDraft();
    setStatus(`Wrote ${path}`, "success");
  } catch (error) {
    setStatus(`Export failed: ${String(error)}`, "error");
  }
}

async function scanReceiptPrinters() {
  try {
    state.status = "Scanning for receipt printers...";
    state.statusKind = "neutral";
    render();
    state.receiptPrinters = await invoke<ReceiptPrinterInfo[]>("discover_receipt_printers");
    const selectedExists = state.receiptPrinters.some(
      (printer) => printer.path === state.settings.receiptPrinterPath
    );
    if (!state.settings.receiptPrinterPath || !selectedExists) {
      state.settings.receiptPrinterPath = state.receiptPrinters[0]?.path ?? state.settings.receiptPrinterPath;
      await saveSettings();
    }
    state.status = state.receiptPrinters.length === 0
      ? "No receipt printers found. Enter a printer path manually if needed."
      : `Found ${state.receiptPrinters.length} receipt printer${state.receiptPrinters.length === 1 ? "" : "s"}.`;
    state.statusKind = state.receiptPrinters.length === 0 ? "neutral" : "success";
    render();
  } catch (error) {
    setStatus(`Printer scan failed: ${String(error)}`, "error");
  }
}

async function testReceiptPrinter() {
  if (!state.settings.receiptPrinterPath.trim()) {
    setStatus("Choose a receipt printer before testing.", "error");
    state.activePage = "printer";
    return;
  }

  try {
    state.status = "Sending test receipt...";
    state.statusKind = "neutral";
    render();
    await invoke("test_print_receipt", { printerPath: state.settings.receiptPrinterPath });
    setStatus("Sent test receipt.", "success");
  } catch (error) {
    setStatus(`Test print failed: ${String(error)}`, "error");
  }
}

async function printReceipt() {
  if (!state.settings.receiptPrinterPath.trim()) {
    setStatus("Choose a receipt printer before printing.", "error");
    state.activePage = "printer";
    return;
  }

  if (
    !hasCustomerData(state.customer) &&
    !hasTicketData(state.ticket) &&
    !state.garments.some(hasGarmentData)
  ) {
    setStatus("Add record data before printing a receipt.", "error");
    state.activePage = "printer";
    return;
  }

  try {
    state.status = "Sending receipt...";
    state.statusKind = "neutral";
    render();
    await invoke("print_receipt", {
      request: {
        printerPath: state.settings.receiptPrinterPath,
        customer: state.customer,
        ticket: state.ticket,
        garments: state.garments,
      },
    });
    setStatus("Sent receipt.", "success");
  } catch (error) {
    setStatus(`Receipt print failed: ${String(error)}`, "error");
  }
}

async function deleteCustomer(accountNumber: string) {
  try {
    cancelQueuedWorkspaceSave();
    state.status = `Deleting customer ${accountNumber}...`;
    state.statusKind = "neutral";
    render();
    await invoke("delete_customer", { request: { accountNumber } });
    await refreshDatabaseSummary();
    if (state.customer.accountNumber === accountNumber || state.ticket.customerAccountNumber === accountNumber) {
      await loadFirstAvailableWorkspace();
    }
    state.status = `Deleted customer ${accountNumber}`;
    state.statusKind = "success";
    render();
  } catch (error) {
    setStatus(`Delete failed: ${String(error)}`, "error");
  }
}

async function deleteTicket(ticketNumber: string) {
  try {
    cancelQueuedWorkspaceSave();
    state.status = `Deleting ticket ${ticketNumber}...`;
    state.statusKind = "neutral";
    render();
    await invoke("delete_ticket", { request: { ticketNumber } });
    await refreshDatabaseSummary();
    if (state.ticket.ticketNumber === ticketNumber) {
      await loadFirstAvailableWorkspace();
    }
    state.status = `Deleted ticket ${ticketNumber}`;
    state.statusKind = "success";
    render();
  } catch (error) {
    setStatus(`Delete failed: ${String(error)}`, "error");
  }
}

async function deleteGarment(ticketNumber: string, garmentId: string) {
  try {
    cancelQueuedWorkspaceSave();
    state.status = `Deleting garment ${garmentId}...`;
    state.statusKind = "neutral";
    render();
    await invoke("delete_garment", { request: { ticketNumber, garmentId } });
    if (state.ticket.ticketNumber === ticketNumber) {
      state.garments = state.garments.filter((garment) => garment.id !== garmentId);
      state.selectedGarmentIndex = Math.min(state.selectedGarmentIndex, Math.max(state.garments.length - 1, 0));
    }
    await refreshDatabaseSummary();
    state.status = `Deleted garment ${garmentId}`;
    state.statusKind = "success";
    render();
  } catch (error) {
    setStatus(`Delete failed: ${String(error)}`, "error");
  }
}

async function deleteEmployee(employeeNumber: string) {
  try {
    cancelQueuedWorkspaceSave();
    state.status = `Deleting employee ${employeeNumber}...`;
    state.statusKind = "neutral";
    render();
    await invoke("delete_employee", { request: { employeeNumber } });
    state.employees = state.employees.filter((employee) => employee.employeeNumber !== employeeNumber);
    state.selectedEmployeeIndex = Math.min(state.selectedEmployeeIndex, Math.max(state.employees.length - 1, 0));
    await refreshDatabaseSummary();
    state.status = `Deleted employee ${employeeNumber}`;
    state.statusKind = "success";
    render();
  } catch (error) {
    setStatus(`Delete failed: ${String(error)}`, "error");
  }
}

function validateExport() {
  if (state.settings.posSystem === "whiteconveyors" && state.settings.exportOperation !== "create") {
    if (!state.settings.outputDirectory.trim()) return "Choose an output folder before exporting.";
    if (state.settings.exportOperation === "employeeDelete") {
      if (!state.employees.some((employee) => employee.employeeNumber.trim())) {
        return "Choose an employee before exporting an employee delete.";
      }
      return "";
    }
    if (!state.customer.accountNumber.trim()) return "Customer account number is required.";
    if (state.settings.exportOperation === "ticketDelete" && !state.ticket.ticketNumber.trim()) {
      return "Ticket number is required.";
    }
    if (state.settings.exportOperation === "garmentDelete") {
      if (!state.ticket.ticketNumber.trim()) return "Ticket number is required.";
      if (!state.garments.some((garment) => garment.id.trim())) {
        return "Add at least one garment number before exporting a garment delete.";
      }
    }
    return "";
  }

  if (!hasExportData(state.customer, state.ticket, state.garments, state.employees)) {
    return "Add at least one record before exporting.";
  }
  if (!state.settings.outputDirectory.trim()) return "Choose an output folder before exporting.";
  if (isEmployeeOnlyCreateExport()) {
    return "";
  }
  if (!state.customer.accountNumber.trim()) return "Customer account number is required.";
  if (!state.ticket.ticketNumber.trim()) return "Ticket number is required.";
  if (state.settings.posSystem === "spot") {
    if (!state.ticket.fullInvoiceNumber.trim()) return "Full invoice number is required.";
    if (!state.ticket.displayInvoiceNumber.trim()) return "Display invoice number is required.";
    if (!state.ticket.dropoffDateTime.trim()) return "Dropoff date/time is required.";
    if (!state.ticket.promisedDateTime.trim()) return "Promised date/time is required.";
  }
  if (state.settings.posSystem === "wincleaners" || state.settings.posSystem === "whiteconveyors") {
    if (!state.ticket.readyDate.trim()) return "Ready date is required.";
    if (!state.ticket.readyTime.trim()) return "Ready time is required.";
  }
  if (state.garments.length === 0) return "Add at least one garment.";
  const incomplete = state.garments.find((garment) => !garment.id.trim() || !garment.description.trim());
  if (incomplete) return "Each garment needs at least an ID and description.";
  if (state.settings.posSystem === "spot") {
    const missingSlot = state.garments.find((garment) => !garment.slotOccupancy.trim());
    if (missingSlot) return "Each SPOT garment needs slot occupancy.";
  }
  if (state.settings.posSystem === "whiteconveyors") {
    const missingServiceType = state.garments.find((garment) => !garment.serviceType.trim());
    if (missingServiceType) return "Each White Conveyors garment needs a service type.";
    const incompleteEmployee = state.employees.find(
      (employee) => !employee.employeeNumber.trim() && employee.employeeName.trim()
    );
    if (incompleteEmployee) return "Each named White Conveyors employee needs an employee number.";
  }
  return "";
}

async function loadSettings() {
  try {
    const loaded = await invoke<Partial<AppSettings>>("load_settings");
    state.settings = { ...defaultSettings, ...loaded };
    const validPosSystems = ["spot", "wincleaners", "whiteconveyors"];
    if (!validPosSystems.includes(state.settings.posSystem)) {
      state.settings.posSystem = "spot";
    }
    if (!isExportOperation(state.settings.exportOperation)) {
      state.settings.exportOperation = "create";
    }
    if (state.settings.posSystem !== "whiteconveyors") {
      state.settings.exportOperation = "create";
    }
  } catch {
    state.settings = { ...defaultSettings };
  }
}

async function loadTicketWorkspace(ticketNumber: string) {
  try {
    const workspace = await invoke<Pick<AppState, "customer" | "ticket" | "garments" | "employees"> | null>(
      "load_ticket_workspace",
      { ticketNumber }
    );
    if (workspace) {
      state.customer = workspace.customer;
      state.ticket = workspace.ticket;
      state.customerDraftActive = true;
      state.ticketDraftActive = true;
      if (!state.ticket.customerAccountNumber) {
        state.ticket.customerAccountNumber = state.customer.accountNumber;
      }
      state.garments = workspace.garments.length > 0 ? workspace.garments : [];
      state.employees = workspace.employees;
      state.selectedGarmentIndex = 0;
      state.selectedEmployeeIndex = 0;
      updateLivePreview();
    }
  } catch {
    // Keep current workspace if selected ticket cannot be loaded.
  }
}

async function loadFirstAvailableWorkspace() {
  resetWorkspaceDraft();
}

function resetWorkspaceDraft() {
  state.customer = defaultCustomer();
  state.ticket = defaultTicket(state.customer.accountNumber);
  state.garments = defaultGarments();
  state.employees = defaultEmployees();
  state.customerDraftActive = false;
  state.ticketDraftActive = false;
  state.selectedGarmentIndex = 0;
  state.selectedEmployeeIndex = 0;
  updateLivePreview();
}

async function saveSettings() {
  try {
    await invoke("save_settings", { settings: state.settings });
  } catch {
    // Settings persistence should not interrupt form editing.
  }
}

async function saveCustomer() {
  if (!state.customer.accountNumber.trim()) {
    return;
  }

  try {
    await invoke("save_customer", { customer: state.customer });
  } catch {
    // Customer editing should continue even if persistence fails.
  }
}

async function saveSelectedEmployee() {
  const employee = state.employees[state.selectedEmployeeIndex];
  if (!employee?.employeeNumber.trim()) {
    return;
  }

  try {
    await invoke("save_employee", { employee });
  } catch {
    // Employee editing should continue even if persistence fails.
  }
}

async function saveWhiteConveyorsEmployees() {
  for (const employee of state.employees) {
    if (!employee.employeeNumber.trim()) {
      continue;
    }

    try {
      await invoke("save_employee", { employee });
    } catch {
      // Export should still be attempted even if local persistence fails.
    }
  }
}

async function saveWorkspace() {
  if (!state.customer.accountNumber.trim() || !state.ticket.ticketNumber.trim()) {
    return;
  }

  try {
    await invoke("save_workspace", {
      workspace: {
        customer: state.customer,
        ticket: state.ticket,
        garments: state.garments,
        employees: state.employees,
      },
    });
  } catch {
    // Form edits should continue even if persistence fails.
  }
}

function queueCurrentRecordSave(path: string) {
  window.clearTimeout(workspaceSaveTimer);
  workspaceSaveTimer = window.setTimeout(() => {
    void saveCurrentRecord(path).then(refreshDatabaseSummary);
  }, 300);
}

async function saveCurrentRecord(path: string) {
  if (isDeleteExportMode()) {
    return;
  }

  if (path.startsWith("customer.")) {
    await saveCustomer();
    return;
  }

  if (path.startsWith("employee.")) {
    await saveSelectedEmployee();
    return;
  }

  await saveWorkspace();
}

function cancelQueuedWorkspaceSave() {
  window.clearTimeout(workspaceSaveTimer);
  workspaceSaveTimer = undefined;
}

async function refreshDatabaseSummary() {
  try {
    state.databaseSummary = await invoke("load_database_summary");
    if (state.activePage === "database") {
      render();
    }
  } catch {
    state.databaseSummary = null;
  }
}

function updateLivePreview() {
  const adapter = adapters[state.settings.posSystem];
  state.preview = adapter.formatExport(
    state.customer,
    state.ticket,
    state.garments,
    state.employees,
    state.settings.exportOperation
  );
  document.querySelector<HTMLPreElement>(".preview")?.replaceChildren(
    document.createTextNode(state.preview)
  );
}

function isExportOperation(value: string): value is ExportOperation {
  return (
    value === "create" ||
    value === "customerDelete" ||
    value === "ticketDelete" ||
    value === "garmentDelete" ||
    value === "employeeDelete"
  );
}

function isDeleteExportMode() {
  return state.settings.posSystem === "whiteconveyors" && state.settings.exportOperation !== "create";
}

function isEmployeeOnlyCreateExport() {
  return (
    state.settings.posSystem === "whiteconveyors" &&
    state.settings.exportOperation === "create" &&
    !state.customer.accountNumber.trim() &&
    !state.ticket.ticketNumber.trim() &&
    !state.garments.some((garment) => garment.id.trim()) &&
    state.employees.some((employee) => employee.employeeNumber.trim())
  );
}

function setStatus(message: string, kind: "success" | "error" | "neutral") {
  state.status = message;
  state.statusKind = kind;
  render();
}

void loadSettings()
  .then(refreshDatabaseSummary)
  .then(() => {
    bindGlobalDeleteHandler();
    render();
  });
