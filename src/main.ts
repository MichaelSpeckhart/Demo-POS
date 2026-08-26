import { invoke } from "@tauri-apps/api/core";
import { adapters } from "./formatters";
import { renderCustomerPage } from "./pages/customer";
import { renderDatabasePage } from "./pages/database";
import { renderEmployeesPage } from "./pages/employees";
import { renderExportPage } from "./pages/export";
import { renderFoldersPage } from "./pages/folders";
import { renderGarmentsPage } from "./pages/garments";
import { renderPosSystemPage } from "./pages/posSystem";
import { renderTicketPage } from "./pages/ticket";
import { resolveOutputFileName } from "./outputFileName";
import AppLogo from "./assets/Logo1.png";
import type {
  AppSettings,
  AppState,
  Customer,
  Employee,
  Garment,
  PageId,
  Ticket,
  WriteExportRequest,
} from "./types";

const defaultSettings: AppSettings = {
  posSystem: "spot",
  inputDirectory: "",
  outputDirectory: "",
  outputFileName: "",
};

function defaultCustomer(): Customer {
  return {
    accountNumber: "14684",
    phoneNumber: "555-0104",
    firstName: "Alex",
    lastName: "Rivera",
    pin: "1234",
    address1: "100 Main St",
    address2: "",
    city: "Rochester",
    state: "NY",
    zipCode: "14604",
  };
}

function defaultTicket(customerAccountNumber = "14684"): Ticket {
  return {
    customerAccountNumber,
    ticketNumber: "01040363",
    fullInvoiceNumber: ".DCDC03-090374",
    displayInvoiceNumber: "03-090374",
    balanceDue: "12.34",
    dropoffDateTime: "2025-04-18T14:27:49",
    promisedDateTime: "2025-04-22T17:00:00",
    comments: "Do not crease",
    readyDate: "04/03/2026",
    readyTime: "05:00:00 PM",
    plant: "01",
    route: "1111",
    routeStop: "23",
    store: "01",
  };
}

function defaultGarments(): Garment[] {
  return [
    {
      id: "T1476237",
      description: "Ld Bag",
      slotOccupancy: "33",
      servicePrice: "1",
      serviceType: "LD",
      garmentType: "T001",
      color: "Red",
      fabric: "F001",
    },
  ];
}

function defaultEmployees(): Employee[] {
  return [
    {
      employeeNumber: "9001",
      employeeName: "Load Station 1",
    },
  ];
}

const state: AppState = {
  activePage: "pos",
  settings: { ...defaultSettings },
  customer: defaultCustomer(),
  ticket: defaultTicket(),
  garments: defaultGarments(),
  employees: defaultEmployees(),
  selectedGarmentIndex: 0,
  selectedEmployeeIndex: 0,
  preview: "",
  status: "",
  statusKind: "neutral",
  databaseSummary: null,
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
  { id: "employees", label: "Employees", render: renderEmployeesPage },
  { id: "customer", label: "Customer", render: renderCustomerPage },
  { id: "ticket", label: "Ticket", render: renderTicketPage },
  { id: "garments", label: "Garments", render: renderGarmentsPage },
  { id: "export", label: "Export", render: renderExportPage },
  { id: "database", label: "Database", render: renderDatabasePage },
];

