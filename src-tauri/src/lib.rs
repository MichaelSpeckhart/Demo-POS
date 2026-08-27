use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    pos_system: String,
    input_directory: String,
    output_directory: String,
    #[serde(default)]
    output_file_name: String,
    #[serde(default = "default_export_operation")]
    export_operation: String,
    #[serde(default)]
    receipt_printer_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pos_system: "spot".to_string(),
            input_directory: String::new(),
            output_directory: String::new(),
            output_file_name: String::new(),
            export_operation: default_export_operation(),
            receipt_printer_path: String::new(),
        }
    }
}

fn default_export_operation() -> String {
    "create".to_string()
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

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
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

    let receipt = build_receipt(&request.customer, &request.ticket, &request.garments);
    send_escpos(request.printer_path.trim(), &receipt)
}

fn build_receipt(customer: &Customer, ticket: &Ticket, garments: &[Garment]) -> Vec<u8> {
    let mut receipt = escpos_init();

    receipt.extend_from_slice(&[0x1B, 0x61, 0x01, 0x1B, 0x45, 0x01, 0x1D, 0x21, 0x10]);
    receipt.extend_from_slice(b"Demo POS\n");
    receipt.extend_from_slice(&[0x1D, 0x21, 0x00, 0x1B, 0x45, 0x00]);
    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(&[0x1B, 0x61, 0x00]);

    push_receipt_line(
        &mut receipt,
        "Customer",
        format!(
            "{} {}",
            customer.first_name.trim(),
            customer.last_name.trim()
        )
        .trim(),
    );
    push_receipt_line(&mut receipt, "Account", customer.account_number.trim());
    push_receipt_line(&mut receipt, "Phone", customer.phone_number.trim());
    push_receipt_line(&mut receipt, "Ticket", ticket.ticket_number.trim());
    push_receipt_line(
        &mut receipt,
        "Invoice",
        ticket.display_invoice_number.trim(),
    );
    push_receipt_line(&mut receipt, "Promised", ticket.promised_date_time.trim());
    push_receipt_line(
        &mut receipt,
        "Ready",
        format!("{} {}", ticket.ready_date, ticket.ready_time).trim(),
    );

    let export_garments: Vec<&Garment> = garments
        .iter()
        .filter(|garment| !garment.id.trim().is_empty() || !garment.description.trim().is_empty())
        .collect();
    if !export_garments.is_empty() {
        receipt.extend_from_slice(b"------------------------------\n");
        receipt.extend_from_slice(b"Garments\n");
        for garment in export_garments {
            let line = format!("{}  {}\n", garment.id.trim(), garment.description.trim());
            receipt.extend_from_slice(line.as_bytes());
        }
    }

    if !ticket.comments.trim().is_empty() {
        receipt.extend_from_slice(b"------------------------------\n");
        receipt.extend_from_slice(ticket.comments.trim().as_bytes());
        receipt.push(0x0A);
    }

    receipt.extend_from_slice(b"------------------------------\n");
    receipt.extend_from_slice(b"\n\n\n");
    receipt.extend_from_slice(&[0x1D, 0x56, 0x42, 0x03]);
    receipt
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

    let pnp_script = r#"Get-CimInstance -ClassName Win32_PnPEntity | Where-Object { $_.Name -match 'COM\d+|USB Printing' -or $_.DeviceID -match 'VID_04B8|VID_0519|VID_1504|VID_1D90|VID_0DD4|VID_0FE6' } | ForEach-Object { "$($_.Name)|$($_.DeviceID)" }"#;
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
            let device_id = parts.get(1).copied().unwrap_or("").trim();
            if name.is_empty() {
                continue;
            }

            let path = if let Some((vid, pid)) = parse_vid_pid_from_device_id(device_id) {
                format!("{vid:04x}:{pid:04x}")
            } else if let Some(start) = name.find("COM") {
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
                    description: name,
                });
            }
        }
    }

    printers.sort_by(|a, b| a.description.cmp(&b.description).then(a.path.cmp(&b.path)));
    Ok(printers)
}

#[cfg(target_os = "windows")]
fn parse_vid_pid_from_device_id(device_id: &str) -> Option<(u16, u16)> {
    let upper = device_id.to_uppercase();
    let vid_pos = upper.find("VID_")? + 4;
    let pid_pos = upper.find("PID_")? + 4;
    if vid_pos + 4 > device_id.len() || pid_pos + 4 > device_id.len() {
        return None;
    }
    let vid = u16::from_str_radix(&device_id[vid_pos..vid_pos + 4], 16).ok()?;
    let pid = u16::from_str_radix(&device_id[pid_pos..pid_pos + 4], 16).ok()?;
    Some((vid, pid))
}

#[cfg(target_os = "windows")]
fn send_raw_windows_printer(printer_name: &str, data: &[u8]) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winspool::{
        ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterW, StartDocPrinterW,
        StartPagePrinter, WritePrinter, DOC_INFO_1W,
    };

    fn to_wide(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

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
    }

    Ok(())
}

fn send_escpos(printer_path: &str, data: &[u8]) -> Result<(), String> {
    use std::io::Write;

    if let Ok((vid, pid)) = parse_vid_pid(printer_path) {
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

    if looks_like_ip(printer_path) {
        use std::net::TcpStream;
        use std::time::Duration;

        let addr = if printer_path.contains(':') {
            printer_path.to_string()
        } else {
            format!("{printer_path}:9100")
        };
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
        let is_device_path =
            printer_path.starts_with("\\\\.\\") || printer_path.to_uppercase().starts_with("COM");
        if !is_device_path {
            return send_raw_windows_printer(printer_path, data);
        }
    }

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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

pub async fn file_watch() {}
