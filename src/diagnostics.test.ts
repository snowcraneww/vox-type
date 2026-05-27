import { describe, expect, it } from 'vitest';
import { formatDiagnosticsForCopy } from './diagnostics';

describe('formatDiagnosticsForCopy', () => {
  it('keeps all entries in copy output', () => {
    const text = formatDiagnosticsForCopy([
      { time: '1:00:00 pm', title: '第一条', result: 'info', detail: '早期日志' },
      { time: '1:01:00 pm', title: '第二条', result: 'success', detail: '后续日志' },
    ]);

    expect(text).toContain('第一条');
    expect(text).toContain('早期日志');
    expect(text).toContain('第二条');
    expect(text).toContain('后续日志');
  });
});
