//! Comandos Tauri: capa fina de adaptación entre el frontend y `core`/`db`.
//!
//! Aquí no vive lógica de negocio: solo validación de entrada, conversión de
//! errores a `String` y traslado del trabajo bloqueante fuera del hilo de la UI.

pub mod git;
pub mod projects;
pub mod scanner;
pub mod settings;
pub mod system;
pub mod tags;
