//! Capa de persistencia: conexión, migraciones y repositorios.

pub mod connection;
pub mod migrations;
pub mod repositories;

pub use connection::{default_db_path, Db};
