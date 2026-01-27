use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1SolidColorBrush,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use crate::config::BorderColor;
use crate::overlay::window;

/// Manages the border overlay rendering via Direct2D.
pub struct BorderOverlay {
    pub hwnd: HWND,
    factory: ID2D1Factory,
    render_target: Option<ID2D1HwndRenderTarget>,
    brush: Option<ID2D1SolidColorBrush>,
    thickness: f32,
    color: BorderColor,
    last_overlay_rect: RECT,
}

impl BorderOverlay {
    pub fn new(color: BorderColor, thickness: f32) -> Option<Self> {
        let hwnd = window::create_overlay_window("WhereIsMyWindowBorder", 1, 1)?;
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).ok()?
        };

        window::set_colorkey(hwnd);

        Some(Self {
            hwnd,
            factory,
            render_target: None,
            brush: None,
            thickness,
            color,
            last_overlay_rect: RECT::default(),
        })
    }

    fn create_render_target(&mut self) {
        unsafe {
            let mut client_rect = RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(self.hwnd, &mut client_rect);
            let width = (client_rect.right - client_rect.left).max(1) as u32;
            let height = (client_rect.bottom - client_rect.top).max(1) as u32;

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
                    width,
                    height,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
            };

            if let Ok(rt) = self.factory.CreateHwndRenderTarget(&render_props, &hwnd_props) {
                let color = D2D1_COLOR_F {
                    r: self.color.r,
                    g: self.color.g,
                    b: self.color.b,
                    a: self.color.a,
                };
                if let Ok(brush) = rt.CreateSolidColorBrush(&color, None) {
                    self.brush = Some(brush);
                }
                self.render_target = Some(rt);
            }
        }
    }

    /// Update overlay position and redraw border around the target rect.
    /// Only recreates the render target when the overlay size changes.
    pub fn update(&mut self, target_rect: &RECT) {
        let t = self.thickness as i32;
        let overlay_rect = RECT {
            left: target_rect.left - t,
            top: target_rect.top - t,
            right: target_rect.right + t,
            bottom: target_rect.bottom + t,
        };

        // Skip if nothing changed
        if overlay_rect == self.last_overlay_rect {
            return;
        }

        let old_w = self.last_overlay_rect.right - self.last_overlay_rect.left;
        let old_h = self.last_overlay_rect.bottom - self.last_overlay_rect.top;
        let new_w = overlay_rect.right - overlay_rect.left;
        let new_h = overlay_rect.bottom - overlay_rect.top;
        let size_changed = new_w != old_w || new_h != old_h;

        self.last_overlay_rect = overlay_rect;

        window::reposition_overlay(self.hwnd, &overlay_rect);

        // Only recreate render target and re-render when size changes.
        // Position-only moves reuse the existing D2D content (border shape
        // depends on size, not position).
        if size_changed {
            self.render_target = None;
            self.brush = None;
            self.create_render_target();
            self.render(&overlay_rect);
        }
    }

    /// Move border to a new target on focus change.
    /// Hides first to prevent ghost at the old position.
    pub fn move_to(&mut self, target_rect: &RECT) {
        window::hide_overlay(self.hwnd);
        self.last_overlay_rect = RECT::default(); // force size recalc
        self.update(target_rect);
        window::show_overlay(self.hwnd);
    }

    fn render(&self, overlay_rect: &RECT) {
        let Some(rt) = &self.render_target else { return };
        let Some(brush) = &self.brush else { return };

        let w = (overlay_rect.right - overlay_rect.left) as f32;
        let h = (overlay_rect.bottom - overlay_rect.top) as f32;
        let t = self.thickness;

        unsafe {
            rt.BeginDraw();
            // Clear to magenta — color-key makes these pixels fully transparent
            let clear_color = D2D1_COLOR_F {
                r: 1.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            };
            rt.Clear(Some(&clear_color));

            // Top border
            rt.FillRectangle(
                &D2D_RECT_F { left: 0.0, top: 0.0, right: w, bottom: t },
                brush,
            );
            // Bottom border
            rt.FillRectangle(
                &D2D_RECT_F { left: 0.0, top: h - t, right: w, bottom: h },
                brush,
            );
            // Left border
            rt.FillRectangle(
                &D2D_RECT_F { left: 0.0, top: t, right: t, bottom: h - t },
                brush,
            );
            // Right border
            rt.FillRectangle(
                &D2D_RECT_F { left: w - t, top: t, right: w, bottom: h - t },
                brush,
            );

            let _ = rt.EndDraw(None, None);
        }
    }

    pub fn hide(&self) {
        window::hide_overlay(self.hwnd);
    }

}
