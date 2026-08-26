import type { AppState, Employee } from "../types";
import { escapeHtml, field } from "../ui/html";

export function renderEmployeesPage(state: AppState) {
  const selectedEmployee = state.employees[state.selectedEmployeeIndex] ?? state.employees[0];
  const isWhiteConveyors = state.settings.posSystem === "whiteconveyors";

  return `
    <section class="page employees-page">
      <div class="page-heading with-actions">
        <div>
          <p class="eyebrow">White Conveyors</p>
          <h1>Create Employees</h1>
          <p>${isWhiteConveyors
            ? "Employees become EMPLOYEE_CREATE rows at the top of the Comp-U-Sort POS.txt export."
            : "Employee rows are only exported when White Conveyors is selected."}</p>
        </div>
        <button class="secondary-button" data-action="add-employee">+ Add Employee</button>
      </div>

      <div class="record-editor">
        <div class="list-panel record-list">
          <div class="panel-title">
            <h2>Employees</h2>
            <span>${state.employees.length} records</span>
          </div>
          <div class="record-list-items">
            ${state.employees.length === 0 ? `<div class="empty-list">No employees added</div>` : state.employees.map((employee, index) => employeeButton(employee, index, index === state.selectedEmployeeIndex)).join("")}
          </div>
        </div>
        ${selectedEmployee ? employeeForm(selectedEmployee) : emptyEmployee()}
      </div>
    </section>
  `;
}

function employeeForm(selectedEmployee: Employee) {
  return `
    <div class="form-surface record-form">
          <div class="section-title">
            <h2>Employee Details</h2>
            <span>Comp-U-Sort employee create fields</span>
          </div>
          <div class="form-grid aligned-grid two-column-grid">
            ${field("employee.employeeNumber", "Employee Number", selectedEmployee.employeeNumber, "Used by Comp-U-Sort as the employee login number.")}
            ${field("employee.employeeName", "Employee Name", selectedEmployee.employeeName)}
          </div>
        </div>
  `;
}

function emptyEmployee() {
  return `
    <div class="empty-state record-form">
      <h2>No employee selected</h2>
      <p>Add an employee when you are ready to enter employee details.</p>
      <button class="primary-button" data-action="add-employee">Add Employee</button>
    </div>
  `;
}

function employeeButton(employee: Employee, index: number, selected: boolean) {
  return `
    <button class="garment-button ${selected ? "active" : ""}" data-action="select-employee" data-index="${index}">
      <strong>${escapeHtml(employee.employeeNumber || `Employee ${index + 1}`)}</strong>
      <span>${escapeHtml(employee.employeeName || "No name")}</span>
    </button>
  `;
}
