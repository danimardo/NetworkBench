use serde::{Deserialize, Serialize};

pub const MIN_WIDTH: u32 = 800;
pub const MIN_HEIGHT: u32 = 600;
pub const DEFAULT_WIDTH: u32 = 1024;
pub const DEFAULT_HEIGHT: u32 = 720;
pub const MIN_VISIBLE_PIXELS: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            is_maximized: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Comprueba si la ventana intersecta al menos con 100×100 px visibles en alguno de los monitores activos
pub fn is_geometry_visible_on_monitors(geom: &WindowGeometry, monitors: &[MonitorBounds]) -> bool {
    if monitors.is_empty() {
        return false;
    }

    let win_w = geom.width.max(MIN_WIDTH) as i32;
    let win_h = geom.height.max(MIN_HEIGHT) as i32;

    for m in monitors {
        let inter_left = geom.x.max(m.x);
        let inter_top = geom.y.max(m.y);
        let inter_right = (geom.x + win_w).min(m.x + m.width as i32);
        let inter_bottom = (geom.y + win_h).min(m.y + m.height as i32);

        let inter_w = inter_right - inter_left;
        let inter_h = inter_bottom - inter_top;

        if inter_w >= MIN_VISIBLE_PIXELS && inter_h >= MIN_VISIBLE_PIXELS {
            return true;
        }
    }

    false
}

/// Normaliza las dimensiones respetando mínimos (800×600) y si la geometría no es visible en
/// al menos 100×100 px en los monitores actuales, la centra en el monitor principal.
pub fn normalize_or_fallback_geometry(
    saved: Option<WindowGeometry>,
    primary: Option<MonitorBounds>,
    all_monitors: &[MonitorBounds],
) -> WindowGeometry {
    if let Some(geom) = saved {
        let width = geom.width.max(MIN_WIDTH);
        let height = geom.height.max(MIN_HEIGHT);
        let normalized = WindowGeometry {
            x: geom.x,
            y: geom.y,
            width,
            height,
            is_maximized: geom.is_maximized,
        };

        if is_geometry_visible_on_monitors(&normalized, all_monitors) {
            return normalized;
        }
    }

    // Fallback: centrar en el monitor principal o por defecto
    if let Some(m) = primary {
        let w = DEFAULT_WIDTH.min(m.width.max(MIN_WIDTH));
        let h = DEFAULT_HEIGHT.min(m.height.max(MIN_HEIGHT));
        let x = m.x + ((m.width as i32 - w as i32) / 2).max(0);
        let y = m.y + ((m.height as i32 - h as i32) / 2).max(0);
        WindowGeometry {
            x,
            y,
            width: w,
            height: h,
            is_maximized: false,
        }
    } else {
        WindowGeometry::default()
    }
}
