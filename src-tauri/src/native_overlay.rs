use crate::error::VoxError;

#[cfg(all(windows, not(test)))]
#[allow(dead_code)]
mod platform {
    use super::VoxError;
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Mutex, OnceLock};
    use std::thread::{self, JoinHandle};

    type Bool = i32;
    type ColorRef = u32;
    type HBitmap = *mut c_void;
    type HBrush = *mut c_void;
    type HCursor = *mut c_void;
    type Hdc = *mut c_void;
    type HIcon = *mut c_void;
    type HInstance = *mut c_void;
    type HMenu = *mut c_void;
    type HRegion = *mut c_void;
    type HgdiObj = *mut c_void;
    type Hwnd = *mut c_void;
    type HwndInsertAfter = *mut c_void;
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

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Size {
        cx: i32,
        cy: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct BlendFunction {
        blend_op: u8,
        blend_flags: u8,
        source_constant_alpha: u8,
        alpha_format: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct BitmapInfoHeader {
        bi_size: u32,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: u32,
        bi_size_image: u32,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: u32,
        bi_clr_important: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct RgbQuad {
        rgb_blue: u8,
        rgb_green: u8,
        rgb_red: u8,
        rgb_reserved: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct BitmapInfo {
        bmi_header: BitmapInfoHeader,
        bmi_colors: [RgbQuad; 1],
    }

    const AC_SRC_ALPHA: u8 = 0x01;
    const AC_SRC_OVER: u8 = 0x00;
    const BI_RGB: u32 = 0;
    const CS_VREDRAW: u32 = 0x0001;
    const CS_HREDRAW: u32 = 0x0002;
    const DIB_RGB_COLORS: u32 = 0;
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
    const ULW_ALPHA: u32 = 0x00000002;
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
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn GetSystemMetrics(index: i32) -> i32;
        fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
        fn KillTimer(hwnd: Hwnd, event_id: usize) -> Bool;
        fn PeekMessageW(message: *mut Msg, hwnd: Hwnd, min: u32, max: u32, remove: u32) -> Bool;
        fn RegisterClassW(wnd_class: *const WndClassW) -> u16;
        fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
        fn SetLayeredWindowAttributes(
            hwnd: Hwnd,
            color_key: ColorRef,
            alpha: u8,
            flags: u32,
        ) -> Bool;
        fn SetTimer(hwnd: Hwnd, event_id: usize, elapsed: u32, timer_func: *const c_void) -> usize;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
        fn TranslateMessage(message: *const Msg) -> Bool;
        fn UpdateLayeredWindow(
            hwnd: Hwnd,
            hdc_dst: Hdc,
            ppt_dst: *const Point,
            psize: *const Size,
            hdc_src: Hdc,
            ppt_src: *const Point,
            cr_key: ColorRef,
            pblend: *const BlendFunction,
            flags: u32,
        ) -> Bool;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
        fn CreateDIBSection(
            hdc: Hdc,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: *mut c_void,
            offset: u32,
        ) -> HBitmap;
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
        fn DeleteDC(hdc: Hdc) -> Bool;
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

    const WIDTH: i32 = 132;
    const HEIGHT: i32 = 44;
    const CAPSULE_X: f32 = 6.0;
    const CAPSULE_Y: f32 = 6.0;
    const CAPSULE_HEIGHT: f32 = 32.0;
    const MARGIN_BOTTOM: i32 = 92;
    const TIMER_ID: usize = 1;
    const TIMER_MS: u32 = 72;
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
                        let frame = frame_cell().lock().map(|frame| *frame).unwrap_or(0);
                        unsafe {
                            render_layered_overlay(hwnd, mode, frame).ok();
                        };
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

            if render_layered_overlay(hwnd, NativeOverlayMode::Recording, 0).is_err() {
                DestroyWindow(hwnd);
                return Err(format!(
                    "render native overlay failed, Win32 error: {}",
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
                let frame = if let Ok(mut frame) = frame_cell().lock() {
                    *frame = frame.wrapping_add(1);
                    *frame
                } else {
                    0
                };
                render_layered_overlay(hwnd, current_mode(), frame).ok();
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
        let frame = frame_cell().lock().map(|frame| *frame).unwrap_or(0);
        render_layered_overlay(hwnd, mode, frame).ok();
    }

    unsafe fn render_layered_overlay(
        hwnd: Hwnd,
        mode: NativeOverlayMode,
        frame: u32,
    ) -> Result<(), ()> {
        let mut buffer = render_overlay_pixels(mode, frame);
        let screen_dc = GetDC(null_mut());
        if screen_dc.is_null() {
            return Err(());
        }

        let memory_dc = CreateCompatibleDC(screen_dc);
        if memory_dc.is_null() {
            ReleaseDC(null_mut(), screen_dc);
            return Err(());
        }

        let bitmap_info = BitmapInfo {
            bmi_header: BitmapInfoHeader {
                bi_size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                bi_width: WIDTH,
                bi_height: -HEIGHT,
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: BI_RGB,
                bi_size_image: (WIDTH * HEIGHT * 4) as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = null_mut();
        let bitmap = CreateDIBSection(
            screen_dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            null_mut(),
            0,
        );
        if bitmap.is_null() || bits.is_null() {
            DeleteDC(memory_dc);
            ReleaseDC(null_mut(), screen_dc);
            return Err(());
        }

        std::ptr::copy_nonoverlapping(buffer.as_mut_ptr(), bits.cast::<u8>(), buffer.len());
        let old_bitmap = SelectObject(memory_dc, bitmap as HgdiObj);
        let size = Size {
            cx: WIDTH,
            cy: HEIGHT,
        };
        let source = Point { x: 0, y: 0 };
        let blend = BlendFunction {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };
        let result = UpdateLayeredWindow(
            hwnd,
            screen_dc,
            null(),
            &size,
            memory_dc,
            &source,
            0,
            &blend,
            ULW_ALPHA,
        );
        SelectObject(memory_dc, old_bitmap);
        DeleteObject(bitmap as HgdiObj);
        DeleteDC(memory_dc);
        ReleaseDC(null_mut(), screen_dc);

        if result == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    fn render_overlay_pixels(mode: NativeOverlayMode, frame: u32) -> Vec<u8> {
        let mut canvas = Canvas::new(WIDTH as usize, HEIGHT as usize, 3);
        canvas.draw_capsule(
            0.0,
            CAPSULE_Y,
            WIDTH as f32,
            CAPSULE_HEIGHT,
            CAPSULE_HEIGHT / 2.0,
        );
        match mode {
            NativeOverlayMode::Recording => draw_recording_bars(&mut canvas, frame),
            NativeOverlayMode::Transcribing => draw_transcribing_dots(&mut canvas, frame),
        }
        canvas.into_bgra()
    }

    struct Canvas {
        width: usize,
        height: usize,
        scale: usize,
        pixels: Vec<LinearPixel>,
    }

    #[derive(Clone, Copy, Default)]
    struct LinearPixel {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    }

    #[derive(Clone, Copy)]
    struct Rgba {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    }

    impl Canvas {
        fn new(width: usize, height: usize, scale: usize) -> Self {
            Self {
                width,
                height,
                scale,
                pixels: vec![LinearPixel::default(); width * height * scale * scale],
            }
        }

        fn draw_capsule(&mut self, x: f32, y: f32, width: f32, height: f32, radius: f32) {
            self.fill_round_rect(
                x,
                y,
                width,
                height,
                radius,
                Rgba {
                    red: 232.0,
                    green: 247.0,
                    blue: 226.0,
                    alpha: 0.88,
                },
            );
            self.fill_round_rect(
                x + 1.2,
                y + 1.2,
                width - 2.4,
                height - 2.4,
                radius - 1.2,
                Rgba {
                    red: 250.0,
                    green: 255.0,
                    blue: 244.0,
                    alpha: 0.96,
                },
            );
        }

        fn fill_round_rect(
            &mut self,
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            radius: f32,
            color: Rgba,
        ) {
            let scale = self.scale as f32;
            let min_x = (x * scale).floor().max(0.0) as usize;
            let max_x = ((x + width) * scale)
                .ceil()
                .min((self.width * self.scale) as f32) as usize;
            let min_y = (y * scale).floor().max(0.0) as usize;
            let max_y = ((y + height) * scale)
                .ceil()
                .min((self.height * self.scale) as f32) as usize;
            let left = x * scale;
            let top = y * scale;
            let right = (x + width) * scale;
            let bottom = (y + height) * scale;
            let radius = radius * scale;

            for sy in min_y..max_y {
                for sx in min_x..max_x {
                    let px = sx as f32 + 0.5;
                    let py = sy as f32 + 0.5;
                    let qx = px.clamp(left + radius, right - radius);
                    let qy = py.clamp(top + radius, bottom - radius);
                    let dx = px - qx;
                    let dy = py - qy;
                    if (dx * dx + dy * dy).sqrt() <= radius {
                        self.blend_scaled(sx, sy, color);
                    }
                }
            }
        }

        fn blend_scaled(&mut self, sx: usize, sy: usize, color: Rgba) {
            let scaled_width = self.width * self.scale;
            let pixel = &mut self.pixels[sy * scaled_width + sx];
            let inv = 1.0 - color.alpha;
            pixel.red = color.red * color.alpha + pixel.red * inv;
            pixel.green = color.green * color.alpha + pixel.green * inv;
            pixel.blue = color.blue * color.alpha + pixel.blue * inv;
            pixel.alpha = color.alpha + pixel.alpha * inv;
        }

        fn fill_bar(&mut self, x: f32, y: f32, width: f32, height: f32, color: Rgba) {
            self.fill_round_rect(x, y, width, height, width / 2.0, color);
        }

        fn fill_dot(&mut self, cx: f32, cy: f32, radius: f32, color: Rgba) {
            self.fill_round_rect(
                cx - radius,
                cy - radius,
                radius * 2.0,
                radius * 2.0,
                radius,
                color,
            );
        }

        fn into_bgra(self) -> Vec<u8> {
            let mut output = vec![0; self.width * self.height * 4];
            let scale_area = (self.scale * self.scale) as f32;
            let scaled_width = self.width * self.scale;
            for y in 0..self.height {
                for x in 0..self.width {
                    let mut red = 0.0;
                    let mut green = 0.0;
                    let mut blue = 0.0;
                    let mut alpha = 0.0;
                    for yy in 0..self.scale {
                        for xx in 0..self.scale {
                            let pixel = self.pixels
                                [(y * self.scale + yy) * scaled_width + (x * self.scale + xx)];
                            red += pixel.red;
                            green += pixel.green;
                            blue += pixel.blue;
                            alpha += pixel.alpha;
                        }
                    }
                    red /= scale_area;
                    green /= scale_area;
                    blue /= scale_area;
                    alpha = (alpha / scale_area).clamp(0.0, 1.0);
                    let index = (y * self.width + x) * 4;
                    output[index] = blue.clamp(0.0, 255.0) as u8;
                    output[index + 1] = green.clamp(0.0, 255.0) as u8;
                    output[index + 2] = red.clamp(0.0, 255.0) as u8;
                    output[index + 3] = (alpha * 255.0).round() as u8;
                }
            }
            output
        }
    }

    fn draw_recording_bars(canvas: &mut Canvas, frame: u32) {
        let colors = recording_palette(frame / 8);
        let base_heights = [
            4.0, 7.0, 10.0, 5.0, 12.0, 8.0, 3.0, 11.0, 6.0, 13.0, 9.0, 4.0, 12.0, 7.0, 10.0, 5.0,
            8.0, 11.0, 6.0, 3.0,
        ];

        for (index, base_height) in base_heights.iter().enumerate() {
            let color = with_alpha(
                dim_rgba(colors[index % colors.len()], sweep_intensity(index, frame)),
                0.95,
            );
            let glow = with_alpha(color, 0.22);
            let pulse_frame = frame as usize / 2;
            let pulse_phase = (index + 6 - (pulse_frame % 6)) % 6;
            let pulse: f32 = [0.0, 1.0, 3.0, 5.0, 3.0, 1.0][pulse_phase];
            let height = (*base_height + pulse).min(18.5_f32);
            let x = CAPSULE_X + 13.0 + index as f32 * 5.0;
            let y = CAPSULE_Y + CAPSULE_HEIGHT / 2.0 - height / 2.0;

            canvas.fill_bar(x - 0.8, y - 1.8, 4.0, height + 3.6, glow);
            canvas.fill_bar(x, y, 2.4, height, color);
        }
    }

    fn draw_transcribing_dots(canvas: &mut Canvas, frame: u32) {
        let colors = recording_palette(0);
        draw_recording_ghost_bars(canvas);

        for index in 0..6 {
            let color = with_alpha(
                dim_rgba(colors[index % colors.len()], dot_intensity(index, frame)),
                0.98,
            );
            let glow = with_alpha(color, 0.18);
            let phase = ((frame as usize / 2) + 11 - index) % 12;
            let radius = [
                1.8, 1.9, 2.1, 2.5, 2.75, 2.5, 2.1, 1.9, 1.8, 1.65, 1.65, 1.75,
            ][phase];
            let cx = CAPSULE_X + 40.0 + index as f32 * 8.0;
            let cy = CAPSULE_Y + CAPSULE_HEIGHT / 2.0;
            canvas.fill_dot(cx, cy, 4.2, glow);
            canvas.fill_dot(cx, cy, radius, color);
        }
    }

    fn draw_recording_ghost_bars(canvas: &mut Canvas) {
        let ghost = Rgba {
            red: 112.0,
            green: 163.0,
            blue: 119.0,
            alpha: 0.28,
        };
        let heights = [
            3.0, 5.0, 7.0, 4.0, 8.0, 6.0, 3.0, 7.0, 4.0, 8.0, 6.0, 3.0, 8.0, 5.0, 7.0, 4.0, 6.0,
            7.0, 4.0, 3.0,
        ];
        for (index, height) in heights.iter().enumerate() {
            let x = CAPSULE_X + 13.0 + index as f32 * 5.0;
            let y = CAPSULE_Y + CAPSULE_HEIGHT / 2.0 - height / 2.0;
            canvas.fill_bar(x, y, 2.4, *height, ghost);
        }
    }

    fn recording_palette(frame: u32) -> [Rgba; 6] {
        let phase = (frame % 6) as usize;
        let palette = [
            rgb(44, 122, 63),
            rgb(79, 184, 102),
            rgb(123, 220, 139),
            rgb(199, 239, 189),
            rgb(104, 184, 118),
            rgb(35, 107, 56),
        ];
        [
            palette[(6 - phase) % 6],
            palette[(7 - phase) % 6],
            palette[(8 - phase) % 6],
            palette[(9 - phase) % 6],
            palette[(10 - phase) % 6],
            palette[(11 - phase) % 6],
        ]
    }

    fn sweep_intensity(index: usize, frame: u32) -> f32 {
        let travel = (frame as usize / 2) % 36;
        let sweep = if travel <= 18 { travel } else { 36 - travel } + 1;
        let direct = index.abs_diff(sweep);
        let wrapped = 20 - direct.min(20);
        let distance = direct.min(wrapped);
        match distance {
            0 | 1 => 1.0,
            2 | 3 => 0.90,
            4 | 5 => 0.80,
            _ => 0.70,
        }
    }

    fn dot_intensity(index: usize, frame: u32) -> f32 {
        let phase = ((frame as usize / 2) + 11 - index) % 12;
        match phase {
            3..=5 => 1.0,
            2 | 6 => 0.82,
            1 | 7 | 11 => 0.66,
            _ => 0.50,
        }
    }

    fn rgb(red: u8, green: u8, blue: u8) -> Rgba {
        Rgba {
            red: red as f32,
            green: green as f32,
            blue: blue as f32,
            alpha: 1.0,
        }
    }

    fn dim_rgba(color: Rgba, ratio: f32) -> Rgba {
        Rgba {
            red: color.red * ratio,
            green: color.green * ratio,
            blue: color.blue * ratio,
            alpha: color.alpha,
        }
    }

    fn with_alpha(color: Rgba, alpha: f32) -> Rgba {
        Rgba { alpha, ..color }
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
