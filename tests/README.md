# Tests

- **Tests unitarios de Rust**: viven junto al código, en módulos `#[cfg(test)]`
  dentro de `src-tauri/src/`.
- **Tests de integración de Rust**: `src-tauri/tests/`, que es donde Cargo los
  busca. Ahora mismo: `scan_flow.rs`, que ejercita registrar ruta -> escanear ->
  listar proyectos.
- **Este directorio** queda reservado para los tests end-to-end del frontend y
  para los repositorios Git de prueba (`fixtures/`) que necesitará la Fase 2.
