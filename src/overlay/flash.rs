use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use crate::overlay::window;

/// Full-screen flash overlay shown when focus moves to a different monitor.
pub struct FlashOverlay {
    pub hwnd: HWND,
    factory: ID2D1Factory,
}

impl FlashOverlay {
    pub fn new(opacity: f32) -> Option<Self> {
        let hwnd = window::create_overlay_window("WhereIsMyWindowFlash", 1, 1)?;
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).ok()?
        };

        // Use whole-window alpha for flash transparency
        window::set_alpha(hwnd, (opacity * 255.0) as u8);

        Some(Self {
            hwnd,
            factory,
        })
    }

    /// Show flash over the given monitor rect.
    pub fn flash(&self, monitor_rect: &RECT) {
        window::reposition_overlay(self.hwnd, monitor_rect);
        self.render(monitor_rect);
        window::show_overlay(self.hwnd);
    }

    pub fn set_opacity(&self, opacity: f32) {
        window::set_alpha(self.hwnd, (opacity * 255.0) as u8);
    }

    pub fn hide(&self) {
        window::hide_overlay(self.hwnd);
    }

    fn render(&self, rect: &RECT) {
        unsafe {
            let w = (rect.right - rect.left).max(1) as u32;
            let h = (rect.bottom - rect.top).max(1) as u32;

            let render_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                ..Default::default()
            };

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: self.hwnd,
                pixelSize: windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U {
                    width: w,
                    height: h,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
            };

            let Ok(rt) = self.factory.CreateHwndRenderTarget(&render_props, &hwnd_props) else {
                return;
            };

            rt.BeginDraw();

            // Render opaque blue — window-level LWA_ALPHA controls transparency
            let color = D2D1_COLOR_F {
                r: 0.0,
                g: 0.47,
                b: 0.84,
                a: 1.0,
            };
            rt.Clear(Some(&color));

            let _ = rt.EndDraw(None, None);
        }
    }
}
