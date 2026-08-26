export function resolveOutputFileName(customFileName: string, defaultFileName: string) {
  const trimmed = customFileName.trim();
  if (!trimmed) return defaultFileName;
  if (trimmed === "." || trimmed === ".." || /[\\/]/.test(trimmed)) return "";
  if (trimmed.includes(".")) return trimmed;
  return `${trimmed}${defaultExtension(defaultFileName)}`;
}

function defaultExtension(fileName: string) {
  const match = fileName.match(/(\.[a-z0-9]+)$/i);
  return match?.[1] ?? ".csv";
}
