// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

/// Módulo con elementos comunes a todos los módulos.
///
/// Traits, structs, funciones... cualquier elemento que sea común a varios
/// módulos. Para aquellos que sean comunes a varios módulos dependientes de
/// sqlx, allí están los elementos comunes. Si hay otros a parte de sqlx,
/// vienen aquí.
pub mod fs;
pub mod generator;
pub mod icon_moon;
pub mod internationalization;
mod i18n_clickhouse;
mod i18n_docker;
mod i18n_http;
mod i18n_sqlx;
mod i18n_kafka;
mod i18n_mongo;
mod i18n_redis;
mod i18n_config;
pub mod macros;
pub mod traits;


pub use i18n_clickhouse::I18nClickHouse;
pub use i18n_docker::I18nDocker;
pub use i18n_http::I18nHttp;
pub use i18n_sqlx::I18nSqlx;
pub use i18n_kafka::I18nKafka;
pub use i18n_redis::I18nRedis;
pub use i18n_config::I18nConfig;
pub use i18n_mongo::I18nMongo;
