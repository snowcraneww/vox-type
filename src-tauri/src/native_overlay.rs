use crate::error::VoxError;

#[cfg(all(windows, not(test)))]
mod platform {
    use super::VoxError;
    use std::ffi::c_void;
    use std::ptr::null;
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Mutex, OnceLock};
    use std::thread::{self, JoinHandle};

    type Bool = i32;
    type ColorRef = u32;
    type HBrush = *mut c_void;
    type HCursor = *mut c_void;
    type Hdc = *mut c_void;
    type HIcon = *mut c_void;
    type HInstance = *mut c_void;
    type HMenu = *mut c_void;
    type HRegion = *mut c_void;
    type HgdiObj = *mut c_void;
    type Hwnd = *mut c_void;
    type LParam = isize;
    type LResult = isize;
    type WParam = usize;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PaintStruct {
        hdc: Hdc,
        f_erase: Bool,
        rc_paint: Rect,
        f_restore: Bool,
        f_inc_update: Bool,
        rgb_reserved: [u8; 32],
    }

    impl Default for PaintStruct {
        fn default() -> Self {
            Self {
                hdc: std::ptr::null_mut(),
                f_erase: 0,
                rc_paint: Rect::default(),
                f_restore: 0,
                f_inc_update: 0,
                rgb_reserved: [0; 32],
            }
        }
    }

    #[repr(C)]
    #[derive(Default)]
    struct WndClassW {
        style: u32,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, u32, WParam, LParam) -> LResult>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: HInstance,
        h_icon: HIcon,
        h_cursor: HCursor,
        hbr_background: HBrush,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Msg {
        hwnd: Hwnd,
        message: u32,
        w_param: WParam,
        l_param: LParam,
        time: u32,
        pt: Point,
    }

    const CS_VREDRAW: u32 = 0x0001;
    const CS_HREDRAW: u32 = 0x0002;
    const LWA_ALPHA: u32 = 0x00000002;
    const PM_REMOVE: u32 = 0x0001;
    const PS_SOLID: i32 = 0;
    const SM_CXSCREEN: i32 = 0;
    const SM_CYSCREEN: i32 = 1;
    const SW_HIDE: i32 = 0;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const TRANSPARENT: i32 = 1;
    const WM_DESTROY: u32 = 0x0002;
    const WM_PAINT: u32 = 0x000F;
    const WM_TIMER: u32 = 0x0113;
    const WS_EX_LAYERED: u32 = 0x00080000;
    const WS_EX_NOACTIVATE: u32 = 0x08000000;
    const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    const WS_EX_TOPMOST: u32 = 0x00000008;
    const WS_POPUP: u32 = 0x80000000;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLastError() -> u32;
        fn GetModuleHandleW(module_name: *const u16) -> HInstance;
    }

    #[link(name = "user32")]
    extern "system" {
        fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
        fn CreateWindowExW(
            ex_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: HMenu,
            instance: HInstance,
            param: *const c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, message: u32, wparam: WParam, lparam: LParam) -> LResult;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn DispatchMessageW(message: *const Msg) -> LResult;
        fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> Bool;
        fn GetSystemMetrics(index: i32) -> i32;
        fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
        fn KillTimer(hwnd: Hwnd, event_id: usize) -> Bool;
        fn PeekMessageW(message: *mut Msg, hwnd: Hwnd, min: u32, max: u32, remove: u32) -> Bool;
        fn RegisterClassW(wnd_class: *const WndClassW) -> u16;
        fn SetLayeredWindowAttributes(
            hwnd: Hwnd,
            color_key: ColorRef,
            alpha: u8,
            flags: u32,
        ) -> Bool;
        fn SetTimer(hwnd: Hwnd, event_id: usize, elapsed: u32, timer_func: *const c_void) -> usize;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
        fn TranslateMessage(message: *const Msg) -> Bool;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreatePen(style: i32, width: i32, color: ColorRef) -> HgdiObj;
        fn CreateRoundRectRgn(
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            width: i32,
            height: i32,
        ) -> HRegion;
        fn CreateSolidBrush(color: ColorRef) -> HBrush;
        fn DeleteObject(object: HgdiObj) -> Bool;
        fn RoundRect(
            hdc: Hdc,
            left: i32,
            top: i32,
            right: i32,
            bottom: i32,
            width: i32,
            height: i32,
        ) -> Bool;
        fn SelectObject(hdc: Hdc, object: HgdiObj) -> HgdiObj;
        fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
        fn SetWindowRgn(hwnd: Hwnd, region: HRegion, redraw: Bool) -> i32;
    }

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
        SetMode(NativeOverlayMode),
        Hide,
        Stop,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeOverlayMode {
        Recording,
        Transcribing,
    }

    struct OverlayThread {
        sender: Sender<OverlayCommand>,
        handle: JoinHandle<()>,
    }

    pub fn show(mode: NativeOverlayMode) -> Result<(), VoxError> {
        let mut guard = overlay_thread_cell()
            .lock()
            .map_err(|error| VoxError::Config(format!("锁定原生浮窗线程失败：{error}")))?;

        if let Some(thread) = guard.as_ref() {
            let _ = thread.sender.send(OverlayCommand::SetMode(mode));
            return Ok(());
        }

        let (ready_sender, ready_receiver) = mpsc::channel();
        let (command_sender, command_receiver) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("voxtype-native-overlay".to_string())
            .spawn(move || run_overlay_window(ready_sender, command_receiver, mode))
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
        initial_mode: NativeOverlayMode,
    ) {
        let hwnd = match create_overlay_window() {
            Ok(hwnd) => hwnd,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };

        let _ = ready_sender.send(Ok(()));
        let mut mode = initial_mode;

        loop {
            if let Ok(command) = command_receiver.try_recv() {
                match command {
                    OverlayCommand::SetMode(next_mode) => {
                        mode = next_mode;
                        unsafe { InvalidateRect(hwnd, null(), 0) };
                    }
                    OverlayCommand::Hide | OverlayCommand::Stop => {
                        unsafe {
                            ShowWindow(hwnd, SW_HIDE);
                            DestroyWindow(hwnd);
                        }
                        return;
                    }
                }
            }

            set_current_mode(mode);
            pump_messages();
            thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    fn create_overlay_window() -> Result<Hwnd, String> {
        unsafe {
            let instance = GetModuleHandleW(null());
            let wnd_class = WndClassW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfn_wnd_proc: Some(window_proc),
                h_instance: instance,
                lpsz_class_name: CLASS_NAME.as_ptr(),
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
                DeleteObject(region as HgdiObj);
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

            if SetTimer(hwnd, TIMER_ID, TIMER_MS, null()) == 0 {
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
        hwnd: Hwnd,
        message: u32,
        wparam: WParam,
        lparam: LParam,
    ) -> LResult {
        match message {
            WM_PAINT => {
                paint_overlay(hwnd, current_mode());
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

    unsafe fn paint_overlay(hwnd: Hwnd, mode: NativeOverlayMode) {
        let mut paint = PaintStruct::default();
        let hdc = BeginPaint(hwnd, &mut paint);
        if hdc.is_null() {
            return;
        }

        let background = CreateSolidBrush(colorref(3, 3, 5));
        let border_pen = CreatePen(PS_SOLID, 1, colorref(3, 3, 5));
        let old_brush = SelectObject(hdc, background as HgdiObj);
        let old_pen = SelectObject(hdc, border_pen as HgdiObj);
        RoundRect(hdc, 0, 0, WIDTH, HEIGHT, HEIGHT, HEIGHT);
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        DeleteObject(border_pen as HgdiObj);
        DeleteObject(background as HgdiObj);

        SetBkMode(hdc, TRANSPARENT);
        let frame = frame_cell().lock().map(|frame| *frame).unwrap_or(0);
        match mode {
            NativeOverlayMode::Recording => draw_recording_bars(hdc, frame),
            NativeOverlayMode::Transcribing => draw_transcribing_dots(hdc, frame),
        }

        EndPaint(hwnd, &paint);
    }

    unsafe fn draw_recording_bars(hdc: Hdc, frame: u32) {
        let colors = recording_palette(frame);
        let base_heights = [
            4, 7, 10, 5, 12, 8, 3, 11, 6, 13, 9, 4, 12, 7, 10, 5, 8, 11, 6, 3,
        ];

        for (index, base_height) in base_heights.iter().enumerate() {
            let color = colors[index % colors.len()];
            let glow = dim_color(color, 0.32);
            let pulse = [0, 2, 5, 8, 5, 2][(index + frame as usize) % 6];
            let height = (*base_height + pulse).min(21);
            let x = 13 + index as i32 * 5;
            let y = 16 - height / 2;

            let glow_brush = CreateSolidBrush(glow);
            let glow_pen = CreatePen(PS_SOLID, 1, glow);
            let old_glow_brush = SelectObject(hdc, glow_brush as HgdiObj);
            let old_glow_pen = SelectObject(hdc, glow_pen as HgdiObj);
            RoundRect(hdc, x - 1, y - 2, x + 4, y + height + 2, 5, 5);
            SelectObject(hdc, old_glow_pen);
            SelectObject(hdc, old_glow_brush);
            DeleteObject(glow_pen as HgdiObj);
            DeleteObject(glow_brush as HgdiObj);

            let brush = CreateSolidBrush(color);
            let pen = CreatePen(PS_SOLID, 1, color);
            let old_brush = SelectObject(hdc, brush as HgdiObj);
            let old_pen = SelectObject(hdc, pen as HgdiObj);
            RoundRect(hdc, x, y, x + 3, y + height, 4, 4);

            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            DeleteObject(pen as HgdiObj);
            DeleteObject(brush as HgdiObj);
        }
    }

    unsafe fn draw_transcribing_dots(hdc: Hdc, frame: u32) {
        let colors = recording_palette(frame / 2);

        draw_recording_ghost_bars(hdc);

        for index in 0..6 {
            let color = colors[index % colors.len()];
            let glow = dim_color(color, 0.38);
            let phase = (frame as usize + index * 2) % 12;
            let radius = if phase <= 2 {
                3
            } else if phase <= 7 {
                2
            } else {
                1
            };
            let cx = 40 + index as i32 * 8;
            let cy = 16;

            let glow_brush = CreateSolidBrush(glow);
            let glow_pen = CreatePen(PS_SOLID, 1, glow);
            let old_glow_brush = SelectObject(hdc, glow_brush as HgdiObj);
            let old_glow_pen = SelectObject(hdc, glow_pen as HgdiObj);
            RoundRect(hdc, cx - 5, cy - 5, cx + 6, cy + 6, 10, 10);
            SelectObject(hdc, old_glow_pen);
            SelectObject(hdc, old_glow_brush);
            DeleteObject(glow_pen as HgdiObj);
            DeleteObject(glow_brush as HgdiObj);

            let brush = CreateSolidBrush(color);
            let pen = CreatePen(PS_SOLID, 1, color);
            let old_brush = SelectObject(hdc, brush as HgdiObj);
            let old_pen = SelectObject(hdc, pen as HgdiObj);
            RoundRect(
                hdc,
                cx - radius,
                cy - radius,
                cx + radius + 1,
                cy + radius + 1,
                radius * 2,
                radius * 2,
            );
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            DeleteObject(pen as HgdiObj);
            DeleteObject(brush as HgdiObj);
        }
    }

    unsafe fn draw_recording_ghost_bars(hdc: Hdc) {
        let ghost = colorref(16, 16, 20);
        let brush = CreateSolidBrush(ghost);
        let pen = CreatePen(PS_SOLID, 1, ghost);
        let old_brush = SelectObject(hdc, brush as HgdiObj);
        let old_pen = SelectObject(hdc, pen as HgdiObj);
        let heights = [3, 5, 7, 4, 8, 6, 3, 7, 4, 8, 6, 3, 8, 5, 7, 4, 6, 7, 4, 3];
        for (index, height) in heights.iter().enumerate() {
            let x = 13 + index as i32 * 5;
            let y = 16 - height / 2;
            RoundRect(hdc, x, y, x + 3, y + height, 4, 4);
        }
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        DeleteObject(pen as HgdiObj);
        DeleteObject(brush as HgdiObj);
    }

    fn recording_palette(frame: u32) -> [ColorRef; 6] {
        let phase = (frame % 6) as usize;
        let palette = [
            colorref(90, 200, 250),
            colorref(126, 231, 255),
            colorref(52, 199, 89),
            colorref(255, 214, 10),
            colorref(255, 45, 146),
            colorref(191, 90, 242),
        ];
        [
            palette[phase % 6],
            palette[(phase + 1) % 6],
            palette[(phase + 2) % 6],
            palette[(phase + 3) % 6],
            palette[(phase + 4) % 6],
            palette[(phase + 5) % 6],
        ]
    }

    fn dim_color(color: ColorRef, ratio: f32) -> ColorRef {
        let red = (color & 0xff) as f32;
        let green = ((color >> 8) & 0xff) as f32;
        let blue = ((color >> 16) & 0xff) as f32;
        colorref(
            (red * ratio) as u8,
            (green * ratio) as u8,
            (blue * ratio) as u8,
        )
    }

    fn current_mode_cell() -> &'static Mutex<NativeOverlayMode> {
        static CURRENT_MODE: OnceLock<Mutex<NativeOverlayMode>> = OnceLock::new();
        CURRENT_MODE.get_or_init(|| Mutex::new(NativeOverlayMode::Recording))
    }

    fn current_mode() -> NativeOverlayMode {
        current_mode_cell()
            .lock()
            .map(|mode| *mode)
            .unwrap_or(NativeOverlayMode::Recording)
    }

    fn set_current_mode(mode: NativeOverlayMode) {
        if let Ok(mut current_mode) = current_mode_cell().lock() {
            *current_mode = mode;
        }
    }

    fn pump_messages() {
        unsafe {
            let mut msg = Msg::default();
            while PeekMessageW(&mut msg, null_mut_hwnd(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn colorref(red: u8, green: u8, blue: u8) -> u32 {
        red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
    }

    fn null_mut_hwnd() -> Hwnd {
        std::ptr::null_mut()
    }

    fn null_mut_hmenu() -> HMenu {
        std::ptr::null_mut()
    }
}

#[cfg(all(windows, not(test)))]
pub fn show_native_overlay() -> Result<(), VoxError> {
    platform::show(platform::NativeOverlayMode::Recording)
}

#[cfg(all(windows, not(test)))]
pub fn show_native_transcribing_overlay() -> Result<(), VoxError> {
    platform::show(platform::NativeOverlayMode::Transcribing)
}

#[cfg(all(windows, not(test)))]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    platform::hide()
}

#[cfg(any(not(windows), test))]
pub fn show_native_overlay() -> Result<(), VoxError> {
    Err(VoxError::Config(
        "原生浮窗只在 Windows 桌面模式可用".to_string(),
    ))
}

#[cfg(any(not(windows), test))]
pub fn show_native_transcribing_overlay() -> Result<(), VoxError> {
    show_native_overlay()
}

#[cfg(any(not(windows), test))]
pub fn hide_native_overlay() -> Result<(), VoxError> {
    Ok(())
}
