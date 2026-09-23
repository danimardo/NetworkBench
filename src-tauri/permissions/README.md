# Permisos de NetworkBench

NetworkBench sigue el principio constitucional de mínimo privilegio (ADR-005, Constitución III):
- No se exponen comandos genéricos de shell, SQL ni sistema de archivos a la WebView.
- Los comandos específicos de NetworkBench se autorizan explícitamente y validan parámetros con Serde.
- Las capacidades de la ventana principal residen en `src-tauri/capabilities/main.json`.
