import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('renders confirmed MVP settings', () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: '本地优先语音输入工具' })).toBeInTheDocument();
    expect(screen.getByText('中文优先，兼容英文')).toBeInTheDocument();
    expect(screen.getByText('whisper.cpp')).toBeInTheDocument();
    expect(screen.getByText('剪贴板粘贴并恢复')).toBeInTheDocument();
    expect(screen.getByText('开始录音采集')).toBeInTheDocument();
    expect(screen.getByText('停止录音采集')).toBeInTheDocument();
    expect(screen.getByText('测试剪贴板上屏')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '诊断日志' })).toBeInTheDocument();
  });
});
