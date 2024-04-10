// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::error::Error;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};

use super::state::{MongoConnectionDefinition, MongoLocalState};


#[derive(Default)]
pub struct MongoConnection {
    // pub client: Arc<Mutex<Option<Client>>>,
    pub client: Option<Client>,
    // messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
    // Realmente esta definción no me hace falta aquí, pero por si acaso...
    pub conn_definition: MongoConnectionDefinition,
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
    println!("Trying to connect to {uri}");
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

pub fn close_connection(rt: &tokio::runtime::Runtime, local_state: &mut MongoLocalState) {
    // Usar `guard` facilita mucho porque take sobre referencia no puede usarse,
    // y usar is_some y dentro hacer algo genera problemas de prestado de
    // referencia.
    if local_state.conn.client.is_none() {
        return;
    }
    let client = local_state.conn.client.as_ref().unwrap().clone();
    // local_state.current_connection.path = String::default();

    // Bloqueo para asegurar que todo cerrado antes de reconectar. Puedo
    // de todas formas lanzar con `spawn` sin problemas.
    rt.block_on(async move {
        client.shutdown().await;
    });

    local_state.conn.client = None;
}
