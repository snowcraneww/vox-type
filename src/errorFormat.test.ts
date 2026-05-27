import { describe, expect, it } from 'vitest';
import { formatError } from './errorFormat';

describe('formatError', () => {
  it('formats structured Tauri errors with message and code', () => {
    const result = formatError({
      code: 'config_failed',
      message: '配置错误：缺少 whisper.cpp 可执行文件路径',
    });

    expect(result).toBe('配置错误：缺少 whisper.cpp 可执行文件路径（config_failed）');
  });

  it('does not render plain objects as object Object', () => {
    const result = formatError({ detail: 'missing model' });

    expect(result).not.toContain('[object Object]');
    expect(result).toContain('missing model');
  });
});
