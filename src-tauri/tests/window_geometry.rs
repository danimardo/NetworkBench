use networkbench_lib::platform::window::{
    MIN_HEIGHT, MIN_WIDTH, MonitorBounds, WindowGeometry, is_geometry_visible_on_monitors,
    normalize_or_fallback_geometry,
};

#[test]
fn test_window_geometry_100x100_visible_rule() {
    let monitor = MonitorBounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    let monitors = vec![monitor];

    // Caso 1: Ventana completamente visible
    let geom_inside = WindowGeometry {
        x: 100,
        y: 100,
        width: 1024,
        height: 720,
        is_maximized: false,
    };
    assert!(is_geometry_visible_on_monitors(&geom_inside, &monitors));

    // Caso 2: Ventana parcialmente fuera pero con exactamente 100x100 px dentro
    let geom_edge_100 = WindowGeometry {
        x: 1920 - 100,
        y: 1080 - 100,
        width: 800,
        height: 600,
        is_maximized: false,
    };
    assert!(is_geometry_visible_on_monitors(&geom_edge_100, &monitors));

    // Caso 3: Ventana con solo 99 px de ancho visibles (menos de 100 px)
    let geom_too_far_x = WindowGeometry {
        x: 1920 - 99,
        y: 100,
        width: 800,
        height: 600,
        is_maximized: false,
    };
    assert!(!is_geometry_visible_on_monitors(&geom_too_far_x, &monitors));

    // Caso 4: Ventana con solo 99 px de alto visibles (menos de 100 px)
    let geom_too_far_y = WindowGeometry {
        x: 100,
        y: 1080 - 99,
        width: 800,
        height: 600,
        is_maximized: false,
    };
    assert!(!is_geometry_visible_on_monitors(&geom_too_far_y, &monitors));

    // Caso 5: Ventana completamente fuera de pantalla (ej. monitor secundario desconectado)
    let geom_disconnected = WindowGeometry {
        x: 3000,
        y: 2000,
        width: 1024,
        height: 720,
        is_maximized: false,
    };
    assert!(!is_geometry_visible_on_monitors(
        &geom_disconnected,
        &monitors
    ));
}

#[test]
fn test_window_geometry_fallback_and_centering() {
    let primary = MonitorBounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    let monitors = vec![primary];

    // Geometría perdida/fuera de pantalla: debe centrar en el monitor principal
    let geom_out = WindowGeometry {
        x: 5000,
        y: 5000,
        width: 1024,
        height: 720,
        is_maximized: false,
    };

    let fallback = normalize_or_fallback_geometry(Some(geom_out), Some(primary), &monitors);
    assert_eq!(fallback.width, 1024);
    assert_eq!(fallback.height, 720);
    // (1920 - 1024) / 2 = 448
    assert_eq!(fallback.x, 448);
    // (1080 - 720) / 2 = 180
    assert_eq!(fallback.y, 180);

    // Geometría válida pero con tamaño inferior al mínimo: debe elevarse a mín 800x600
    let geom_small = WindowGeometry {
        x: 50,
        y: 50,
        width: 400,
        height: 300,
        is_maximized: false,
    };
    let normalized = normalize_or_fallback_geometry(Some(geom_small), Some(primary), &monitors);
    assert_eq!(normalized.width, MIN_WIDTH);
    assert_eq!(normalized.height, MIN_HEIGHT);
    assert_eq!(normalized.x, 50);
    assert_eq!(normalized.y, 50);
}
