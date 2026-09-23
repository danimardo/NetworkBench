use networkbench_lib::model::{InstanceId, MetricValue, ThroughputBps};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct CommonFixtures {
    valid_instances: Vec<String>,
    invalid_instances: Vec<String>,
    valid_u64_decimals: Vec<String>,
    invalid_u64_decimals: Vec<String>,
    metric_values: MetricValuesFixture,
}

#[derive(Debug, Deserialize)]
struct MetricValuesFixture {
    available: MetricValue<ThroughputBps>,
    not_available: MetricValue<ThroughputBps>,
    not_evaluable: MetricValue<ThroughputBps>,
    invalid: MetricValue<ThroughputBps>,
}

#[test]
fn test_common_fixtures_round_trip() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .parent()
        .expect("parent dir")
        .join("tests/contracts/fixtures/common-fixtures.json");

    let fixture_data = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file at {:?}", fixture_path));

    let fixtures: CommonFixtures = serde_json::from_str(&fixture_data)
        .expect("Failed to parse fixture JSON into CommonFixtures");

    // 1. Validar round-trip de instancias válidas
    for id_str in &fixtures.valid_instances {
        let instance = InstanceId::new(id_str);
        let serialized = serde_json::to_string(&instance).unwrap();
        let deserialized: InstanceId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(instance, deserialized);
        assert_eq!(instance.as_str(), id_str);
    }

    // 2. Validar rechazo de UUIDs inválidos o no v4 RFC4122 en Rust
    for inv_id in &fixtures.invalid_instances {
        let is_valid_v4 = match uuid::Uuid::parse_str(inv_id) {
            Ok(u) => u.get_version_num() == 4 && u.get_variant() == uuid::Variant::RFC4122,
            Err(_) => false,
        };
        assert!(!is_valid_v4, "ID {} no debería ser un UUIDv4 válido", inv_id);
    }

    // 3. Validar u64 serializado como cadena decimal sin pérdida de precisión
    for val_str in &fixtures.valid_u64_decimals {
        let val: u64 = val_str.parse().expect("valid u64 string");
        let bps = ThroughputBps::from_bps(val);
        let serialized = serde_json::to_string(&bps).unwrap();
        assert_eq!(serialized, format!("\"{}\"", val_str));

        let deserialized: ThroughputBps = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.as_u64(), val);
    }

    // 4. Validar rechazo de u64 inválidos o fuera de rango
    for inv_u64 in &fixtures.invalid_u64_decimals {
        let parsed: Result<u64, _> = inv_u64.parse();
        assert!(parsed.is_err());
    }

    // 5. Validar estados discriminados MetricValue
    assert!(fixtures.metric_values.available.is_available());
    assert_eq!(
        fixtures.metric_values.available.value().map(|b| b.as_u64()),
        Some(10000000000)
    );

    assert_eq!(
        fixtures.metric_values.not_available,
        MetricValue::not_available()
    );

    assert_eq!(
        fixtures.metric_values.not_evaluable,
        MetricValue::not_evaluable("Muestras insuficientes durante la fase de medición")
    );

    assert_eq!(
        fixtures.metric_values.invalid,
        MetricValue::invalid("Timestamp fuera de secuencia")
    );
}
