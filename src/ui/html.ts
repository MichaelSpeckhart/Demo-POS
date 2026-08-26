export function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function escapeAttribute(value: string) {
  return escapeHtml(value);
}

export function field(path: string, label: string, value: string, hint = "") {
  return `
    <label class="field">
      <span>${label}</span>
      ${hint ? `<small>${hint}</small>` : ""}
      <input data-bind="${path}" value="${escapeAttribute(value)}" />
    </label>
  `;
}

export function pageHeader(step: string, title: string, description: string) {
  return `
    <div class="page-heading">
      <p class="eyebrow">${step}</p>
      <h1>${title}</h1>
      <p>${description}</p>
    </div>
  `;
}

export function statusMarkup(message: string, kind: "neutral" | "success" | "error") {
  if (!message) return "";
  return `<div class="status ${kind}">${escapeHtml(message)}</div>`;
}
