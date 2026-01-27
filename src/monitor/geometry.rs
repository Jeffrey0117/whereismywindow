use windows::Win32::Foundation::RECT;

/// Calculate the overlap area between two rectangles.
pub fn overlap_area(a: &RECT, b: &RECT) -> i64 {
    let x_overlap = (a.right.min(b.right) - a.left.max(b.left)).max(0) as i64;
    let y_overlap = (a.bottom.min(b.bottom) - a.top.max(b.top)).max(0) as i64;
    x_overlap * y_overlap
}

/// Find which monitor index has the most overlap with the given rect.
pub fn best_monitor_index(window_rect: &RECT, monitor_rects: &[RECT]) -> usize {
    monitor_rects
        .iter()
        .enumerate()
        .max_by_key(|(_, mr)| overlap_area(window_rect, mr))
        .map(|(i, _)| i)
        .unwrap_or(0)
}