function render() {
  const adapter = adapters[state.settings.posSystem];
  state.preview = adapter.formatExport(state.customer, state.ticket, state.garments, state.employees);
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

  document.querySelectorAll<HTMLInputElement>("[data-bind]").forEach((input) => {
    input.addEventListener("input", () => {
      updateValue(input.dataset.bind ?? "", input.value);
      updateLivePreview();
      if (input.dataset.bind?.startsWith("settings.")) {
        void saveSettings();
      } else {
        queueWorkspaceSave();
      }
    });
    input.addEventListener("change", () => {
      updateValue(input.dataset.bind ?? "", input.value);
      updateLivePreview();
      if (input.dataset.bind?.startsWith("settings.")) {
        void saveSettings();
      } else {
        void saveWorkspace().then(refreshDatabaseSummary);
      }
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-action='select-customer']").forEach((select) => {
    select.addEventListener("change", async () => {
      const accountNumber = select.value;
      state.ticket.customerAccountNumber = accountNumber;
      const customer = await invoke<Customer | null>("load_customer", { accountNumber });
      if (customer) {
        state.customer = customer;
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
      if (!ticketNumber) return;
      await loadTicketWorkspace(ticketNumber);
      state.status = "";
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-action='select-pos']").forEach((button) => {
    button.addEventListener("click", () => {
      const pos = button.dataset.pos;
      if (pos === "wincleaners" || pos === "spot" || pos === "whiteconveyors") {
        state.settings.posSystem = pos;
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
    state.garments.push({
      id: `T${Math.floor(1000000 + Math.random() * 9000000)}`,
      description: "Pressed Shirt",
      slotOccupancy: String(30 + state.garments.length),
      servicePrice: "1",
      serviceType: "DC",
      garmentType: "",
      color: "Blue",
      fabric: "",
    });
    state.selectedGarmentIndex = state.garments.length - 1;
    state.status = "";
    render();
    void saveWorkspace().then(refreshDatabaseSummary);
  });

  document.querySelector<HTMLButtonElement>("[data-action='add-employee']")?.addEventListener("click", () => {
    state.employees.push({
      employeeNumber: String(9001 + state.employees.length),
      employeeName: "",
    });
    state.selectedEmployeeIndex = state.employees.length - 1;
    state.status = "";
    render();
    void saveWorkspace().then(refreshDatabaseSummary);
  });

  document.querySelector<HTMLButtonElement>("[data-action='export']")?.addEventListener("click", () => {
    void exportFile();
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
      "inputDirectory" | "outputDirectory" | "outputFileName"
    >;
    state.settings[key] = value;
    return;
  }

  if (path.startsWith("customer.")) {
    const key = path.replace("customer.", "") as keyof Customer;
    state.customer[key] = value;
    if (key === "accountNumber") {
      state.ticket.customerAccountNumber = value;
    }
    return;
  }

  if (path.startsWith("ticket.")) {
    const key = path.replace("ticket.", "") as keyof Ticket;
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
    contents: adapter.formatExport(state.customer, state.ticket, state.garments, state.employees),
  };

  try {
    await saveWorkspace();
    const path = await invoke<string>("write_export_file", { request });
    await refreshDatabaseSummary();
    setStatus(`Wrote ${path}`, "success");
  } catch (error) {
    setStatus(`Export failed: ${String(error)}`, "error");
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
    if (state.employees.length === 0) {
      state.employees = defaultEmployees();
    }
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
  if (!state.settings.outputDirectory.trim()) return "Choose an output folder before exporting.";
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
  } catch {
    state.settings = { ...defaultSettings };
  }
}

async function loadWorkspace() {
  try {
    const workspace = await invoke<Pick<AppState, "customer" | "ticket" | "garments" | "employees"> | null>(
      "load_workspace"
    );
    if (workspace) {
      state.customer = workspace.customer;
      state.ticket = workspace.ticket;
      if (!state.ticket.customerAccountNumber) {
        state.ticket.customerAccountNumber = state.customer.accountNumber;
      }
      state.garments = workspace.garments.length > 0 ? workspace.garments : state.garments;
      state.employees = workspace.employees.length > 0 ? workspace.employees : state.employees;
      state.selectedGarmentIndex = 0;
      state.selectedEmployeeIndex = 0;
    }
  } catch {
    // Empty or unavailable database should fall back to the sample workspace.
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
      if (!state.ticket.customerAccountNumber) {
        state.ticket.customerAccountNumber = state.customer.accountNumber;
      }
      state.garments = workspace.garments.length > 0 ? workspace.garments : [];
      state.employees = workspace.employees.length > 0 ? workspace.employees : state.employees;
      state.selectedGarmentIndex = 0;
      state.selectedEmployeeIndex = 0;
      updateLivePreview();
    }
  } catch {
    // Keep current workspace if selected ticket cannot be loaded.
  }
}

async function loadFirstAvailableWorkspace() {
  const firstTicket = state.databaseSummary?.tickets[0];
  if (firstTicket) {
    await loadTicketWorkspace(firstTicket.ticketNumber);
    return;
  }

  state.customer = defaultCustomer();
  state.ticket = defaultTicket(state.customer.accountNumber);
  state.garments = defaultGarments();
  state.employees = defaultEmployees();
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

async function saveWorkspace() {
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

function queueWorkspaceSave() {
  window.clearTimeout(workspaceSaveTimer);
  workspaceSaveTimer = window.setTimeout(() => {
    void saveWorkspace().then(refreshDatabaseSummary);
  }, 300);
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
  state.preview = adapter.formatExport(state.customer, state.ticket, state.garments, state.employees);
  document.querySelector<HTMLPreElement>(".preview")?.replaceChildren(
    document.createTextNode(state.preview)
  );
}

function setStatus(message: string, kind: "success" | "error" | "neutral") {
  state.status = message;
  state.statusKind = kind;
  render();
}

void loadSettings()
  .then(loadWorkspace)
  .then(refreshDatabaseSummary)
  .then(() => {
    bindGlobalDeleteHandler();
    render();
  });
