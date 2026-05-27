export interface DiagnosticEntryForExport {
  time: string;
  title: string;
  result: string;
  detail: string;
}

export function formatDiagnosticsForCopy(entries: DiagnosticEntryForExport[]): string {
  return entries
    .map((entry) => `[${entry.time}] ${entry.title} / ${entry.result}\n${entry.detail}`)
    .join('\n\n');
}
