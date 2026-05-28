use crate::error::VoxError;

#[cfg(windows)]
mod platform {
    use super::VoxError;
    use std::ptr::null;
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Mutex, OnceLock};
    use std::thread::{self, JoinHandle};
    use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        BeginPaint, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DeleteObject, EndPaint,
        InvalidateRect, Polyline, RoundRect, SelectObject, SetBkMode, SetWindowRgn, HGDIOBJ,
        PAINTSTRUCT, PS_SOLID, TRANSPARENT,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetSystemMetrics,
        KillTimer, PeekMessageW, RegisterClassW, SetLayeredWindowAttributes, SetTimer, ShowWindow,
        TranslateMessage, CS_HREDRAW, CS_VREDRAW, LWA_ALPHA, MSG, PM_REMOVE, SM_CXSCREEN,
        SM_CYSCREEN, SW_HIDE, SW_SHOWNOACTIVATE, WM_DESTROY, WM_PAINT, WM_TIMER, WNDCLASSW,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    const WIDTH: i32 = 120;
    const HEIGHT: i32 = 32;
    const MARGIN_BOTTOM: i32 = 92;
    const TIMER_ID: usize = 1;
    const TIMER_MS: u32 = 90;
    const CLASS_NAME: &[u16] = &[
        b'V' as u16,
        b'o' as u16,
        b'x' as u16,
        b'T' as u16,
        b'y' as u16,
        b'p' as u16,
        b'e' as u16,
        b'N' as u16,
        b'a' as u16,
        b't' as u16,
        b'i' as u16,
        b'v' as u16,
        b'e' as u16,
        b'O' as u16,
        b'v' as u16,
        b'e' as u16,
        b'r' as u16,
        b'l' as u16,
        b'a' as u16,
        b'y' as u16,
        0,
    ];
    const WINDOW_TITLE: &[u16] = &[
        b'V' as u16,
        b'o' as u16,
        b'x' as u16,
        b'T' as u16,
        b'y' as u16,
        b'p' as u16,
        b'e' as u16,
        b' ' as u16,
        b'O' as u16,
        b'v' as u16,
        b'e' as u16,
        b'r' as u16,
        b'l' as u16,
        b'a' as u16,
        b'y' as u16,
        0,
    ];

    static OVERLAY_THREAD: OnceLock<Mutex<Option<OverlayThread>>> = OnceLock::new();
    static FRAME_INDEX: OnceLock<Mutex<u32>> = OnceLock::new();

    enum OverlayCommand {
        Hide,
        Stop,
    }

    struct OverlayThread {
        sender: Sender<OverlayCommand>,
        handle: JoinHandle<()>,
    }

    pub fn show() -> Result<(), VoxError> {
        let mut guard = overlay_thread_cell()
            .lock()
            .map_err(|error| VoxError::Config(format!("锁定原生浮窗线程失败：{error}")))?;

        if guard.is_some() {
            return Ok(());
        }

        let (ready_sender, ready_receiver) = mpsc::channel();
        let (command_sender, command_receiver) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("voxtype-native-overlay".to_string())
            .spawn(move || run_overlay_window(ready_sender, command_receiver))
            .map_err(|error| VoxError::Config(format!("启动原生浮窗线程失败：{error}")))?;

        match ready_receiver.recv() {
            Ok(Ok(())) => {
                *guard = Some(OverlayThread {
                    sender: command_sender,
                    handle,
                });
                Ok(())
            }
            Ok(Err(error)) => {
                let _ = command_sender.send(OverlayCommand::Stop);
                let _ = handle.join();
                Err(VoxError::Config(error))
            }
            Err(error) => {
                let _ = command_sender.send(OverlayCommand::Stop);
                let _ = handle.join();
                Err(VoxError::Config(format!("等待原生浮窗启动失败：{error}")))
            }
        }
    }

    pub fn hide() -> Result<(), VoxError> {
        let mut guard = overlay_thread_cell()
            .lock()
            .map_err(|error| VoxError::Config(format!("锁定原生浮窗线程失败：{error}")))?;

        if let Some(thread) = guard.take() {
            let _ = thread.sender.send(OverlayCommand::Hide);
            let _ = thread.handle.join();
        }

        Ok(())
    }

    fn overlay_thread_cell() -> &'static Mutex<Option<OverlayThread>> {
        OVERLAY_THREAD.get_or_init(|| Mutex::new(None))
    }

    fn frame_cell() -> &'static Mutex<u32> {
        FRAME_INDEX.get_or_init(|| Mutex::new(0))
    }

    fn run_overlay_window(
        ready_sender: Sender<Result<(), String>>,
        command_receiver: mpsc::Receiver<OverlayCommand>,
    ) {
        let hwnd = match create_overlay_window() {
            Ok(hwnd) => hwnd,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };

        let _ = ready_sender.send(Ok(()));

        loop {
            if let Ok(command) = command_receiver.try_recv() {
                match command {
                    OverlayCommand::Hide | OverlayCommand::Stop => {
                        unsafe {
                            ShowWindow(hwnd, SW_HIDE);
                            DestroyWindow(hwnd);
                        }
                        return;
                    }
                }
            }

            pump_messages();
            thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    fn create_overlay_window() -> Result<HWND, String> {
        unsafe {
            let instance = GetModuleHandleW(null());
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: CLASS_NAME.as_ptr(),
                ..Default::default()
            };
            RegisterClassW(&wnd_class);

            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_width - WIDTH) / 2;
            let y = screen_height - HEIGHT - MARGIN_BOTTOM;
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                CLASS_NAME.as_ptr(),
                WINDOW_TITLE.as_ptr(),
                WS_POPUP,
                x,
                y,
                WIDTH,
                HEIGHT,
                null_mut_hwnd(),
                null_mut_hmenu(),
                instance,
                null(),
            );

            if hwnd.is_null() {
                return Err(format!(
                    "创建原生浮窗失败，Win32 错误码：{}",
                    GetLastError()
                ));
            }

            let region = CreateRoundRectRgn(0, 0, WIDTH + 1, HEIGHT + 1, HEIGHT, HEIGHT);
            if region.is_null() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "创建原生浮窗圆角区域失败，Win32 错误码：{}",
                    GetLastError()
                ));
            }
            if SetWindowRgn(hwnd, region, 1) == 0 {
                DeleteObject(region as HGDIOBJ);
                DestroyWindow(hwnd);
                return Err(format!(
                    "应用原生浮窗圆角区域失败，Win32 错误码：{}",
                    GetLastError()
                ));
            }

            if SetLayeredWindowAttributes(hwnd, 0, 244, LWA_ALPHA) == 0 {
                DestroyWindow(hwnd);
                return Err(format!(
                    "设置原生浮窗透明度失败，Win32 错误码：{}",
                    GetLastError()
                ));
            }

            if SetTimer(hwnd, TIMER_ID, TIMER_MS, None) == 0 {
                DestroyWindow(hwnd);
                return Err(format!(
                    "启动原生浮窗动画计时器失败，Win32 错误码：{}",
                    GetLastError()
                ));
            }

            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            InvalidateRect(hwnd, null(), 0);
            Ok(hwnd)
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_PAINT => {
                paint_overlay(hwnd);
                0
            }
            WM_TIMER => {
                if let Ok(mut frame) = frame_cell().lock() {
                    *frame = frame.wrapping_add(1);
                }
                InvalidateRect(hwnd, null(), 0);
                0
            }
            WM_DESTROY => {
                KillTimer(hwnd, TIMER_ID);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    unsafe fn paint_overlay(hwnd: HWND) {
        let mut paint = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut paint);
        if hdc.is_null() {
            return;
        }

        let background = CreateSolidBrush(colorref(12, 12, 14));
        let border_pen = CreatePen(PS_SOLID, 1, colorref(38, 38, 42));
        let old_brush = SelectObject(hdc, background as HGDIOBJ);
        let old_pen = SelectObject(hdc, border_pen as HGDIOBJ);
        RoundRect(hdc, 0, 0, WIDTH, HEIGHT, HEIGHT, HEIGHT);
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        DeleteObject(border_pen as HGDIOBJ);
        DeleteObject(background as HGDIOBJ);

        SetBkMode(hdc, TRANSPARENT as i32);
        let frame = frame_cell().lock().map(|frame| *frame).unwrap_or(0);
        draw_wave(hdc, frame);

        EndPaint(hwnd, &paint);
    }

    unsafe fn draw_wave(hdc: windows_sys::Win32::Graphics::Gdi::HDC, frame: u32) {
        let colors = [
            colorref(78, 205, 255),
            colorref(142, 109, 255),
            colorref(255, 82, 178),
            colorref(255, 193, 74),
        ];
        for lane in 0..3 {
            let color = colors[((frame as usize / 2) + lane) % colors.len()];
            let pen = CreatePen(PS_SOLID, 2, color);
            let old_pen = SelectObject(hdc, pen as HGDIOBJ);
            let mut points = Vec::new();
            for index in 0_i32..22 {
                let x = 18 + index * 4;
                let phase = ((index + frame as i32 + lane as i32 * 3) % 8) as usize;
                let wave = [0, 3, 5, 3, 0, -3, -5, -3][phase];
                let y = 16 + (lane as i32 - 1) * 4 + wave;
                points.push(POINT { x, y });
            }
            Polyline(hdc, points.as_ptr(), points.len() as i32);
            SelectObject(hdc, old_pen);
            DeleteObject(pen as HGDIOBJ);
        }
    }

    fn pump_messages() {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, null_mut_hwnd(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn colorref(red: u8, green: u8, blue: u8) -> u32 {
        red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
    }

    fn null_mut_hwnd() -> HWND {
        std::ptr::null_mut()
    }

    fn null_mut_hmenu() -> windows_sys::Win32::UI::WindowsAndMessaging::HMENU {
        std::ptr::null_mut()
    }
}

#[cfg(windows)]
pub fn show_native_overlay() -> Result<(), VoxError> {
    platform::show()
}

#[cfg(windows)]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    platform::hide()
}

#[cfg(not(windows))]
pub fn show_native_overlay() -> Result<(), VoxError> {
    Err(VoxError::Config(
        "原生浮窗只在 Windows 桌面模式可用".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    Ok(())
}
