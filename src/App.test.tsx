import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('renders the user-facing voice input view by default', () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: 'VoxType' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '开始录音' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '诊断模式' })).toBeInTheDocument();
    expect(screen.getByRole('status', { name: '语音输入状态：已就绪' })).toBeInTheDocument();
    expect(screen.getByText('中文优先，兼容英文')).toBeInTheDocument();
    expect(screen.getByText('whisper.cpp')).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: '诊断工作台' })).not.toBeInTheDocument();
    expect(document.querySelector('.traffic-lights')).not.toBeInTheDocument();
  });

  it('opens the diagnostic workbench on request', () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: '诊断模式' }));

    expect(screen.getByRole('heading', { name: '诊断工作台' })).toBeInTheDocument();
    expect(screen.getByText('开始录音采集')).toBeInTheDocument();
    expect(screen.getByText('停止录音采集')).toBeInTheDocument();
    expect(screen.getByText('转写最近录音')).toBeInTheDocument();
    expect(screen.getByText('测试剪贴板上屏')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: '诊断日志' })).toBeInTheDocument();
  });
});
