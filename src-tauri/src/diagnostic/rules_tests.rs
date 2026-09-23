use super::capacity::{CapacityReference, CapacitySource};
use super::rules::*;

#[test]
fn test_stability_insufficient_samples() {
    let engine = DiagnosticEngine::new();
    let samples = vec![100.0, 105.0, 95.0]; // < 10 muestras
    let stats = engine.evaluate_stability(&samples, 0);
    assert_eq!(stats.level, StabilityLevel::NotEvaluable);
    assert_eq!(stats.samples_count, 3);
}

#[test]
fn test_stability_very_stable() {
    let engine = DiagnosticEngine::new();
    // 10 muestras casi idénticas -> cv < 0.05
    let samples = vec![1000.0; 12];
    let stats = engine.evaluate_stability(&samples, 0);
    assert_eq!(stats.level, StabilityLevel::VeryStable);
    assert_eq!(stats.drops_count, 0);
    assert_eq!(stats.cv, 0.0);
}

#[test]
fn test_stability_stable() {
    let engine = DiagnosticEngine::new();
    // cv entre 0.05 y 0.10 (std dev ~ 63, mean 1000 -> cv ~ 0.063)
    let samples = vec![940.0, 1060.0, 940.0, 1060.0, 940.0, 1060.0, 940.0, 1060.0, 940.0, 1060.0];
    let stats = engine.evaluate_stability(&samples, 0);
    assert!(stats.cv >= 0.05 && stats.cv < 0.10);
    assert_eq!(stats.level, StabilityLevel::Stable);
}

#[test]
fn test_stability_drops_penalty() {
    let engine = DiagnosticEngine::new();
    // 12 muestras, 2 caídas por debajo del 50% de la media (< 500)
    let mut samples = vec![1000.0; 12];
    samples[2] = 400.0; // caída 1
    samples[6] = 450.0; // caída 2
    let stats = engine.evaluate_stability(&samples, 0);
    assert_eq!(stats.drops_count, 2);
    // Debe haber bajado al menos a Variable o VeryVariable
    assert!(matches!(stats.level, StabilityLevel::Variable | StabilityLevel::VeryVariable));
}

#[test]
fn test_asymmetry_boundaries() {
    let engine = DiagnosticEngine::new();
    
    // Exactly 20% difference: (1000 - 800) / 1000 = 0.20 -> NOT asymmetric (strictly > 0.20)
    let asym_20 = engine.evaluate_asymmetry(Some(1000), Some(800)).unwrap();
    assert!(!asym_20.is_asymmetric);
    assert!((asym_20.ratio - 0.20).abs() < 1e-6);

    // 25% difference: (1000 - 750) / 1000 = 0.25 -> ASYMMETRIC
    let asym_25 = engine.evaluate_asymmetry(Some(1000), Some(750)).unwrap();
    assert!(asym_25.is_asymmetric);
    assert!((asym_25.ratio - 0.25).abs() < 1e-6);

    // One direction missing -> None
    assert!(engine.evaluate_asymmetry(Some(1000), None).is_none());
    assert!(engine.evaluate_asymmetry(None, Some(1000)).is_none());
}

#[test]
fn test_retransmissions_boundaries() {
    let engine = DiagnosticEngine::new();

    // Ratio < 0.001 -> Normal (e.g. 5 / 10000 = 0.0005)
    let normal = engine.evaluate_retransmissions(Some(10000), Some(5));
    assert_eq!(normal.level, RetransmissionLevel::Normal);

    // Exactly 0.001 -> Elevated (0.001 .. 0.01)
    let elevated_boundary = engine.evaluate_retransmissions(Some(10000), Some(10));
    assert_eq!(elevated_boundary.level, RetransmissionLevel::Elevated);

    // Exactly 0.01 -> Elevated
    let elevated_upper = engine.evaluate_retransmissions(Some(10000), Some(100));
    assert_eq!(elevated_upper.level, RetransmissionLevel::Elevated);

    // > 0.01 -> High
    let high = engine.evaluate_retransmissions(Some(10000), Some(150));
    assert_eq!(high.level, RetransmissionLevel::High);

    // Missing -> NotAvailable
    let missing = engine.evaluate_retransmissions(None, None);
    assert_eq!(missing.level, RetransmissionLevel::NotAvailable);
}

#[test]
fn test_cpu_boundaries() {
    let engine = DiagnosticEngine::new();
    assert_eq!(engine.evaluate_cpu(Some(20.0)), CpuLevel::Low);
    assert_eq!(engine.evaluate_cpu(Some(25.0)), CpuLevel::Moderate);
    assert_eq!(engine.evaluate_cpu(Some(60.0)), CpuLevel::Moderate);
    assert_eq!(engine.evaluate_cpu(Some(60.1)), CpuLevel::High);
    assert_eq!(engine.evaluate_cpu(None), CpuLevel::NotAvailable);
}

#[test]
fn test_verdict_generation_ok_path() {
    let engine = DiagnosticEngine::new();
    let cap = CapacityReference {
        ref_bps: Some(1_000_000_000),
        ref_source: CapacitySource::Negotiated,
        cap_a_bps: Some(1_000_000_000),
        cap_b_bps: Some(1_000_000_000),
    };
    let stability = StabilityStats {
        level: StabilityLevel::VeryStable,
        mean_bps: 950_000_000.0,
        std_dev_bps: 10_000.0,
        cv: 0.01,
        min_bps: 940_000_000.0,
        max_bps: 960_000_000.0,
        p5_bps: 945_000_000.0,
        p50_bps: 950_000_000.0,
        p95_bps: 955_000_000.0,
        drops_count: 0,
        samples_count: 20,
        gaps_count: 0,
    };
    let asym = AsymmetryStats {
        ratio: 0.02,
        is_asymmetric: false,
    };
    let retrans = RetransmissionStats {
        level: RetransmissionLevel::Normal,
        ratio: Some(0.0001),
        packets_sent: Some(50000),
        packets_retransmitted: Some(5),
    };

    let verdict = engine.generate_verdict(
        &cap,
        Some(950_000_000),
        Some(940_000_000),
        Some(&stability),
        Some(&stability),
        Some(&asym),
        Some(&retrans),
        Some(15.0),
        true,
    );

    assert_eq!(verdict.level, VerdictLevel::Ok);
    assert_eq!(verdict.performance_level, Some(VerdictLevel::Ok));
    assert!(verdict.observations.is_empty());
}

#[test]
fn test_verdict_without_ref_bps_has_no_performance_verdict() {
    let engine = DiagnosticEngine::new();
    let cap = CapacityReference {
        ref_bps: None,
        ref_source: CapacitySource::Wifi,
        cap_a_bps: None,
        cap_b_bps: None,
    };
    let stability = StabilityStats {
        level: StabilityLevel::Stable,
        mean_bps: 450_000_000.0,
        std_dev_bps: 30_000_000.0,
        cv: 0.06,
        min_bps: 400_000_000.0,
        max_bps: 500_000_000.0,
        p5_bps: 410_000_000.0,
        p50_bps: 450_000_000.0,
        p95_bps: 490_000_000.0,
        drops_count: 0,
        samples_count: 20,
        gaps_count: 0,
    };

    let verdict = engine.generate_verdict(
        &cap,
        Some(450_000_000),
        Some(440_000_000),
        Some(&stability),
        Some(&stability),
        None,
        None,
        Some(10.0),
        true,
    );

    assert_eq!(verdict.performance_level, None);
    assert_eq!(verdict.level, VerdictLevel::Ok);
}
