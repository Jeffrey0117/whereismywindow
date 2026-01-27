use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use crate::config::{BorderColor, BorderStyle};
use crate::overlay::window;

const GLOW_LAYERS: usize = 4;

/// Glow layer definitions: (thickness_px, color)
/// Outermost → innermost, each layer draws a frame at that offset.
fn glow_colors(base: &BorderColor) -> [(f32, D2D1_COLOR_F); GLOW_LAYERS] {
    [
        (2.0, D2D1_COLOR_F { r: base.r * 0.15, g: base.g * 0.15, b: base.b * 0.15, a: 1.0 }),
        (2.0, D2D1_COLOR_F { r: base.r * 0.35, g: base.g * 0.35, b: base.b * 0.35, a: 1.0 }),
        (2.0, D2D1_COLOR_F { r: base.r * 0.65, g: base.g * 0.65, b: base.b * 0.65, a: 1.0 }),
        (2.0, D2D1_COLOR_F { r: base.r, g: base.g, b: base.b, a: 1.0 }),
    ]
}

fn glow_total_thickness() -> f32 {
    GLOW_LAYERS as f32 * 2.0 // 8px total
}

/// Manages the border overlay rendering via Direct2D.
pub struct BorderOverlay {
    pub hwnd: HWND,
    factory: ID2D1Factory,
    render_target: Option<ID2D1HwndRenderTarget>,
    thickness: f32,
    color: BorderColor,
    style: BorderStyle,
    last_overlay_rect: RECT,
}

