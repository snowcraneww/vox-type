interface StructuredError {
  code?: unknown;
  message?: unknown;
}

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (error && typeof error === 'object') {
    const structured = error as StructuredError;
    const message = typeof structured.message === 'string' ? structured.message : null;
    const code = typeof structured.code === 'string' ? structured.code : null;
    if (message && code) {
      return `${message}（${code}）`;
    }
    if (message) {
      return message;
    }

    try {
      return JSON.stringify(error);
    } catch {
      return '未知错误对象';
    }
  }

  return String(error);
}
