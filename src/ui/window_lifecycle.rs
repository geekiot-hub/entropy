use super::*;

impl EntropyApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn poll_single_instance_signal(&mut self, ctx: &egui::Context) {
        let signal = read_single_instance_signal();
        if signal.is_empty() || signal == self.last_single_instance_signal {
            return;
        }
        self.last_single_instance_signal = signal;
        self.status_msg = "Entropy refreshed from a repeated launch".into();
        self.restore_from_tray(ctx);
        self.start_device_scan();
        ctx.request_repaint();
    }

    pub(super) fn restore_from_tray(&mut self, ctx: &egui::Context) {
        #[cfg(target_os = "windows")]
        if let Some(hwnd) = self.windows_hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
                };
                let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
                ShowWindow(hwnd, SW_SHOW);
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    #[cfg(target_os = "windows")]
    pub(super) fn cache_windows_hwnd(&mut self, frame: &eframe::Frame) {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if self.windows_hwnd.is_some() {
            return;
        }
        if let Ok(handle) = frame.window_handle() {
            if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                self.windows_hwnd = Some(win32.hwnd.get());
            }
        }
    }

    pub(super) fn handle_close_to_tray(&mut self, ctx: &egui::Context) {
        if !self.app_settings.minimize_to_tray_on_close {
            return;
        }
        if !ctx.input(|i| i.viewport().close_requested()) {
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        #[cfg(target_os = "windows")]
        {
            self.ensure_tray_icon(ctx);
            if let Some(hwnd) = self.windows_hwnd {
                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                    ShowWindow(hwnd as windows_sys::Win32::Foundation::HWND, SW_HIDE);
                }
                self.status_msg = "Entropy is running in the tray".into();
                return;
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.status_msg = "Entropy is running in the tray".into();
    }

    #[cfg(target_os = "windows")]
    pub(super) fn ensure_tray_icon(&mut self, ctx: &egui::Context) {
        if self.tray_icon.is_some() {
            return;
        }
        let mut rgba = Vec::with_capacity(16 * 16 * 4);
        let accent = app_accent();
        for y in 0..16 {
            for x in 0..16 {
                let inside = (3..=12).contains(&x) && (3..=12).contains(&y);
                let alpha = if inside { 255 } else { 0 };
                rgba.extend_from_slice(&[accent.r(), accent.g(), accent.b(), alpha]);
            }
        }
        let Ok(icon) = tray_icon::Icon::from_rgba(rgba, 16, 16) else {
            return;
        };
        let ctx_for_handler = ctx.clone();
        let hwnd_for_handler = self.windows_hwnd;
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    if let Some(hwnd) = hwnd_for_handler {
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
                            };
                            let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
                            ShowWindow(hwnd, SW_SHOW);
                            ShowWindow(hwnd, SW_RESTORE);
                            SetForegroundWindow(hwnd);
                        }
                    }
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx_for_handler.request_repaint();
                }
                _ => {}
            }
        }));
        let ctx_for_menu = ctx.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                if event.id == "entropy_tray_quit" {
                    TRAY_QUIT_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
                    ctx_for_menu.request_repaint();
                }
            },
        ));
        let tray_menu = tray_icon::menu::Menu::new();
        let quit_item =
            tray_icon::menu::MenuItem::with_id("entropy_tray_quit", "Quit Entropy", true, None);
        if let Err(e) = tray_menu.append(&quit_item) {
            log::warn!("failed to create tray menu: {e}");
        }
        match tray_icon::TrayIconBuilder::new()
            .with_tooltip("Entropy")
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .with_icon(icon)
            .build()
        {
            Ok(icon) => self.tray_icon = Some(icon),
            Err(e) => log::warn!("failed to create tray icon: {e}"),
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn handle_tray_quit_request(&mut self) {
        if TRAY_QUIT_REQUESTED.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.fallback_entropy_display_presets_before_exit();
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "windows")]
    pub(super) fn poll_tray_events(&mut self, ctx: &egui::Context) {
        use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => self.restore_from_tray(ctx),
                _ => {}
            }
        }
    }
}
