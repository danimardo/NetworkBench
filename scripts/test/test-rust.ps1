# Script canónico para ejecutar pruebas y validación Rust
$ErrorActionPreference = "Stop"
Write-Host "Ejecutando pruebas de unidad e integración Rust..." -ForegroundColor Cyan
cargo test --manifest-path src-tauri/Cargo.toml
