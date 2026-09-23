pub mod args;
pub mod job_object;
pub mod parser;
pub mod process;

pub use args::*;
pub use job_object::*;
pub use parser::*;
pub use process::*;

/// Versión del motor empaquetado, leída de `engine/VERSION`.
pub const ENGINE_VERSION: &str = include_str!("../../../../engine/VERSION");

/// SHA-256 esperado del ejecutable, leído de `engine/SHA256`.
///
/// Fuente única: el mismo fichero lo lee el paso de integridad de CI. Duplicar este
/// valor en el workflow permitiría que uno de los dos quedara obsoleto sin que nada
/// avisara. Se comprueba antes de cada ejecución (FR-063).
pub const ENGINE_SHA256: &str = include_str!("../../../../engine/SHA256");

/// Ambos ficheros terminan en salto de línea; los consumidores necesitan el valor limpio.
pub fn engine_version() -> &'static str {
    ENGINE_VERSION.trim()
}

pub fn engine_sha256() -> &'static str {
    ENGINE_SHA256.trim()
}

#[cfg(test)]
mod ntttcp_tests;
