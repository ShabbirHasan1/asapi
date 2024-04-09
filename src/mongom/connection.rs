// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use std::error::Error;

use crate::info;

use super::state::MongoConnectionDefinition;

pub struct MongoPresenter {}

impl Default for MongoPresenter {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Default)]
pub struct MongoConnection {
    // pub client: Arc<Mutex<Option<Client>>>,
    pub client: Option<Client>,
    // messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
    // Realmente esta definción no me hace falta aquí, pero por si acaso...
    pub conn_definition: MongoConnectionDefinition,
}

impl MongoConnection {
    pub async fn connect(&mut self, conn_definition: MongoConnectionDefinition) {
        self.conn_definition = conn_definition;

        if let Some(client) = self.client.take() {
            client.shutdown().await;
        }

        match connect_with_default(&self.conn_definition).await {
            Ok(con) => {
                self.client = Some(con);
            }
            Err(err) => {
                info!("{:?}", err);
                self.client = None;
            }
        }
    }

    pub async fn shutdown(&mut self) {
        if let Some(client) = self.client.take() {
            // client.shutdown_immediate().await;
            // client.shutdown().await;
            client.shutdown().await;
        }
    }
}

pub async fn connect(
    host: &str,
    port: i16,
    user: &str,
    password: &str,
    is_srv: bool,
) -> Result<Client, Box<dyn Error>> {
    let protocol = if is_srv { "mongodb+srv" } else { "mongodb" };
    let uri = format!("{protocol}://{user}:{password}@{host}:{port}/?retryWrites=true&w=majority");
    info!("Trying to connect to {uri}");
    let options =
        ClientOptions::parse_with_resolver_config(&uri, ResolverConfig::cloudflare()).await?;
    let client = Client::with_options(options)?;

    Ok(client)
}

pub async fn connect_with_default(
    conn_definition: &MongoConnectionDefinition,
) -> Result<Client, Box<dyn Error>> {
    let port = conn_definition.port.parse::<i16>().unwrap_or(27172);
    connect(
        &conn_definition.host,
        port,
        &conn_definition.user,
        &conn_definition.password,
        conn_definition.is_srv,
    )
    .await
}
