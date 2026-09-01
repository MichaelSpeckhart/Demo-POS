use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    pos_system: String,
    input_directory: String,
    #[serde(default)]
    input_file_name: String,
    output_directory: String,
    #[serde(default)]
    output_file_name: String,
    #[serde(default = "default_export_operation")]
    export_operation: String,
    #[serde(default)]
    receipt_printer_path: String,
    #[serde(default = "default_receipt_ticket_template")]
    receipt_ticket_template: TicketTemplateConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pos_system: "spot".to_string(),
            input_directory: String::new(),
            input_file_name: String::new(),
            output_directory: String::new(),
            output_file_name: String::new(),
            export_operation: default_export_operation(),
            receipt_printer_path: String::new(),
            receipt_ticket_template: default_receipt_ticket_template(),
        }
    }
}

fn default_export_operation() -> String {
    "create".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TicketField {
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    show_barcode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TicketTemplateConfig {
    #[serde(default = "default_receipt_header_text")]
    header_text: String,
    #[serde(default = "default_receipt_footer_text")]
    footer_text: String,
    #[serde(default = "default_receipt_ticket_fields")]
    fields: Vec<TicketField>,
}

fn default_receipt_header_text() -> String {
    "Demo POS".to_string()
}

fn default_receipt_footer_text() -> String {
    String::new()
}

fn default_receipt_ticket_template() -> TicketTemplateConfig {
    TicketTemplateConfig {
        header_text: default_receipt_header_text(),
        footer_text: default_receipt_footer_text(),
        fields: default_receipt_ticket_fields(),
    }
}

fn default_receipt_ticket_fields() -> Vec<TicketField> {
    receipt_field_defaults()
        .into_iter()
        .map(|(id, label, enabled, show_barcode)| TicketField {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            show_barcode,
        })
        .collect()
}

fn receipt_field_defaults() -> Vec<(&'static str, &'static str, bool, bool)> {
    vec![
        ("customerName", "Customer Name", true, false),
        ("customerIdentifier", "Customer Account", true, false),
        ("customerPhone", "Customer Phone", true, false),
        ("ticketNumber", "Ticket Number", true, true),
        ("invoiceNumber", "Invoice Number", true, false),
        ("balanceDue", "Balance Due", false, false),
        ("dropoffDate", "Dropoff Date", false, false),
        ("pickupDate", "Pickup Date", true, false),
        ("readyDate", "Ready Date", true, false),
        ("numItems", "Number of Items", false, false),
        ("itemList", "Garment List", true, false),
        ("comments", "Comments", true, false),
        ("ticketMessage", "Ticket Message", true, false),
    ]
}

impl AppSettings {
    fn normalize(&mut self) {
        self.receipt_ticket_template =
            normalize_receipt_ticket_template(self.receipt_ticket_template.clone());
    }
}

fn normalize_receipt_ticket_template(template: TicketTemplateConfig) -> TicketTemplateConfig {
    let defaults = default_receipt_ticket_fields();
    let mut seen = HashSet::new();
    let mut fields = Vec::new();

    for mut field in template.fields {
        if !defaults.iter().any(|default| default.id == field.id) || !seen.insert(field.id.clone())
        {
            continue;
        }
        if field.label.trim().is_empty() {
            field.label = receipt_field_label(&field.id)
                .map(str::to_string)
                .unwrap_or_else(|| field.id.clone());
        }
        fields.push(field);
    }

    for field in defaults {
        if seen.insert(field.id.clone()) {
            fields.push(field);
        }
    }

    if fields.is_empty() {
        fields = default_receipt_ticket_fields();
    }

    TicketTemplateConfig {
        header_text: template.header_text,
        footer_text: template.footer_text,
        fields,
    }
}

fn receipt_field_label(id: &str) -> Option<&'static str> {
    receipt_field_defaults()
        .into_iter()
        .find(|(field_id, _, _, _)| *field_id == id)
        .map(|(_, label, _, _)| label)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteExportRequest {
    pos_system: String,
    ticket_number: String,
    output_directory: String,
    file_name: String,
    contents: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteCustomerRequest {
    account_number: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteTicketRequest {
    ticket_number: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteGarmentRequest {
    ticket_number: String,
    garment_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteEmployeeRequest {
    employee_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Customer {
    account_number: String,
    phone_number: String,
    first_name: String,
    last_name: String,
    pin: String,
    address1: String,
    address2: String,
    city: String,
    state: String,
    zip_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ticket {
    #[serde(default)]
    customer_account_number: String,
    ticket_number: String,
    full_invoice_number: String,
    display_invoice_number: String,
    balance_due: String,
    dropoff_date_time: String,
    promised_date_time: String,
    comments: String,
    ready_date: String,
    ready_time: String,
    plant: String,
    route: String,
    route_stop: String,
    store: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Garment {
    id: String,
    description: String,
    slot_occupancy: String,
    service_price: String,
    service_type: String,
    #[serde(default)]
    garment_type: String,
    color: String,
    #[serde(default)]
    fabric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Employee {
    employee_number: String,
    employee_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptPrinterInfo {
    path: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrintReceiptRequest {
    printer_path: String,
    customer: Customer,
    ticket: Ticket,
    garments: Vec<Garment>,
    #[serde(default = "default_receipt_ticket_template")]
    receipt_ticket_template: TicketTemplateConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReceiptPrintCommand {
    account_number: String,
    ticket_number: String,
    ticket_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceData {
    customer: Customer,
    ticket: Ticket,
    garments: Vec<Garment>,
    #[serde(default)]
    employees: Vec<Employee>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseSummary {
    path: String,
    customer_count: i64,
    employee_count: i64,
    ticket_count: i64,
    garment_count: i64,
    export_count: i64,
    last_export_path: Option<String>,
    last_export_at: Option<String>,
    customers: Vec<CustomerRecord>,
    employees: Vec<EmployeeRecord>,
    tickets: Vec<TicketRecord>,
    garments: Vec<GarmentRecord>,
    exports: Vec<ExportRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomerRecord {
    account_number: String,
    phone_number: String,
    first_name: String,
    last_name: String,
    pin: String,
    address1: String,
    address2: String,
    name: String,
    city: String,
    state: String,
    zip_code: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TicketRecord {
    ticket_number: String,
    customer_account_number: String,
    full_invoice_number: String,
    display_invoice_number: String,
    promised_date_time: String,
    ready_date: String,
    ready_time: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GarmentRecord {
    ticket_number: String,
    garment_id: String,
    description: String,
    slot_occupancy: String,
    service_type: String,
    garment_type: String,
    color: String,
    fabric: String,
    position: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmployeeRecord {
    employee_number: String,
    employee_name: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportRecord {
    id: i64,
    pos_system: String,
    ticket_number: String,
    file_path: String,
    payload: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputFileEventRecord {
    id: i64,
    path: String,
    modified_at: String,
    file_size: i64,
    command: String,
    status: String,
    message: String,
    processed_at: String,
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir.join("settings.json"))
}

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    Ok(data_dir.join("demo-pos.sqlite3"))
}

fn open_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let conn = Connection::open(db_path(app)?).map_err(|e| e.to_string())?;
    migrate_db(&conn)?;
    Ok(conn)
}

fn migrate_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS customers (
            account_number TEXT PRIMARY KEY,
            phone_number TEXT NOT NULL DEFAULT '',
            first_name TEXT NOT NULL DEFAULT '',
            last_name TEXT NOT NULL DEFAULT '',
            pin TEXT NOT NULL DEFAULT '',
            address1 TEXT NOT NULL DEFAULT '',
            address2 TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            state TEXT NOT NULL DEFAULT '',
            zip_code TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS employees (
            employee_number TEXT PRIMARY KEY,
            employee_name TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tickets (
            ticket_number TEXT PRIMARY KEY,
            customer_account_number TEXT NOT NULL,
            full_invoice_number TEXT NOT NULL DEFAULT '',
            display_invoice_number TEXT NOT NULL DEFAULT '',
            balance_due TEXT NOT NULL DEFAULT '',
            dropoff_date_time TEXT NOT NULL DEFAULT '',
            promised_date_time TEXT NOT NULL DEFAULT '',
            comments TEXT NOT NULL DEFAULT '',
            ready_date TEXT NOT NULL DEFAULT '',
            ready_time TEXT NOT NULL DEFAULT '',
            plant TEXT NOT NULL DEFAULT '',
            route TEXT NOT NULL DEFAULT '',
            route_stop TEXT NOT NULL DEFAULT '',
            store TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL,
            FOREIGN KEY(customer_account_number) REFERENCES customers(account_number)
        );

        CREATE TABLE IF NOT EXISTS garments (
            row_id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_number TEXT NOT NULL,
            garment_id TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            slot_occupancy TEXT NOT NULL DEFAULT '',
            service_price TEXT NOT NULL DEFAULT '',
            service_type TEXT NOT NULL DEFAULT '',
            garment_type TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '',
            fabric TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(ticket_number, garment_id),
            FOREIGN KEY(ticket_number) REFERENCES tickets(ticket_number) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS exports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pos_system TEXT NOT NULL,
            ticket_number TEXT NOT NULL,
            file_path TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS input_file_events (
            path TEXT PRIMARY KEY,
            modified_at TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            command TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT NOT NULL,
            processed_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS input_file_event_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            command TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT NOT NULL,
            processed_at TEXT NOT NULL
        );
        "#,
    )
    .map_err(|e| e.to_string())?;

    add_column_if_missing(conn, "garments", "garment_type", "TEXT NOT NULL DEFAULT ''")?;
    add_column_if_missing(conn, "garments", "fabric", "TEXT NOT NULL DEFAULT ''")?;
    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if columns.iter().any(|existing| existing == column) {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn now_epoch_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn read_settings(app: &tauri::AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut settings: AppSettings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    settings.normalize();
    Ok(settings)
}

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, mut settings: AppSettings) -> Result<(), String> {
    settings.normalize();
    let path = settings_path(&app)?;
    let data = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_workspace(app: tauri::AppHandle) -> Result<Option<WorkspaceData>, String> {
    let conn = open_db(&app)?;
    let current_ticket = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'current_ticket_number'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let Some(ticket_number) = current_ticket else {
        return Ok(None);
    };

    load_workspace_by_ticket_number(&conn, &ticket_number)
}

#[tauri::command]
fn load_ticket_workspace(
    app: tauri::AppHandle,
    ticket_number: String,
) -> Result<Option<WorkspaceData>, String> {
    let conn = open_db(&app)?;
    load_workspace_by_ticket_number(&conn, &ticket_number)
}

fn load_workspace_by_ticket_number(
    conn: &Connection,
    ticket_number: &str,
) -> Result<Option<WorkspaceData>, String> {
    let ticket = conn
        .query_row(
            r#"
            SELECT customer_account_number, ticket_number, full_invoice_number, display_invoice_number, balance_due,
                   dropoff_date_time, promised_date_time, comments, ready_date, ready_time,
                   plant, route, route_stop, store
            FROM tickets
            WHERE ticket_number = ?1
            "#,
            params![ticket_number],
            |row| {
                Ok(Ticket {
                    customer_account_number: row.get(0)?,
                    ticket_number: row.get(1)?,
                    full_invoice_number: row.get(2)?,
                    display_invoice_number: row.get(3)?,
                    balance_due: row.get(4)?,
                    dropoff_date_time: row.get(5)?,
                    promised_date_time: row.get(6)?,
                    comments: row.get(7)?,
                    ready_date: row.get(8)?,
                    ready_time: row.get(9)?,
                    plant: row.get(10)?,
                    route: row.get(11)?,
                    route_stop: row.get(12)?,
                    store: row.get(13)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let Some(ticket) = ticket else {
        return Ok(None);
    };

    let customer = conn
        .query_row(
            r#"
            SELECT c.account_number, c.phone_number, c.first_name, c.last_name, c.pin,
                   c.address1, c.address2, c.city, c.state, c.zip_code
            FROM customers c
            INNER JOIN tickets t ON t.customer_account_number = c.account_number
            WHERE t.ticket_number = ?1
            "#,
            params![ticket.ticket_number],
            |row| {
                Ok(Customer {
                    account_number: row.get(0)?,
                    phone_number: row.get(1)?,
                    first_name: row.get(2)?,
                    last_name: row.get(3)?,
                    pin: row.get(4)?,
                    address1: row.get(5)?,
                    address2: row.get(6)?,
                    city: row.get(7)?,
                    state: row.get(8)?,
                    zip_code: row.get(9)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT garment_id, description, slot_occupancy, service_price, service_type,
                   garment_type, color, fabric
            FROM garments
            WHERE ticket_number = ?1
            ORDER BY position ASC, row_id ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let garments = stmt
        .query_map(params![ticket.ticket_number], |row| {
            Ok(Garment {
                id: row.get(0)?,
                description: row.get(1)?,
                slot_occupancy: row.get(2)?,
                service_price: row.get(3)?,
                service_type: row.get(4)?,
                garment_type: row.get(5)?,
                color: row.get(6)?,
                fabric: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let employees = load_employees(conn)?;

    Ok(Some(WorkspaceData {
        customer,
        ticket,
        garments,
        employees,
    }))
}

#[tauri::command]
fn load_customer(
    app: tauri::AppHandle,
    account_number: String,
) -> Result<Option<Customer>, String> {
    let conn = open_db(&app)?;
    conn.query_row(
        r#"
        SELECT account_number, phone_number, first_name, last_name, pin,
               address1, address2, city, state, zip_code
        FROM customers
        WHERE account_number = ?1
        "#,
        params![account_number],
        |row| {
            Ok(Customer {
                account_number: row.get(0)?,
                phone_number: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                pin: row.get(4)?,
                address1: row.get(5)?,
                address2: row.get(6)?,
                city: row.get(7)?,
                state: row.get(8)?,
                zip_code: row.get(9)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}

fn load_employees(conn: &Connection) -> Result<Vec<Employee>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT employee_number, employee_name
            FROM employees
            ORDER BY employee_number ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Employee {
                employee_number: row.get(0)?,
                employee_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_customer(app: tauri::AppHandle, customer: Customer) -> Result<(), String> {
    if customer.account_number.trim().is_empty() {
        return Ok(());
    }

    let conn = open_db(&app)?;
    let now = now_epoch_seconds();
    conn.execute(
        r#"
        INSERT INTO customers (
            account_number, phone_number, first_name, last_name, pin,
            address1, address2, city, state, zip_code, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(account_number) DO UPDATE SET
            phone_number = excluded.phone_number,
            first_name = excluded.first_name,
            last_name = excluded.last_name,
            pin = excluded.pin,
            address1 = excluded.address1,
            address2 = excluded.address2,
            city = excluded.city,
            state = excluded.state,
            zip_code = excluded.zip_code,
            updated_at = excluded.updated_at
        "#,
        params![
            &customer.account_number,
            &customer.phone_number,
            &customer.first_name,
            &customer.last_name,
            &customer.pin,
            &customer.address1,
            &customer.address2,
            &customer.city,
            &customer.state,
            &customer.zip_code,
            &now,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_employee(app: tauri::AppHandle, employee: Employee) -> Result<(), String> {
    if employee.employee_number.trim().is_empty() {
        return Ok(());
    }

    let conn = open_db(&app)?;
    let now = now_epoch_seconds();
    conn.execute(
        r#"
        INSERT INTO employees (employee_number, employee_name, updated_at)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(employee_number) DO UPDATE SET
            employee_name = excluded.employee_name,
            updated_at = excluded.updated_at
        "#,
        params![&employee.employee_number, &employee.employee_name, &now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_workspace(app: tauri::AppHandle, workspace: WorkspaceData) -> Result<(), String> {
    let mut conn = open_db(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let now = now_epoch_seconds();

    tx.execute(
        r#"
        INSERT INTO customers (
            account_number, phone_number, first_name, last_name, pin,
            address1, address2, city, state, zip_code, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(account_number) DO UPDATE SET
            phone_number = excluded.phone_number,
            first_name = excluded.first_name,
            last_name = excluded.last_name,
            pin = excluded.pin,
            address1 = excluded.address1,
            address2 = excluded.address2,
            city = excluded.city,
            state = excluded.state,
            zip_code = excluded.zip_code,
            updated_at = excluded.updated_at
        "#,
        params![
            &workspace.customer.account_number,
            &workspace.customer.phone_number,
            &workspace.customer.first_name,
            &workspace.customer.last_name,
            &workspace.customer.pin,
            &workspace.customer.address1,
            &workspace.customer.address2,
            &workspace.customer.city,
            &workspace.customer.state,
            &workspace.customer.zip_code,
            &now,
        ],
    )
    .map_err(|e| e.to_string())?;

    let ticket_customer_account_number =
        if workspace.ticket.customer_account_number.trim().is_empty() {
            workspace.customer.account_number.as_str()
        } else {
            workspace.ticket.customer_account_number.as_str()
        };

    tx.execute(
        r#"
        INSERT INTO tickets (
            ticket_number, customer_account_number, full_invoice_number,
            display_invoice_number, balance_due, dropoff_date_time, promised_date_time,
            comments, ready_date, ready_time, plant, route, route_stop, store, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(ticket_number) DO UPDATE SET
            customer_account_number = excluded.customer_account_number,
            full_invoice_number = excluded.full_invoice_number,
            display_invoice_number = excluded.display_invoice_number,
            balance_due = excluded.balance_due,
            dropoff_date_time = excluded.dropoff_date_time,
            promised_date_time = excluded.promised_date_time,
            comments = excluded.comments,
            ready_date = excluded.ready_date,
            ready_time = excluded.ready_time,
            plant = excluded.plant,
            route = excluded.route,
            route_stop = excluded.route_stop,
            store = excluded.store,
            updated_at = excluded.updated_at
        "#,
        params![
            &workspace.ticket.ticket_number,
            ticket_customer_account_number,
            &workspace.ticket.full_invoice_number,
            &workspace.ticket.display_invoice_number,
            &workspace.ticket.balance_due,
            &workspace.ticket.dropoff_date_time,
            &workspace.ticket.promised_date_time,
            &workspace.ticket.comments,
            &workspace.ticket.ready_date,
            &workspace.ticket.ready_time,
            &workspace.ticket.plant,
            &workspace.ticket.route,
            &workspace.ticket.route_stop,
            &workspace.ticket.store,
            &now,
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM garments WHERE ticket_number = ?1",
        params![&workspace.ticket.ticket_number],
    )
    .map_err(|e| e.to_string())?;

    for (position, garment) in workspace.garments.iter().enumerate() {
        if garment.id.trim().is_empty() {
            continue;
        }

        tx.execute(
            r#"
            INSERT INTO garments (
                ticket_number, garment_id, description, slot_occupancy,
                service_price, service_type, garment_type, color, fabric, position, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                &workspace.ticket.ticket_number,
                &garment.id,
                &garment.description,
                &garment.slot_occupancy,
                &garment.service_price,
                &garment.service_type,
                &garment.garment_type,
                &garment.color,
                &garment.fabric,
                position as i64,
                &now,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for employee in workspace.employees.iter() {
        if employee.employee_number.trim().is_empty() {
            continue;
        }

        tx.execute(
            r#"
            INSERT INTO employees (employee_number, employee_name, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(employee_number) DO UPDATE SET
                employee_name = excluded.employee_name,
                updated_at = excluded.updated_at
            "#,
            params![&employee.employee_number, &employee.employee_name, &now],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('current_ticket_number', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![&workspace.ticket.ticket_number],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_database_summary(app: tauri::AppHandle) -> Result<DatabaseSummary, String> {
    let path = db_path(&app)?;
    let conn = open_db(&app)?;
    let customer_count = table_count(&conn, "customers")?;
    let employee_count = table_count(&conn, "employees")?;
    let ticket_count = table_count(&conn, "tickets")?;
    let garment_count = table_count(&conn, "garments")?;
    let export_count = table_count(&conn, "exports")?;
    let last_export = conn
        .query_row(
            "SELECT file_path, created_at FROM exports ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let customers = load_customer_records(&conn)?;
    let employees = load_employee_records(&conn)?;
    let tickets = load_ticket_records(&conn)?;
    let garments = load_garment_records(&conn)?;
    let exports = load_export_records(&conn)?;

    Ok(DatabaseSummary {
        path: path.to_string_lossy().to_string(),
        customer_count,
        employee_count,
        ticket_count,
        garment_count,
        export_count,
        last_export_path: last_export.as_ref().map(|value| value.0.clone()),
        last_export_at: last_export.map(|value| value.1),
        customers,
        employees,
        tickets,
        garments,
        exports,
    })
}

fn load_customer_records(conn: &Connection) -> Result<Vec<CustomerRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT account_number, phone_number, first_name, last_name, pin,
                   address1, address2, first_name || ' ' || last_name AS name,
                   city, state, zip_code, updated_at
            FROM customers
            ORDER BY updated_at DESC, account_number ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CustomerRecord {
                account_number: row.get(0)?,
                phone_number: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                pin: row.get(4)?,
                address1: row.get(5)?,
                address2: row.get(6)?,
                name: row.get(7)?,
                city: row.get(8)?,
                state: row.get(9)?,
                zip_code: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn load_employee_records(conn: &Connection) -> Result<Vec<EmployeeRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT employee_number, employee_name, updated_at
            FROM employees
            ORDER BY updated_at DESC, employee_number ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EmployeeRecord {
                employee_number: row.get(0)?,
                employee_name: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn load_ticket_records(conn: &Connection) -> Result<Vec<TicketRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT ticket_number, customer_account_number, full_invoice_number,
                   display_invoice_number, promised_date_time, ready_date, ready_time, updated_at
            FROM tickets
            ORDER BY updated_at DESC, ticket_number ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TicketRecord {
                ticket_number: row.get(0)?,
                customer_account_number: row.get(1)?,
                full_invoice_number: row.get(2)?,
                display_invoice_number: row.get(3)?,
                promised_date_time: row.get(4)?,
                ready_date: row.get(5)?,
                ready_time: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn load_garment_records(conn: &Connection) -> Result<Vec<GarmentRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT ticket_number, garment_id, description, slot_occupancy,
                   service_type, garment_type, color, fabric, position
            FROM garments
            ORDER BY ticket_number ASC, position ASC, row_id ASC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(GarmentRecord {
                ticket_number: row.get(0)?,
                garment_id: row.get(1)?,
                description: row.get(2)?,
                slot_occupancy: row.get(3)?,
                service_type: row.get(4)?,
                garment_type: row.get(5)?,
                color: row.get(6)?,
                fabric: row.get(7)?,
                position: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn load_export_records(conn: &Connection) -> Result<Vec<ExportRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, pos_system, ticket_number, file_path, payload, created_at
            FROM exports
            ORDER BY id DESC
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ExportRecord {
                id: row.get(0)?,
                pos_system: row.get(1)?,
                ticket_number: row.get(2)?,
                file_path: row.get(3)?,
                payload: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn load_input_file_events(app: tauri::AppHandle) -> Result<Vec<InputFileEventRecord>, String> {
    let conn = open_db(&app)?;
    load_input_file_event_records(&conn)
}

fn load_input_file_event_records(conn: &Connection) -> Result<Vec<InputFileEventRecord>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, path, modified_at, file_size, command, status, message, processed_at
            FROM input_file_event_log
            ORDER BY id DESC
            LIMIT 200
            "#,
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(InputFileEventRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                modified_at: row.get(2)?,
                file_size: row.get(3)?,
                command: row.get(4)?,
                status: row.get(5)?,
                message: row.get(6)?,
                processed_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn table_count(conn: &Connection, table: &str) -> Result<i64, String> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_folder(path: String) -> bool {
    std::path::Path::new(&path).is_dir()
}

#[tauri::command]
fn write_export_file(app: tauri::AppHandle, request: WriteExportRequest) -> Result<String, String> {
    let output_dir = PathBuf::from(request.output_directory.trim());
    if !output_dir.is_dir() {
        return Err("Output folder does not exist".to_string());
    }

    let file_name = std::path::Path::new(&request.file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid export filename".to_string())?;

    if file_name.is_empty() || file_name == "." || file_name == ".." {
        return Err("Invalid export filename".to_string());
    }

    let path = output_dir.join(file_name);
    fs::write(&path, &request.contents).map_err(|e| e.to_string())?;
    record_export(
        &app,
        &request.pos_system,
        &request.ticket_number,
        &path.to_string_lossy(),
        &request.contents,
    )?;
    Ok(path.to_string_lossy().to_string())
}

fn record_export(
    app: &tauri::AppHandle,
    pos_system: &str,
    ticket_number: &str,
    file_path: &str,
    payload: &str,
) -> Result<(), String> {
    let conn = open_db(app)?;
    conn.execute(
        r#"
        INSERT INTO exports (pos_system, ticket_number, file_path, payload, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            pos_system,
            ticket_number,
            file_path,
            payload,
            now_epoch_seconds()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn start_file_watch(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        if let Err(error) = scan_input_folder_for_receipts(&app) {
            eprintln!("Receipt input watcher error: {error}");
        }
        println!("Starting loopping\n");
        thread::sleep(Duration::from_secs(5));
    });
}

fn scan_input_folder_for_receipts(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = read_settings(app)?;
    let input_directory = settings.input_directory.trim();
    let input_file_name = settings.input_file_name.trim();
    let printer_path = settings.receipt_printer_path.trim();

    println!("Input Directory: {input_directory}/{input_file_name}\n");

    if input_directory.is_empty() || printer_path.is_empty() {
        return Ok(());
    }

    let input_path = Path::new(input_directory);
    if !input_path.is_dir() {
        return Ok(());
    }

    let conn = open_db(app)?;
    for entry in fs::read_dir(input_path).map_err(|e| e.to_string())? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Receipt input watcher skipped directory entry: {error}");
                continue;
            }
        };
        let path = entry.path();
        if !matches_configured_input_file(&path, input_file_name) {
            continue;
        }

        if let Err(error) = process_receipt_input_file(&conn, &settings, &path) {
            eprintln!(
                "Receipt input watcher failed for {}: {error}",
                path.to_string_lossy()
            );
        }
    }

    Ok(())
}

fn process_receipt_input_file(
    conn: &Connection,
    settings: &AppSettings,
    path: &Path,
) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Ok(());
    }

    let modified_at = file_modified_epoch(&metadata);
    let file_size = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
    let path_string = path.to_string_lossy().to_string();

    if input_file_event_is_current(conn, &path_string, &modified_at, file_size)? {
        record_input_file_event(
            conn,
            &path_string,
            &modified_at,
            file_size,
            "",
            "skipped",
            "Skipped duplicate read for unchanged file.",
        )?;
        delete_processed_input_file(path)?;
        return Ok(());
    }

    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let contents = String::from_utf8_lossy(&bytes);
    if !contains_print_receipt_command(&contents) {
        record_input_file_event(
            conn,
            &path_string,
            &modified_at,
            file_size,
            "",
            "ignored",
            "No RECEIPT_PRINT command found.",
        )?;
        delete_processed_input_file(path)?;
        return Ok(());
    }

    let result = print_receipt_from_command(conn, settings, &contents);
    let record_result = match result {
        Ok(ticket_number) => record_input_file_event(
            conn,
            &path_string,
            &modified_at,
            file_size,
            "RECEIPT_PRINT",
            "printed",
            &format!("Printed receipt for ticket {ticket_number}."),
        ),
        Err(error) => record_input_file_event(
            conn,
            &path_string,
            &modified_at,
            file_size,
            "RECEIPT_PRINT",
            "failed",
            &error,
        ),
    };

    record_result?;
    delete_processed_input_file(path)
}

fn delete_processed_input_file(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| {
        format!(
            "Processed input file but could not delete {}: {e}",
            path.to_string_lossy()
        )
    })
}

fn print_receipt_from_command(
    conn: &Connection,
    settings: &AppSettings,
    contents: &str,
) -> Result<String, String> {
    let command = receipt_print_command(conn, contents)?;
    let workspace = load_workspace_by_ticket_number(conn, &command.ticket_number)?
        .ok_or_else(|| format!("Ticket {} was not found in SQLite.", command.ticket_number))?;
    if workspace.customer.account_number.trim() != command.account_number.trim() {
        return Err(format!(
            "Ticket {} belongs to account {}, not account {}.",
            command.ticket_number, workspace.customer.account_number, command.account_number
        ));
    }

    let receipt = build_receipt_with_message(
        &workspace.customer,
        &workspace.ticket,
        &workspace.garments,
        &command.ticket_message,
        &settings.receipt_ticket_template,
    );
    send_escpos(settings.receipt_printer_path.trim(), &receipt)?;
    Ok(command.ticket_number)
}

fn receipt_print_command(conn: &Connection, contents: &str) -> Result<ReceiptPrintCommand, String> {
    if let Some(command) = extract_receipt_print_command(contents) {
        return resolve_receipt_print_command(conn, command);
    }

    if let Some(account_number) = extract_keyed_value(contents, &["ACCOUNT_NUMBER", "ACCOUNT"]) {
        let ticket_number = resolve_account_lookup(conn, &account_number)?;
        return Ok(ReceiptPrintCommand {
            account_number,
            ticket_number,
            ticket_message: extract_keyed_value(contents, &["TICKET_MESSAGE"]).unwrap_or_default(),
        });
    }

    let ticket_number = resolve_embedded_ticket_lookup(conn, contents)?;
    let workspace = load_workspace_by_ticket_number(conn, &ticket_number)?
        .ok_or_else(|| format!("Ticket {ticket_number} was not found in SQLite."))?;
    Ok(ReceiptPrintCommand {
        account_number: workspace.customer.account_number,
        ticket_number,
        ticket_message: extract_keyed_value(contents, &["TICKET_MESSAGE"]).unwrap_or_default(),
    })
}

fn resolve_ticket_lookup(conn: &Connection, value: &str) -> Result<String, String> {
    let lookup = value.trim();
    if lookup.is_empty() {
        return Err("RECEIPT_PRINT command did not include a ticket number.".to_string());
    }

    let ticket_number = conn
        .query_row(
            r#"
            SELECT ticket_number
            FROM tickets
            WHERE ticket_number = ?1
               OR full_invoice_number = ?1
               OR display_invoice_number = ?1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![lookup],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    ticket_number
        .ok_or_else(|| format!("No SQLite ticket matched RECEIPT_PRINT lookup value '{lookup}'."))
}

fn resolve_receipt_print_command(
    conn: &Connection,
    mut command: ReceiptPrintCommand,
) -> Result<ReceiptPrintCommand, String> {
    if command.ticket_number.trim().is_empty() {
        return Err("RECEIPT_PRINT command is missing TICKET_NUMBER.".to_string());
    }

    command.ticket_number = resolve_ticket_lookup(conn, &command.ticket_number)?;
    if command.account_number.trim().is_empty() {
        let workspace = load_workspace_by_ticket_number(conn, &command.ticket_number)?
            .ok_or_else(|| format!("Ticket {} was not found in SQLite.", command.ticket_number))?;
        command.account_number = workspace.customer.account_number;
    }
    Ok(command)
}

fn resolve_account_lookup(conn: &Connection, account_number: &str) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT ticket_number
            FROM tickets
            WHERE customer_account_number = ?1
            ORDER BY updated_at DESC
            LIMIT 2
            "#,
        )
        .map_err(|e| e.to_string())?;
    let tickets = stmt
        .query_map(params![account_number.trim()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    match tickets.as_slice() {
        [ticket_number] => Ok(ticket_number.clone()),
        [] => Err(format!(
            "No SQLite tickets matched account number '{}'.",
            account_number.trim()
        )),
        _ => Err(format!(
            "Account number '{}' has multiple tickets; include TICKET_NUMBER in the RECEIPT_PRINT file.",
            account_number.trim()
        )),
    }
}

fn resolve_embedded_ticket_lookup(conn: &Connection, contents: &str) -> Result<String, String> {
    let tickets = load_ticket_records(conn)?;
    let mut matches = HashSet::new();

    for ticket in tickets {
        for value in [
            ticket.ticket_number,
            ticket.full_invoice_number,
            ticket.display_invoice_number,
        ] {
            let value = value.trim();
            if !value.is_empty() && contents.contains(value) {
                matches.insert(value.to_string());
            }
        }
    }

    match matches.len() {
        1 => {
            let value = matches
                .into_iter()
                .next()
                .ok_or_else(|| "Unable to read matched ticket number.".to_string())?;
            resolve_ticket_lookup(conn, &value)
        }
        0 => Err(
            "RECEIPT_PRINT command needs a TICKET_NUMBER, invoice number, or matching ticket value."
                .to_string(),
        ),
        _ => Err("RECEIPT_PRINT file matched multiple tickets; include TICKET_NUMBER.".to_string()),
    }
}

fn extract_receipt_print_command(contents: &str) -> Option<ReceiptPrintCommand> {
    if contains_print_receipt_command(contents) {
        if let Some(ticket_number) = extract_keyed_value(contents, &["TICKET_NUMBER", "TICKET"]) {
            return Some(ReceiptPrintCommand {
                account_number: extract_keyed_value(contents, &["ACCOUNT_NUMBER", "ACCOUNT"])
                    .unwrap_or_default(),
                ticket_number,
                ticket_message: extract_keyed_value(contents, &["TICKET_MESSAGE"])
                    .unwrap_or_default(),
            });
        }
    }

    for row in delimited_rows(contents) {
        if row.is_empty() || !is_print_receipt_command(&row[0]) {
            continue;
        }

        return Some(receipt_print_command_from_positional_row(&row));
    }

    for pair in delimited_rows(contents).windows(2) {
        let header = &pair[0];
        let row = &pair[1];
        let Some(transaction_index) = header
            .iter()
            .position(|value| normalize_key(value) == "TRANSACTION")
        else {
            continue;
        };
        if !row
            .get(transaction_index)
            .is_some_and(|value| is_print_receipt_command(value))
        {
            continue;
        }

        return Some(receipt_print_command_from_header_row(header, row));
    }

    None
}

fn receipt_print_command_from_positional_row(row: &[String]) -> ReceiptPrintCommand {
    if normalize_key(&row[0]) != "RECEIPT_PRINT" && row.len() == 2 {
        return ReceiptPrintCommand {
            account_number: String::new(),
            ticket_number: row.get(1).cloned().unwrap_or_default(),
            ticket_message: String::new(),
        };
    }

    ReceiptPrintCommand {
        account_number: row.get(1).cloned().unwrap_or_default(),
        ticket_number: row.get(2).cloned().unwrap_or_default(),
        ticket_message: row.get(7).cloned().unwrap_or_default(),
    }
}

fn receipt_print_command_from_header_row(header: &[String], row: &[String]) -> ReceiptPrintCommand {
    ReceiptPrintCommand {
        account_number: header_value(header, row, "ACCOUNT_NUMBER").unwrap_or_default(),
        ticket_number: header_value(header, row, "TICKET_NUMBER").unwrap_or_default(),
        ticket_message: header_value(header, row, "TICKET_MESSAGE").unwrap_or_default(),
    }
}

fn header_value(header: &[String], row: &[String], key: &str) -> Option<String> {
    let index = header
        .iter()
        .position(|value| normalize_key(value) == key)?;
    let value = row.get(index)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn extract_keyed_value(contents: &str, keys: &[&str]) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim().trim_start_matches('\u{feff}');
        if trimmed.is_empty() {
            continue;
        }

        for delimiter in ["=", ":", "|", ",", "\t"] {
            let Some((key, value)) = trimmed.split_once(delimiter) else {
                continue;
            };
            if keys.iter().any(|expected| normalize_key(key) == *expected) {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

fn delimited_rows(contents: &str) -> Vec<Vec<String>> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_start_matches('\u{feff}');
            if trimmed.is_empty() {
                return None;
            }

            let delimiter = [',', '|', '\t']
                .into_iter()
                .max_by_key(|delimiter| trimmed.matches(*delimiter).count())?;
            if !trimmed.contains(delimiter) {
                return None;
            }

            Some(split_delimited_row(trimmed, delimiter))
        })
        .collect()
}

fn split_delimited_row(line: &str, delimiter: char) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(character) = chars.next() {
        if character == '"' {
            if in_quotes && chars.peek() == Some(&'"') {
                current.push('"');
                chars.next();
            } else {
                in_quotes = !in_quotes;
            }
            continue;
        }

        if character == delimiter && !in_quotes {
            values.push(clean_delimited_value(&current));
            current.clear();
            continue;
        }

        current.push(character);
    }

    values.push(clean_delimited_value(&current));
    values
}

fn clean_delimited_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn normalize_key(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .flat_map(|character| character.to_uppercase())
        .collect()
}

fn contains_print_receipt_command(contents: &str) -> bool {
    let upper = contents.to_ascii_uppercase();
    upper.contains("RECEIPT_PRINT")
        || upper.contains("PRINT_RECIEPT")
        || upper.contains("PRINT_RECEIPT")
}

fn is_print_receipt_command(value: &str) -> bool {
    let normalized = normalize_key(value);
    normalized == "RECEIPT_PRINT" || normalized == "PRINT_RECIEPT" || normalized == "PRINT_RECEIPT"
}

fn is_txt_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
}

fn matches_configured_input_file(path: &Path, input_file_name: &str) -> bool {
    if input_file_name.trim().is_empty() {
        return is_txt_file(path);
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(input_file_name.trim()))
}

fn file_modified_epoch(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn input_file_event_is_current(
    conn: &Connection,
    path: &str,
    modified_at: &str,
    file_size: i64,
) -> Result<bool, String> {
    let seen = conn
        .query_row(
            r#"
            SELECT 1
            FROM input_file_events
            WHERE path = ?1
              AND modified_at = ?2
              AND file_size = ?3
            "#,
            params![path, modified_at, file_size],
            |_| Ok(()),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(seen.is_some())
}

fn record_input_file_event(
    conn: &Connection,
    path: &str,
    modified_at: &str,
    file_size: i64,
    command: &str,
    status: &str,
    message: &str,
) -> Result<(), String> {
    let processed_at = now_epoch_seconds();
    conn.execute(
        r#"
        INSERT INTO input_file_event_log (
            path, modified_at, file_size, command, status, message, processed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            path,
            modified_at,
            file_size,
            command,
            status,
            message,
            processed_at
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        r#"
        INSERT INTO input_file_events (
            path, modified_at, file_size, command, status, message, processed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(path) DO UPDATE SET
            modified_at = excluded.modified_at,
            file_size = excluded.file_size,
            command = excluded.command,
            status = excluded.status,
            message = excluded.message,
            processed_at = excluded.processed_at
        "#,
        params![
            path,
            modified_at,
            file_size,
            command,
            status,
            message,
            processed_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_customer(app: tauri::AppHandle, request: DeleteCustomerRequest) -> Result<(), String> {
    let mut conn = open_db(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM garments WHERE ticket_number IN (
            SELECT ticket_number FROM tickets WHERE customer_account_number = ?1
        )",
        params![&request.account_number],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM exports WHERE ticket_number IN (
            SELECT ticket_number FROM tickets WHERE customer_account_number = ?1
        )",
        params![&request.account_number],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM tickets WHERE customer_account_number = ?1",
        params![&request.account_number],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM customers WHERE account_number = ?1",
        params![&request.account_number],
    )
    .map_err(|e| e.to_string())?;
    clear_current_ticket_if_missing(&tx)?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_ticket(app: tauri::AppHandle, request: DeleteTicketRequest) -> Result<(), String> {
    let mut conn = open_db(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM garments WHERE ticket_number = ?1",
        params![&request.ticket_number],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM exports WHERE ticket_number = ?1",
        params![&request.ticket_number],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM tickets WHERE ticket_number = ?1",
        params![&request.ticket_number],
    )
    .map_err(|e| e.to_string())?;
    clear_current_ticket_if_missing(&tx)?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_garment(app: tauri::AppHandle, request: DeleteGarmentRequest) -> Result<(), String> {
    let conn = open_db(&app)?;
    conn.execute(
        "DELETE FROM garments WHERE ticket_number = ?1 AND garment_id = ?2",
        params![request.ticket_number, request.garment_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_employee(app: tauri::AppHandle, request: DeleteEmployeeRequest) -> Result<(), String> {
    let conn = open_db(&app)?;
    conn.execute(
        "DELETE FROM employees WHERE employee_number = ?1",
        params![request.employee_number],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn clear_current_ticket_if_missing(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "DELETE FROM app_meta
         WHERE key = 'current_ticket_number'
           AND NOT EXISTS (SELECT 1 FROM tickets WHERE ticket_number = app_meta.value)",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn choose_folder(title: Option<String>) -> Result<Option<String>, String> {
    choose_folder_impl(title.unwrap_or_else(|| "Choose folder".to_string()))
}

#[cfg(target_os = "macos")]
fn choose_folder_impl(title: String) -> Result<Option<String>, String> {
    let script = format!(
        r#"POSIX path of (choose folder with prompt "{}")"#,
        title.replace('"', "\\\"")
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to open folder picker: {e}"))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!path.is_empty()).then_some(path));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") {
        return Ok(None);
    }

    Err(stderr.trim().to_string())
}

#[cfg(target_os = "windows")]
fn choose_folder_impl(title: String) -> Result<Option<String>, String> {
    let script = format!(
        r#"$shell = New-Object -ComObject Shell.Application
$folder = $shell.BrowseForFolder(0, '{}', 0)
if ($folder -ne $null) {{ $folder.Self.Path }}"#,
        title.replace('\'', "''")
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to open folder picker: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!path.is_empty()).then_some(path))
}

#[cfg(target_os = "linux")]
fn choose_folder_impl(title: String) -> Result<Option<String>, String> {
    let output = Command::new("zenity")
        .args(["--file-selection", "--directory", "--title", &title])
        .output()
        .map_err(|e| format!("Failed to open folder picker: {e}"))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!path.is_empty()).then_some(path));
    }

    Ok(None)
}

#[tauri::command]
fn discover_receipt_printers() -> Result<Vec<ReceiptPrinterInfo>, String> {
    discover_receipt_printers_impl()
}

#[tauri::command]
fn test_print_receipt(printer_path: String) -> Result<(), String> {
    if printer_path.trim().is_empty() {
        return Err("Choose a receipt printer before testing.".to_string());
    }

    let mut receipt = escpos_init();
    receipt.extend_from_slice(&[0x1B, 0x61, 0x01, 0x1B, 0x45, 0x01]);
    receipt.extend_from_slice(b"Demo POS\n");
    receipt.extend_from_slice(&[0x1B, 0x45, 0x00]);
    receipt.extend_from_slice(b"Receipt printer test\n");
    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(b"Printer connection is working.\n");
    receipt.extend_from_slice(b"\n\n\n");
    receipt.extend_from_slice(&[0x1D, 0x56, 0x42, 0x03]);
    send_escpos(printer_path.trim(), &receipt)
}

#[tauri::command]
fn print_receipt(request: PrintReceiptRequest) -> Result<(), String> {
    if request.printer_path.trim().is_empty() {
        return Err("Choose a receipt printer before printing.".to_string());
    }

    let receipt = build_receipt(
        &request.customer,
        &request.ticket,
        &request.garments,
        &request.receipt_ticket_template,
    );
    send_escpos(request.printer_path.trim(), &receipt)
}

fn build_receipt(
    customer: &Customer,
    ticket: &Ticket,
    garments: &[Garment],
    template: &TicketTemplateConfig,
) -> Vec<u8> {
    build_receipt_with_message(customer, ticket, garments, "", template)
}

fn build_receipt_with_message(
    customer: &Customer,
    ticket: &Ticket,
    garments: &[Garment],
    ticket_message: &str,
    template: &TicketTemplateConfig,
) -> Vec<u8> {
    let mut receipt = escpos_init();
    let template = normalize_receipt_ticket_template(template.clone());

    if !template.header_text.trim().is_empty() {
        receipt.extend_from_slice(&[0x1B, 0x61, 0x01, 0x1B, 0x45, 0x01, 0x1D, 0x21, 0x10]);
        receipt.extend_from_slice(template.header_text.trim().as_bytes());
        receipt.push(0x0A);
        receipt.extend_from_slice(&[0x1D, 0x21, 0x00, 0x1B, 0x45, 0x00]);
        receipt.extend_from_slice(b"------------------------------\n");
        receipt.extend_from_slice(&[0x1B, 0x61, 0x00]);
    }

    for field in template.fields.iter().filter(|field| field.enabled) {
        push_receipt_template_field(
            &mut receipt,
            field,
            customer,
            ticket,
            garments,
            ticket_message,
        );
    }

    if !template.footer_text.trim().is_empty() {
        receipt.extend_from_slice(b"------------------------------\n");
        receipt.extend_from_slice(&[0x1B, 0x61, 0x01]);
        receipt.extend_from_slice(template.footer_text.trim().as_bytes());
        receipt.push(0x0A);
        receipt.extend_from_slice(&[0x1B, 0x61, 0x00]);
    }

    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(b"\n\n\n");
    receipt.extend_from_slice(&[0x1D, 0x56, 0x42, 0x03]);
    receipt
}

fn push_receipt_template_field(
    receipt: &mut Vec<u8>,
    field: &TicketField,
    customer: &Customer,
    ticket: &Ticket,
    garments: &[Garment],
    ticket_message: &str,
) {
    match field.id.as_str() {
        "customerName" => push_receipt_line(
            receipt,
            "Customer",
            format!(
                "{} {}",
                customer.first_name.trim(),
                customer.last_name.trim()
            )
            .trim(),
        ),
        "customerIdentifier" => {
            let value = if ticket.customer_account_number.trim().is_empty() {
                customer.account_number.trim()
            } else {
                ticket.customer_account_number.trim()
            };
            push_receipt_line(receipt, "Account", value);
            if field.show_barcode {
                push_receipt_barcode(receipt, value);
            }
        }
        "customerPhone" => push_receipt_line(receipt, "Phone", customer.phone_number.trim()),
        "ticketNumber" => {
            push_receipt_line(receipt, "Ticket", ticket.ticket_number.trim());
            if field.show_barcode {
                push_receipt_barcode(receipt, ticket.ticket_number.trim());
            }
        }
        "invoiceNumber" => {
            let value = if ticket.display_invoice_number.trim().is_empty() {
                ticket.full_invoice_number.trim()
            } else {
                ticket.display_invoice_number.trim()
            };
            push_receipt_line(receipt, "Invoice", value);
            if field.show_barcode {
                push_receipt_barcode(receipt, value);
            }
        }
        "balanceDue" => push_receipt_line(receipt, "Balance", ticket.balance_due.trim()),
        "dropoffDate" => push_receipt_line(receipt, "Dropoff", ticket.dropoff_date_time.trim()),
        "pickupDate" => push_receipt_line(receipt, "Promised", ticket.promised_date_time.trim()),
        "readyDate" => push_receipt_line(
            receipt,
            "Ready",
            format!("{} {}", ticket.ready_date, ticket.ready_time).trim(),
        ),
        "numItems" => {
            let count = receipt_garments(garments).len();
            if count > 0 {
                let label = if count == 1 { "1 item" } else { "items" };
                let value = if count == 1 {
                    label.to_string()
                } else {
                    format!("{count} {label}")
                };
                push_receipt_line(receipt, "Items", &value);
            }
        }
        "itemList" => push_receipt_garments(receipt, garments, field.show_barcode),
        "comments" => push_receipt_block(receipt, ticket.comments.trim()),
        "ticketMessage" => push_receipt_block(receipt, ticket_message.trim()),
        _ => {}
    }
}

fn receipt_garments(garments: &[Garment]) -> Vec<&Garment> {
    garments
        .iter()
        .filter(|garment| !garment.id.trim().is_empty() || !garment.description.trim().is_empty())
        .collect()
}

fn push_receipt_garments(receipt: &mut Vec<u8>, garments: &[Garment], show_barcode: bool) {
    let export_garments = receipt_garments(garments);
    if export_garments.is_empty() {
        return;
    }

    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(b"Garments\n");
    for garment in export_garments {
        let line = format!("{}  {}\n", garment.id.trim(), garment.description.trim());
        receipt.extend_from_slice(line.as_bytes());
        if show_barcode && !garment.id.trim().is_empty() {
            push_receipt_barcode(receipt, garment.id.trim());
        }
    }
}

fn push_receipt_block(receipt: &mut Vec<u8>, value: &str) {
    if value.trim().is_empty() {
        return;
    }

    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(value.trim().as_bytes());
    receipt.push(0x0A);
}

fn push_receipt_barcode(receipt: &mut Vec<u8>, value: &str) {
    let content: String = value
        .trim()
        .chars()
        .filter(|character| character.is_ascii_graphic() || *character == ' ')
        .collect();
    if content.is_empty() {
        return;
    }

    let barcode_data = format!("{{B{content}");
    let Ok(length) = u8::try_from(barcode_data.len()) else {
        return;
    };

    receipt.extend_from_slice(&[0x1B, 0x61, 0x01]);
    receipt.extend_from_slice(&[0x1D, 0x48, 0x02]);
    receipt.extend_from_slice(&[0x1D, 0x68, 0x3C]);
    receipt.extend_from_slice(&[0x1D, 0x77, 0x02]);
    receipt.extend_from_slice(&[0x1D, 0x6B, 0x49, length]);
    receipt.extend_from_slice(barcode_data.as_bytes());
    receipt.push(0x0A);
    receipt.extend_from_slice(&[0x1B, 0x61, 0x00]);
}

fn escpos_init() -> Vec<u8> {
    vec![0x1B, 0x40, 0x0A]
}

fn push_receipt_line(receipt: &mut Vec<u8>, label: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }

    let line = format!("{label}: {}\n", value.trim());
    receipt.extend_from_slice(line.as_bytes());
}

fn parse_vid_pid(value: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = value.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Expected VID:PID format, got: {value}"));
    }

    let vid = u16::from_str_radix(parts[0].trim(), 16)
        .map_err(|_| format!("Invalid vendor ID: {}", parts[0]))?;
    let pid = u16::from_str_radix(parts[1].trim(), 16)
        .map_err(|_| format!("Invalid product ID: {}", parts[1]))?;
    Ok((vid, pid))
}

fn looks_like_ip(value: &str) -> bool {
    let host = value.splitn(2, ':').next().unwrap_or(value);
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4 && parts.iter().all(|part| part.parse::<u8>().is_ok())
}

fn is_receipt_printer_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("epson")
        || lower.contains("tm-")
        || lower.contains("tm_")
        || lower.contains("receipt")
        || lower.contains("thermal")
        || lower.contains("star")
        || lower.contains("bixolon")
        || lower.contains("citizen")
}

fn discover_known_network_receipt_printers() -> Vec<ReceiptPrinterInfo> {
    const NETWORK_PRINTERS: &[(&str, &str)] =
        &[("192.168.192.168:9100", "Epson Ethernet Receipt Printer")];

    NETWORK_PRINTERS
        .iter()
        .filter_map(|(addr, description)| {
            network_printer_is_reachable(addr).then(|| ReceiptPrinterInfo {
                path: addr.strip_suffix(":9100").unwrap_or(addr).to_string(),
                description: (*description).to_string(),
            })
        })
        .collect()
}

fn network_printer_is_reachable(addr: &str) -> bool {
    use std::net::{SocketAddr, TcpStream};

    let Ok(socket_addr) = addr.parse::<SocketAddr>() else {
        return false;
    };

    TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
}

#[cfg(not(target_os = "windows"))]
fn vendor_name(vid: u16) -> &'static str {
    match vid {
        0x04b8 => "Epson",
        0x0519 => "Star Micronics",
        0x1504 => "Bixolon",
        0x1d90 => "Citizen",
        0x0dd4 => "Custom",
        0x0fe6 => "ICS",
        _ => "",
    }
}

#[cfg(not(target_os = "windows"))]
fn discover_receipt_printers_impl() -> Result<Vec<ReceiptPrinterInfo>, String> {
    use std::collections::HashSet;

    let mut printers = Vec::new();
    let mut seen = HashSet::new();

    for printer in discover_known_network_receipt_printers() {
        if seen.insert(printer.path.clone()) {
            printers.push(printer);
        }
    }

    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("cu.usb")
                || name.starts_with("cu.usbserial")
                || name.starts_with("cu.usbmodem")
            {
                let path = format!("/dev/{name}");
                if seen.insert(path.clone()) {
                    printers.push(ReceiptPrinterInfo {
                        path,
                        description: "USB Serial Receipt Printer".to_string(),
                    });
                }
            }
        }
    }

    if let Ok(output) = Command::new("lpstat").arg("-p").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.starts_with("printer ") {
                continue;
            }
            let rest = &line["printer ".len()..];
            let name = rest.split_whitespace().next().unwrap_or("").to_string();
            if name.is_empty() || !is_receipt_printer_name(&name) {
                continue;
            }
            if seen.insert(name.clone()) {
                printers.push(ReceiptPrinterInfo {
                    path: name,
                    description: "CUPS Receipt Printer Queue".to_string(),
                });
            }
        }
    }

    let devices = rusb::devices().map_err(|error| format!("USB enumeration error: {error}"))?;
    for device in devices.iter() {
        let Ok(desc) = device.device_descriptor() else {
            continue;
        };
        let vid = desc.vendor_id();
        let pid = desc.product_id();
        let name = vendor_name(vid);
        if name.is_empty() {
            continue;
        }
        let path = format!("{vid:04x}:{pid:04x}");
        if seen.insert(path.clone()) {
            printers.push(ReceiptPrinterInfo {
                path,
                description: format!("{name} USB Receipt Printer"),
            });
        }
    }

    printers.sort_by(|a, b| a.description.cmp(&b.description).then(a.path.cmp(&b.path)));
    Ok(printers)
}

#[cfg(target_os = "windows")]
fn discover_receipt_printers_impl() -> Result<Vec<ReceiptPrinterInfo>, String> {
    use std::collections::HashSet;

    let mut printers = Vec::new();
    let mut seen = HashSet::new();

    for printer in discover_known_network_receipt_printers() {
        if seen.insert(printer.path.clone()) {
            printers.push(printer);
        }
    }

    if let Ok(queue_printers) = discover_windows_printer_queues() {
        for printer in queue_printers {
            if seen.insert(printer.path.to_lowercase()) {
                printers.push(printer);
            }
        }
    }

    if let Ok(usb_printers) = discover_windows_usbprint_devices() {
        for printer in usb_printers {
            if seen.insert(printer.path.to_lowercase()) {
                printers.push(printer);
            }
        }
    }

    let printer_script = r#"Get-CimInstance -ClassName Win32_Printer | ForEach-Object { "$($_.Name)|$($_.PortName)" }"#;
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", printer_script])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() < 2 {
                continue;
            }
            let name = parts[0].trim().to_string();
            let port = parts[1].trim().to_string();
            if name.is_empty() {
                continue;
            }
            let on_usb = port.to_uppercase().starts_with("USB");
            if !on_usb && !is_receipt_printer_name(&name) {
                continue;
            }
            if seen.insert(name.to_lowercase()) {
                printers.push(ReceiptPrinterInfo {
                    path: name.clone(),
                    description: format!("Windows Printer Queue - {name}"),
                });
            }
        }
    }

    let pnp_script = r#"Get-CimInstance -ClassName Win32_PnPEntity | Where-Object { $_.Name -match 'COM\d+' } | ForEach-Object { "$($_.Name)|$($_.DeviceID)" }"#;
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", pnp_script])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.is_empty() {
                continue;
            }
            let name = parts[0].trim().to_string();
            if name.is_empty() {
                continue;
            }

            let path = if let Some(start) = name.find("COM") {
                let com: String = name[start..]
                    .chars()
                    .take_while(|character| character.is_ascii_alphanumeric())
                    .collect();
                format!("\\\\.\\{com}")
            } else {
                continue;
            };

            if seen.insert(path.to_lowercase()) {
                printers.push(ReceiptPrinterInfo {
                    path,
                    description: format!("Windows Serial Device - {name}"),
                });
            }
        }
    }

    printers.sort_by(|a, b| a.description.cmp(&b.description).then(a.path.cmp(&b.path)));
    Ok(printers)
}

#[cfg(target_os = "windows")]
fn discover_windows_printer_queues() -> Result<Vec<ReceiptPrinterInfo>, String> {
    use std::{ptr, slice};
    use winapi::shared::minwindef::{DWORD, LPBYTE};
    use winapi::um::winspool::{
        EnumPrintersW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_INFO_2W,
    };

    let mut needed: DWORD = 0;
    let mut returned: DWORD = 0;
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;

    unsafe {
        EnumPrintersW(
            flags,
            ptr::null_mut(),
            2,
            ptr::null_mut(),
            0,
            &mut needed,
            &mut returned,
        );
    }

    if needed == 0 {
        return Ok(Vec::new());
    }

    let mut buffer = vec![0u8; needed as usize];
    let ok = unsafe {
        EnumPrintersW(
            flags,
            ptr::null_mut(),
            2,
            buffer.as_mut_ptr() as LPBYTE,
            needed,
            &mut needed,
            &mut returned,
        )
    };
    if ok == 0 {
        return Err(format!(
            "Windows printer enumeration failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let records = unsafe {
        slice::from_raw_parts(buffer.as_ptr() as *const PRINTER_INFO_2W, returned as usize)
    };
    let mut printers = Vec::new();
    for record in records {
        let name = wide_ptr_to_string(record.pPrinterName);
        if name.trim().is_empty() {
            continue;
        }

        let port = wide_ptr_to_string(record.pPortName);
        let usb_port = port.trim().to_uppercase().starts_with("USB");
        if !usb_port && !is_receipt_printer_name(&name) {
            continue;
        }

        let description = if port.trim().is_empty() {
            format!("Windows Printer Queue - {name}")
        } else {
            format!("Windows Printer Queue - {name} on {port}")
        };
        printers.push(ReceiptPrinterInfo {
            path: name,
            description,
        });
    }

    Ok(printers)
}

#[cfg(target_os = "windows")]
fn discover_windows_usbprint_devices() -> Result<Vec<ReceiptPrinterInfo>, String> {
    use std::{mem, ptr};
    use winapi::shared::guiddef::GUID;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::setupapi::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInterfaceDetailW, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
        SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_W,
    };

    const GUID_DEVINTERFACE_USBPRINT: GUID = GUID {
        Data1: 0x28d78fad,
        Data2: 0x5a12,
        Data3: 0x11d1,
        Data4: [0xae, 0x5b, 0x00, 0x00, 0xf8, 0x03, 0xa8, 0xc2],
    };

    let device_info = unsafe {
        SetupDiGetClassDevsW(
            &GUID_DEVINTERFACE_USBPRINT,
            ptr::null(),
            ptr::null_mut(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    };
    if device_info == INVALID_HANDLE_VALUE {
        return Err(format!(
            "Windows USB printer enumeration failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let mut printers = Vec::new();
    let mut index: DWORD = 0;
    loop {
        let mut interface_data: SP_DEVICE_INTERFACE_DATA = unsafe { mem::zeroed() };
        interface_data.cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;

        let found = unsafe {
            SetupDiEnumDeviceInterfaces(
                device_info,
                ptr::null_mut(),
                &GUID_DEVINTERFACE_USBPRINT,
                index,
                &mut interface_data,
            )
        };
        if found == 0 {
            break;
        }

        let mut required_size: DWORD = 0;
        unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                device_info,
                &mut interface_data,
                ptr::null_mut(),
                0,
                &mut required_size,
                ptr::null_mut(),
            );
        }

        if required_size > 0 {
            let mut buffer = vec![0u8; required_size as usize];
            let detail = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            unsafe {
                (*detail).cbSize = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as DWORD;
            }

            let ok = unsafe {
                SetupDiGetDeviceInterfaceDetailW(
                    device_info,
                    &mut interface_data,
                    detail,
                    required_size,
                    &mut required_size,
                    ptr::null_mut(),
                )
            };
            if ok != 0 {
                let path = unsafe { wide_ptr_to_string((*detail).DevicePath.as_ptr()) };
                if !path.trim().is_empty() {
                    printers.push(ReceiptPrinterInfo {
                        path,
                        description: "Windows USB Printer Device".to_string(),
                    });
                }
            }
        }

        index += 1;
    }

    unsafe {
        SetupDiDestroyDeviceInfoList(device_info);
    }

    Ok(printers)
}

#[cfg(target_os = "windows")]
fn to_wide(value: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_os = "windows")]
fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
    }
}

#[cfg(target_os = "windows")]
fn send_raw_windows_printer(printer_name: &str, data: &[u8]) -> Result<(), String> {
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winspool::{
        ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterW, StartDocPrinterW,
        StartPagePrinter, WritePrinter, DOC_INFO_1W,
    };

    let mut name_w = to_wide(printer_name);
    let mut datatype_w = to_wide("RAW");
    let mut docname_w = to_wide("Demo POS Receipt");
    let mut handle = ptr::null_mut();

    unsafe {
        if OpenPrinterW(name_w.as_mut_ptr(), &mut handle, ptr::null_mut()) == 0 {
            return Err(format!(
                "Cannot open printer '{}': {}",
                printer_name,
                std::io::Error::last_os_error()
            ));
        }

        let mut doc_info = DOC_INFO_1W {
            pDocName: docname_w.as_mut_ptr(),
            pOutputFile: ptr::null_mut(),
            pDatatype: datatype_w.as_mut_ptr(),
        };
        let job = StartDocPrinterW(handle, 1, &mut doc_info as *mut _ as *mut _);
        if job == 0 {
            ClosePrinter(handle);
            return Err(format!(
                "StartDocPrinter failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        if StartPagePrinter(handle) == 0 {
            EndDocPrinter(handle);
            ClosePrinter(handle);
            return Err(format!(
                "StartPagePrinter failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mut written: DWORD = 0;
        let ok = WritePrinter(
            handle,
            data.as_ptr() as *mut _,
            data.len() as DWORD,
            &mut written,
        );
        EndPagePrinter(handle);
        EndDocPrinter(handle);
        ClosePrinter(handle);

        if ok == 0 {
            return Err(format!(
                "WritePrinter failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        if written != data.len() as DWORD {
            return Err(format!(
                "WritePrinter wrote {written} of {} bytes",
                data.len()
            ));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn send_windows_device_path(path: &str, data: &[u8]) -> Result<(), String> {
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, WriteFile, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::winnt::{
        FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE,
    };

    let path_w = to_wide(path);
    let handle = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(format!(
            "Cannot open Windows device '{}': {}",
            path,
            std::io::Error::last_os_error()
        ));
    }

    let mut written: DWORD = 0;
    let ok = unsafe {
        WriteFile(
            handle,
            data.as_ptr() as *const _,
            data.len() as DWORD,
            &mut written,
            ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }

    if ok == 0 {
        return Err(format!(
            "Windows device write failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    if written != data.len() as DWORD {
        return Err(format!(
            "Windows device write sent {written} of {} bytes",
            data.len()
        ));
    }

    Ok(())
}

fn send_escpos(printer_path: &str, data: &[u8]) -> Result<(), String> {
    use std::io::Write;

    if let Ok((vid, pid)) = parse_vid_pid(printer_path) {
        #[cfg(target_os = "windows")]
        {
            let _ = (vid, pid);
            return Err(
                "VID:PID USB printing is not supported on Windows. Use Scan and select the Windows USB printer device, select the Windows printer queue, use tcp://host[:port], or use a COM device path."
                    .to_string(),
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            use escpos_rs::{Printer, PrinterProfile};
            let profile = PrinterProfile::usb_builder(vid, pid).build();
            let printer = Printer::new(profile)
                .map_err(|error| format!("Failed to connect to printer: {error}"))?
                .ok_or_else(|| {
                    format!(
                    "Printer {printer_path} not found. Make sure it is connected and powered on."
                )
                })?;
            return printer
                .raw(data)
                .map_err(|error| format!("Failed to send data to printer: {error}"));
        }
    }

    if let Some(addr) = tcp_printer_addr(printer_path) {
        use std::net::TcpStream;
        use std::time::Duration;

        let mut stream = TcpStream::connect(&addr)
            .map_err(|error| format!("Cannot connect to {addr}: {error}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| format!("Timeout error: {error}"))?;
        stream
            .write_all(data)
            .map_err(|error| format!("Network write error: {error}"))?;
        stream
            .flush()
            .map_err(|error| format!("Network flush error: {error}"))?;
        return Ok(());
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    if !printer_path.starts_with("/dev/") {
        let tmp = std::env::temp_dir().join("demo_pos_escpos.bin");
        fs::write(&tmp, data).map_err(|error| format!("Temp file error: {error}"))?;
        let tmp_path = tmp
            .to_str()
            .ok_or_else(|| "Invalid temporary print file path".to_string())?;
        let output = Command::new("lp")
            .args(["-d", printer_path, "-o", "raw", tmp_path])
            .output()
            .map_err(|error| format!("lp command failed: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "lp error: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        if is_windows_device_path(printer_path) {
            return send_windows_device_path(printer_path, data);
        }

        return send_raw_windows_printer(printer_path, data);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(printer_path)
            .map_err(|error| format!("Cannot open {printer_path}: {error}"))?;
        file.write_all(data)
            .map_err(|error| format!("Write error: {error}"))?;
        file.flush()
            .map_err(|error| format!("Flush error: {error}"))?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn is_windows_device_path(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with(r"\\?\")
        || trimmed.starts_with(r"\\.\")
        || trimmed.to_uppercase().starts_with("COM")
}

fn tcp_printer_addr(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(addr) = trimmed
        .strip_prefix("tcp://")
        .or_else(|| trimmed.strip_prefix("socket://"))
    {
        return normalize_tcp_printer_addr(addr);
    }

    if looks_like_ip(trimmed) {
        return normalize_tcp_printer_addr(trimmed);
    }

    None
}

fn normalize_tcp_printer_addr(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((_, port)) = trimmed.rsplit_once(':') {
        if !port.trim().is_empty() && port.chars().all(|character| character.is_ascii_digit()) {
            return Some(trimmed.to_string());
        }
    }

    Some(format!("{trimmed}:9100"))
}

fn find_printer_ip() -> Option<String> {
    let ip_address = "";

    return Some(format!("{ip_address}:9100"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_print_receipt_command_spellings() {
        assert!(contains_print_receipt_command("RECEIPT_PRINT"));
        assert!(contains_print_receipt_command("PRINT_RECIEPT"));
        assert!(contains_print_receipt_command("print_receipt"));
    }

    #[test]
    fn extracts_keyed_receipt_print_command() {
        let contents = "TRANSACTION=RECEIPT_PRINT\nACCOUNT_NUMBER=42\nTICKET_NUMBER=12345\nTICKET_MESSAGE=Split ticket\n";
        assert_eq!(
            extract_receipt_print_command(contents),
            Some(ReceiptPrintCommand {
                account_number: "42".to_string(),
                ticket_number: "12345".to_string(),
                ticket_message: "Split ticket".to_string(),
            })
        );
    }

    #[test]
    fn extracts_receipt_print_command_from_positional_row() {
        let contents =
            "RECEIPT_PRINT,42,12345,,7,CONV1,LOAD1,Split ticket,08/27/2026,03:10:22 PM\n";
        assert_eq!(
            extract_receipt_print_command(contents),
            Some(ReceiptPrintCommand {
                account_number: "42".to_string(),
                ticket_number: "12345".to_string(),
                ticket_message: "Split ticket".to_string(),
            })
        );
    }

    #[test]
    fn extracts_receipt_print_command_from_quoted_positional_row() {
        let contents =
            "\"RECEIPT_PRINT\",\"42\",\"12345\",\"\",\"7\",\"CONV1\",\"LOAD1\",\"Split, ticket\",\"08/27/2026\",\"03:10:22 PM\"\n";
        assert_eq!(
            extract_receipt_print_command(contents),
            Some(ReceiptPrintCommand {
                account_number: "42".to_string(),
                ticket_number: "12345".to_string(),
                ticket_message: "Split, ticket".to_string(),
            })
        );
    }

    #[test]
    fn extracts_receipt_print_command_from_header_row() {
        let contents = "TRANSACTION,ACCOUNT_NUMBER,TICKET_NUMBER,NOT_USED_1,EMPLOYEE_NUMBER,CONVEYOR_ID,LOADSTATION_ID,TICKET_MESSAGE,TRANSACTION_DATE,TRANSACTION_TIME\nRECEIPT_PRINT,42,12345,,7,CONV1,LOAD1,Split ticket,08/27/2026,03:10:22 PM\n";
        assert_eq!(
            extract_receipt_print_command(contents),
            Some(ReceiptPrintCommand {
                account_number: "42".to_string(),
                ticket_number: "12345".to_string(),
                ticket_message: "Split ticket".to_string(),
            })
        );
    }

    #[test]
    fn receipt_template_controls_printed_fields() {
        let customer = Customer {
            account_number: "42".to_string(),
            phone_number: "555-0100".to_string(),
            first_name: "Ada".to_string(),
            last_name: "Lovelace".to_string(),
            pin: String::new(),
            address1: String::new(),
            address2: String::new(),
            city: String::new(),
            state: String::new(),
            zip_code: String::new(),
        };
        let ticket = Ticket {
            customer_account_number: "42".to_string(),
            ticket_number: "T-100".to_string(),
            full_invoice_number: "INV-FULL".to_string(),
            display_invoice_number: "INV-100".to_string(),
            balance_due: "$19.50".to_string(),
            dropoff_date_time: "2026-09-01T09:00:00".to_string(),
            promised_date_time: "2026-09-02T17:00:00".to_string(),
            comments: "No starch".to_string(),
            ready_date: "09/02/2026".to_string(),
            ready_time: "05:00:00 PM".to_string(),
            plant: String::new(),
            route: String::new(),
            route_stop: String::new(),
            store: String::new(),
        };
        let mut template = default_receipt_ticket_template();
        for field in &mut template.fields {
            field.enabled = field.id == "balanceDue";
        }

        let receipt = build_receipt(&customer, &ticket, &[], &template);
        let text = String::from_utf8_lossy(&receipt);

        assert!(text.contains("Balance: $19.50"));
        assert!(!text.contains("Ticket: T-100"));
        assert!(!text.contains("Customer: Ada Lovelace"));
    }

    #[test]
    fn blank_input_file_name_matches_txt_files_only() {
        assert!(matches_configured_input_file(Path::new("inbox.txt"), ""));
        assert!(!matches_configured_input_file(Path::new("inbox.csv"), ""));
    }

    #[test]
    fn configured_input_file_name_matches_exact_file_name_case_insensitive() {
        assert!(matches_configured_input_file(
            Path::new("/tmp/POS.TXT"),
            "pos.txt"
        ));
        assert!(!matches_configured_input_file(
            Path::new("/tmp/other.txt"),
            "pos.txt"
        ));
    }

    #[test]
    fn tcp_printer_addr_defaults_ip_to_raw_print_port() {
        assert_eq!(
            tcp_printer_addr("192.168.1.50"),
            Some("192.168.1.50:9100".to_string())
        );
        assert_eq!(
            tcp_printer_addr("192.168.1.50:9101"),
            Some("192.168.1.50:9101".to_string())
        );
    }

    #[test]
    fn tcp_printer_addr_accepts_explicit_socket_prefixes() {
        assert_eq!(
            tcp_printer_addr("tcp://receipt-printer"),
            Some("receipt-printer:9100".to_string())
        );
        assert_eq!(
            tcp_printer_addr("socket://receipt-printer.local:9102"),
            Some("receipt-printer.local:9102".to_string())
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            start_file_watch(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            load_workspace,
            load_ticket_workspace,
            load_customer,
            save_customer,
            save_employee,
            save_workspace,
            load_database_summary,
            load_input_file_events,
            delete_customer,
            delete_ticket,
            delete_garment,
            delete_employee,
            check_folder,
            choose_folder,
            write_export_file,
            discover_receipt_printers,
            test_print_receipt,
            print_receipt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
