use std::sync::atomic::{AtomicIsize, AtomicBool, Ordering};
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, EVENT_OBJECT_LOCATIONCHANGE,
    WINEVENT_OUTOFCONTEXT, PostMessageW,
};

/// Custom message posted when focus changes.
pub const WM_FOCUS_CHANGED: u32 = 0x0400 + 1; // WM_APP + 1
/// Custom message posted when window location changes.
pub const WM_LOCATION_CHANGED: u32 = 0x0400 + 2; // WM_APP + 2

static FOCUS_CHANGED: AtomicBool = AtomicBool::new(false);
static LOCATION_CHANGED: AtomicBool = AtomicBool::new(false);

/// The HWND of the main message-only window, stored atomically.
static MSG_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn set_msg_hwnd(hwnd: HWND) {
    MSG_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
}

pub fn msg_hwnd_value() -> isize {
    MSG_HWND.load(Ordering::SeqCst)
}

/// Install SetWinEventHook for EVENT_SYSTEM_FOREGROUND and EVENT_OBJECT_LOCATIONCHANGE.
/// Returns hook handles that must be unhooked on exit.
pub fn install_hooks() -> (HWINEVENTHOOK, HWINEVENTHOOK) {
    unsafe {
        let focus_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(focus_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        let location_hook = SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_LOCATIONCHANGE,
            None,
            Some(location_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        (focus_hook, location_hook)
    }
}

pub fn unhook(focus_hook: HWINEVENTHOOK, location_hook: HWINEVENTHOOK) {
    unsafe {
        let _ = UnhookWinEvent(focus_hook);
        let _ = UnhookWinEvent(location_hook);
    }
}

unsafe extern "system" fn focus_event_callback(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    FOCUS_CHANGED.store(true, Ordering::SeqCst);
    let raw = MSG_HWND.load(Ordering::SeqCst);
    let msg_hwnd = HWND(raw as *mut _);
    if !msg_hwnd.0.is_null() {
        let _ = PostMessageW(Some(msg_hwnd), WM_FOCUS_CHANGED, WPARAM(0), LPARAM(0));
    }
}

unsafe extern "system" fn location_event_callback(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    LOCATION_CHANGED.store(true, Ordering::SeqCst);
    let raw = MSG_HWND.load(Ordering::SeqCst);
    let msg_hwnd = HWND(raw as *mut _);
    if !msg_hwnd.0.is_null() {
        let _ = PostMessageW(Some(msg_hwnd), WM_LOCATION_CHANGED, WPARAM(0), LPARAM(0));
    }
}
