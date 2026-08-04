// Network services module
// Contains subscription and network management functionality
//
// Внимание: состав модуля объявлен в `app/mod.rs` встроенным блоком
// `pub mod network { … }` — этот файл Rust не читает вовсе. Новый модуль
// добавляется туда, иначе он просто не попадёт в сборку.

pub mod subscription_service;
