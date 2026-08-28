import type { AppState, InputFileEventRecord } from "../types";
import { escapeHtml, pageHeader, statusMarkup } from "../ui/html";

export function renderConveyorLogPage(state: AppState) {
  const configuredName = state.settings.inputFileName.trim() || "*.txt";
  return `
    <section class="page database-page">
      <div class="page-heading with-actions">
        ${pageHeader(
          "Input Log",
          "Conveyor File Reads",
          `Latest reads from ${configuredName} in the configured input folder.`
        )}
        <button type="button" class="secondary-button" data-action="refresh-input-log">Refresh</button>
      </div>

      ${statusMarkup(state.status, state.statusKind)}
      ${logTable(state.inputFileEvents)}
    </section>
  `;
}

function logTable(rows: InputFileEventRecord[]) {
  return `
    <section class="db-section">
      <div class="db-section-heading">
        <h2>Read History</h2>
        <span>${rows.length} rows</span>
      </div>
      ${rows.length === 0 ? emptyLog() : `
        <div class="db-table-wrap">
          <table class="db-table input-log-table">
            <thead>
              <tr>
                <th>Read At</th>
                <th>Operation</th>
                <th>Status</th>
                <th>Message</th>
                <th>File</th>
                <th>Modified</th>
                <th>Bytes</th>
              </tr>
            </thead>
            <tbody>
              ${rows.map((row) => `
                <tr>
                  <td>${escapeHtml(formatEpoch(row.processedAt))}</td>
                  <td>${escapeHtml(row.command.trim() || "No command")}</td>
                  <td>${statusPill(row.status)}</td>
                  <td>${escapeHtml(row.message || "-")}</td>
                  <td><code>${escapeHtml(row.path)}</code></td>
                  <td>${escapeHtml(formatEpoch(row.modifiedAt))}</td>
                  <td>${row.fileSize}</td>
                </tr>
              `).join("")}
            </tbody>
          </table>
        </div>
      `}
    </section>
  `;
}

function statusPill(status: string) {
  const normalized = status.trim().toLowerCase();
  const className = normalized === "printed"
    ? "success"
    : normalized === "failed"
      ? "error"
      : "neutral";
  return `<span class="log-status ${className}">${escapeHtml(status || "unknown")}</span>`;
}

function formatEpoch(value: string) {
  const seconds = Number(value);
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return value || "-";
  }

  return new Date(seconds * 1000).toLocaleString();
}

function emptyLog() {
  return `<div class="empty-table">No conveyor file reads have been logged yet.</div>`;
}
