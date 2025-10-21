use crate::wayland::Monitor;
use std::i32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorXY {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub width: u32,
    pub height: u32,
}

impl MonitorXY {
    /// Convert from Monitor to self
    pub fn from_monitor(monitor: &Monitor) -> Self {
        let x1 = monitor.x.min(monitor.x + monitor.width as i32);
        let x2 = monitor.x.max(monitor.x + monitor.width as i32);
        let y1 = monitor.y.min(monitor.y + monitor.height as i32);
        let y2 = monitor.y.max(monitor.y + monitor.height as i32);
        Self {
            x1,
            y1,
            x2,
            y2,
            width: monitor.width,
            height: monitor.height,
        }
    }
    /// Canonicalize monitor coordinates
    pub fn canonical(mut self) -> Self {
        if self.x1 > self.x2 {
            std::mem::swap(&mut self.x1, &mut self.x2);
        }
        if self.y1 > self.y2 {
            std::mem::swap(&mut self.y1, &mut self.y2);
        }
        self
    }
    /// Apply translation
    fn translate(&mut self, dx: i32, dy: i32) {
        self.x1 += dx;
        self.x2 += dx;
        self.y1 += dy;
        self.y2 += dy;
    }
}

/// Normalize a set of monitors so that all coordinates are positive
pub fn normalize_to_positive(monitors: &mut [MonitorXY]) {
    if monitors.is_empty() {
        return;
    }

    let min_x = monitors.iter().map(|r| r.x1).fold(i32::MAX, i32::min);
    let min_y = monitors.iter().map(|r| r.y1).fold(i32::MAX, i32::min);

    let dx = if min_x < 0 { -min_x } else { 0 };
    let dy = if min_y < 0 { -min_y } else { 0 };

    for r in monitors.iter_mut() {
        r.translate(dx, dy);
    }
}

/// Resolves touching and overlapping monitors
pub fn resolve_layout(monitors: &mut [MonitorXY], padding: u32, max_iterations: usize) {
    for _ in 0..max_iterations {
        let mut changed = false;

        for i in 0..monitors.len() {
            for j in (i + 1)..monitors.len() {
                let (mut a, mut b) = (monitors[i].canonical(), monitors[j].canonical());

                let (x_overlap, y_overlap);
                if padding > 0 {
                    x_overlap = a.x2 >= b.x1 && a.x1 <= b.x2;
                    y_overlap = a.y2 >= b.y1 && a.y1 <= b.y2;
                } else {
                    x_overlap = a.x2 > b.x1 && a.x1 < b.x2;
                    y_overlap = a.y2 > b.y1 && a.y1 < b.y2;
                }

                if x_overlap && y_overlap {
                    // Compute overlap depth
                    let overlap_x = (a.x2.min(b.x2) - a.x1.max(b.x1)).max(0);
                    let overlap_y = (a.y2.min(b.y2) - a.y1.max(b.y1)).max(0);

                    // Determine smallest axis of overlap
                    if overlap_x < overlap_y {
                        let dir = if a.x1 < b.x1 { -1 } else { 1 };
                        let move_dist = (overlap_x + padding as i32) / 2;
                        a.translate(dir * move_dist, 0);
                        b.translate(-dir * move_dist, 0);
                    } else {
                        let dir = if a.y1 < b.y1 { -1 } else { 1 };
                        let move_dist = (overlap_y + padding as i32) / 2;
                        a.translate(0, dir * move_dist);
                        b.translate(0, -dir * move_dist);
                    }

                    monitors[i] = a;
                    monitors[j] = b;
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }
}
