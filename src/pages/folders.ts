import type { AppSettings, AppState } from "../types";
import { escapeAttribute, pageHeader } from "../ui/html";

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
      <div>
        <label>${label}</label>
        <p>${description}</p>
      </div>
      <div class="folder-row">
        <input data-bind="settings.${key}" value="${escapeAttribute(value)}" placeholder="/path/to/folder" />
        <button class="icon-button" title="Choose folder" data-action="choose-folder" data-folder-key="${key}">...</button>
      </div>
    </div>
  `;
}
