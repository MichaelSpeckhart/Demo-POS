import type { AppSettings, AppState } from "../types";
import { escapeHtml, pageHeader } from "../ui/html";

export function renderFoldersPage(state: AppState) {
  return `
    <section class="page folders-page">
      ${pageHeader(
        "Step 2",
        "Choose Folders",
        "The output folder receives generated POS exports. The input folder is reserved for OAS responses."
      )}

      <div class="folder-stack">
        ${folderField("outputDirectory", "Output Folder", "Where Demo POS writes WinCleaners CSV exports.", state.settings.outputDirectory)}
        ${folderField("inputDirectory", "Input Folder", "Where Demo POS can read OAS responses or confirmations.", state.settings.inputDirectory)}
      </div>
    </section>
  `;
}

function folderField(key: keyof AppSettings, label: string, description: string, value: string) {
  return `
    <div class="folder-field">
      <div class="folder-field-main">
        <div class="folder-copy">
          <label>${escapeHtml(label)}</label>
          <p>${escapeHtml(description)}</p>
          ${
            value
              ? `<div class="folder-path"><span class="folder-status-dot" aria-hidden="true"></span><code>${escapeHtml(value)}</code></div>`
              : `<p class="folder-empty">No folder selected</p>`
          }
        </div>
        <button class="primary-button browse-folder-button" title="Choose folder" data-action="choose-folder" data-folder-key="${key}">
          <span class="folder-button-icon" aria-hidden="true"></span>
          Browse
        </button>
      </div>
    </div>
  `;
}