impl BorderOverlay {
    pub fn new(color: BorderColor, thickness: f32, style: BorderStyle) -> Option<Self> {
        let hwnd = window::create_overlay_window("WhereIsMyWindowBorder", 1, 1)?;
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).ok()?
        };

        window::set_colorkey(hwnd);

        Some(Self {
            hwnd,
            factory,
            render_target: None,
            thickness,
            color,
            style,
            last_overlay_rect: RECT::default(),
        })
    }

    fn effective_thickness(&self) -> f32 {
        match self.style {
            BorderStyle::Solid => self.thickness,
            BorderStyle::Glow => glow_total_thickness(),
        }
    }

    pub fn set_style(&mut self, style: BorderStyle) {
        if self.style != style {
            self.style = style;
            self.last_overlay_rect = RECT::default();
        }
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
                self.render_target = Some(rt);
            }
        }
    }

    /// Update overlay position and redraw border around the target rect.
    pub fn update(&mut self, target_rect: &RECT) {
        let t = self.effective_thickness() as i32;
        let overlay_rect = RECT {
            left: target_rect.left - t,
            top: target_rect.top - t,
            right: target_rect.right + t,
            bottom: target_rect.bottom + t,
        };

        if overlay_rect == self.last_overlay_rect {
            return;
        }

        let old_w = self.last_overlay_rect.right - self.last_overlay_rect.left;
        let old_h = self.last_overlay_rect.bottom - self.last_overlay_rect.top;
        let new_w = overlay_rect.right - overlay_rect.left;
        let new_h = overlay_rect.bottom - overlay_rect.top;
        let size_changed = new_w != old_w || new_h != old_h;

        if size_changed {
            // Alpha=0 protection: surface is invalidated on resize,
            // hide content until re-render completes to avoid black flash.
            window::set_fully_transparent(self.hwnd);
        }

        self.last_overlay_rect = overlay_rect;
        window::reposition_overlay(self.hwnd, &overlay_rect);

        if size_changed {
            self.render_target = None;
            self.create_render_target();
            if self.render(&overlay_rect) {
                window::set_colorkey(self.hwnd);
            } else {
                // Render failed — hide overlay instead of showing black
                window::hide_overlay(self.hwnd);
            }
        }
    }

    /// Move border to a new target on focus change.
    /// Uses alpha=0 to prevent ghost flash — window stays "visible" to the system
    /// (so D2D works) but is fully invisible to the user during transition:
    /// 1. Set alpha=0 (+ colorkey) — entire window invisible, but not SW_HIDE'd
    /// 2. Reposition + resize (invisible, no ghost)
    /// 3. D2D render (window is visible to system, render works correctly)
    /// 4. Restore colorkey-only mode — rendered content becomes visible
    /// 5. Bring to front
    pub fn move_to(&mut self, target_rect: &RECT) {
        // Step 1: alpha=0 — fully invisible but D2D still functional
        window::set_fully_transparent(self.hwnd);

        // Step 2: Calculate new overlay rect and reposition
        let t = self.effective_thickness() as i32;
        let overlay_rect = RECT {
            left: target_rect.left - t,
            top: target_rect.top - t,
            right: target_rect.right + t,
            bottom: target_rect.bottom + t,
        };
        window::reposition_overlay(self.hwnd, &overlay_rect);

        // Step 3: Recreate render target at new size and render
        self.render_target = None;
        self.create_render_target();

        if self.render(&overlay_rect) {
            // Step 4: Restore colorkey-only (non-magenta pixels become visible)
            window::set_colorkey(self.hwnd);

            // Step 5: Update state and bring to front
            self.last_overlay_rect = overlay_rect;
            window::bring_to_front(self.hwnd);
        } else {
            // Render failed — hide overlay instead of showing black
            window::hide_overlay(self.hwnd);
        }
    }

    fn render(&self, overlay_rect: &RECT) -> bool {
        let Some(rt) = &self.render_target else { return false };

        let w = (overlay_rect.right - overlay_rect.left) as f32;
        let h = (overlay_rect.bottom - overlay_rect.top) as f32;

        unsafe {
            rt.BeginDraw();

            let clear_color = D2D1_COLOR_F { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
            rt.Clear(Some(&clear_color));

            match self.style {
                BorderStyle::Solid => self.render_solid(rt, w, h),
                BorderStyle::Glow => self.render_glow(rt, w, h),
            }

            rt.EndDraw(None, None).is_ok()
        }
    }

    unsafe fn render_solid(&self, rt: &ID2D1HwndRenderTarget, w: f32, h: f32) {
        let t = self.thickness;
        let color = D2D1_COLOR_F {
            r: self.color.r,
            g: self.color.g,
            b: self.color.b,
            a: 1.0,
        };
        let Ok(brush) = rt.CreateSolidColorBrush(&color, None) else { return };

        rt.FillRectangle(&D2D_RECT_F { left: 0.0, top: 0.0, right: w, bottom: t }, &brush);
        rt.FillRectangle(&D2D_RECT_F { left: 0.0, top: h - t, right: w, bottom: h }, &brush);
        rt.FillRectangle(&D2D_RECT_F { left: 0.0, top: t, right: t, bottom: h - t }, &brush);
        rt.FillRectangle(&D2D_RECT_F { left: w - t, top: t, right: w, bottom: h - t }, &brush);
    }

    unsafe fn render_glow(&self, rt: &ID2D1HwndRenderTarget, w: f32, h: f32) {
        let layers = glow_colors(&self.color);
        let mut offset: f32 = 0.0;

        for (layer_t, color) in &layers {
            let Ok(brush) = rt.CreateSolidColorBrush(color, None) else { return };

            let inner_top = offset;
            let inner_bottom = offset + layer_t;
            let outer_w = w - offset;
            let outer_h = h - offset;

            // Top
            rt.FillRectangle(
                &D2D_RECT_F { left: offset, top: inner_top, right: outer_w, bottom: inner_bottom },
                &brush,
            );
            // Bottom
            rt.FillRectangle(
                &D2D_RECT_F { left: offset, top: outer_h - layer_t, right: outer_w, bottom: outer_h },
                &brush,
            );
            // Left
            rt.FillRectangle(
                &D2D_RECT_F { left: offset, top: inner_bottom, right: offset + layer_t, bottom: outer_h - layer_t },
                &brush,
            );
            // Right
            rt.FillRectangle(
                &D2D_RECT_F { left: outer_w - layer_t, top: inner_bottom, right: outer_w, bottom: outer_h - layer_t },
                &brush,
            );

            offset += layer_t;
        }
    }

    pub fn hide(&self) {
        window::hide_overlay(self.hwnd);
    }
}
