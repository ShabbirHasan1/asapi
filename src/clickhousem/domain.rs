// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use crate::common::traits::ToUrl;

#[derive(serde::Serialize, Debug, serde::Deserialize, Default, Clone)]
pub struct ClickHouseConnectionOptions {
    pub schema: String,
    pub compression: String,
    pub readonly: u8,
    pub connection_timeout: u16,
    pub keapalive: u16,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default, PartialEq)]
pub enum ClickHouseProtocol {
    #[default]
    Tcp,
    // clickhouse-rs no soport http ni https
    Http,
    Https,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct ClickHouseConnectionDefinition {
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub protocol: ClickHouseProtocol,
    // pub dbname: String,
    pub options: ClickHouseConnectionOptions,
}

impl ToUrl for ClickHouseConnectionDefinition {
    fn to_url(&self) -> String {
        let p = match self.protocol {
            ClickHouseProtocol::Tcp => "tcp",
            ClickHouseProtocol::Http => "http",
            ClickHouseProtocol::Https => "https"
        };
        format!("{p}://{}:{}", self.host, self.port)
    }
}

#[derive(Debug)]
pub enum ClickHouseMessage {
    InsertStatement(String), // stmt
    DeleteStatement((String, usize)),
    DeleteAllStmt(String), // table name
    // Datos -- Definición columnas -- Mostrar todas las columnas
    SelectResponse((Vec<Vec<String>>, Vec<(String, String)>, bool)),
    Error(String),
    Empty, // para errores, pero para poder resetear (o cualquier otra cosa que necesitemos).
    AddConnection(ClickHouseConnectionDefinition),
    EditConnection((usize, ClickHouseConnectionDefinition)),
    DatabaseTables(Vec<String>)
}
