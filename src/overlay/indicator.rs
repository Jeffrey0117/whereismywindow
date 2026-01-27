use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, D2D1_ROUNDED_RECT, ID2D1Factory,
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_IMMEDIATELY,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_CENTER,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

use crate::overlay::window;

const BADGE_W: u32 = 48;
const BADGE_H: u32 = 36;
const MARGIN: i32 = 12;
const CORNER_RADIUS: f32 = 8.0;

const ACTIVE_COLOR: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.0,
    g: 0.47,
    b: 0.84,
    a: 1.0,
};
const INACTIVE_COLOR: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.35,
    g: 0.35,
    b: 0.35,
    a: 1.0,
};
const TEXT_COLOR: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// One badge per monitor, displayed at bottom-left corner.
pub struct MonitorIndicators {
    badges: Vec<Badge>,
}

struct Badge {
    hwnd: HWND,
    index: usize,
    d2d_factory: ID2D1Factory,
    dwrite_factory: IDWriteFactory,
    is_active: bool,
}

impl MonitorIndicators {
    pub fn new(monitor_rects: &[RECT]) -> Option<Self> {
        let mut badges = Vec::with_capacity(monitor_rects.len());

        for (i, mon_rect) in monitor_rects.iter().enumerate() {
            let class_name = format!("WhereIsMyWindowIndicator{}", i);
            let hwnd = window::create_overlay_window(
                &class_name,
                BADGE_W as i32,
                BADGE_H as i32,
            )?;

            let badge_rect = RECT {
                left: mon_rect.left + MARGIN,
                top: mon_rect.bottom - BADGE_H as i32 - MARGIN,
                right: mon_rect.left + MARGIN + BADGE_W as i32,
                bottom: mon_rect.bottom - MARGIN,
            };
            window::reposition_overlay(hwnd, &badge_rect);

            let d2d_factory: ID2D1Factory = unsafe {
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).ok()?
            };

            let dwrite_factory: IDWriteFactory = unsafe {
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).ok()?
            };

            window::set_colorkey(hwnd);

            let badge = Badge {
                hwnd,
                index: i,
                d2d_factory,
                dwrite_factory,
                is_active: false,
            };
            badge.render();
            window::show_overlay(hwnd);

            badges.push(badge);
        }

        Some(Self { badges })
    }

    /// Update which monitor is active; always re-render and bring to front.
    pub fn set_active(&mut self, active_index: usize) {
        for badge in &mut self.badges {
            badge.is_active = badge.index == active_index;
            badge.render();
        }
        self.bring_to_front();
    }

    /// Bring all badge windows to the top of the TOPMOST z-order.
    pub fn bring_to_front(&self) {
        for badge in &self.badges {
            window::bring_to_front(badge.hwnd);
        }
    }

    pub fn hwnd_list(&self) -> Vec<isize> {
        self.badges.iter().map(|b| b.hwnd.0 as isize).collect()
    }

    pub fn hide_all(&self) {
        for badge in &self.badges {
            window::hide_overlay(badge.hwnd);
        }
    }

    pub fn show_all(&self) {
        for badge in &self.badges {
            window::show_overlay(badge.hwnd);
        }
        self.bring_to_front();
    }
}

impl Badge {
    fn render(&self) {
        unsafe {
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
                    width: BADGE_W,
                    height: BADGE_H,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
            };

            let Ok(rt) = self
                .d2d_factory
                .CreateHwndRenderTarget(&render_props, &hwnd_props)
            else {
                return;
            };

            let bg_color = if self.is_active {
                ACTIVE_COLOR
            } else {
                INACTIVE_COLOR
            };
            let Ok(bg_brush) = rt.CreateSolidColorBrush(&bg_color, None) else {
                return;
            };
            let Ok(text_brush) = rt.CreateSolidColorBrush(&TEXT_COLOR, None) else {
                return;
            };

            let Ok(text_format) = self.dwrite_factory.CreateTextFormat(
                windows::core::w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                18.0,
                windows::core::w!(""),
            ) else {
                return;
            };

            let _ = text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            rt.BeginDraw();

            // Clear to magenta — color-key makes these pixels fully transparent
            let clear = D2D1_COLOR_F {
                r: 1.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            };
            rt.Clear(Some(&clear));

            // Rounded rectangle background
            let rounded_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: BADGE_W as f32,
                    bottom: BADGE_H as f32,
                },
                radiusX: CORNER_RADIUS,
                radiusY: CORNER_RADIUS,
            };
            rt.FillRoundedRectangle(&rounded_rect, &bg_brush);

            // Monitor number text
            let label = format!("{}", self.index + 1);
            let label_wide: Vec<u16> = label.encode_utf16().collect();
            let layout_rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: BADGE_W as f32,
                bottom: BADGE_H as f32,
            };
            rt.DrawText(
                &label_wide,
                &text_format,
                &layout_rect,
                &text_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );

            let _ = rt.EndDraw(None, None);
        }
    }
}
