// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use bson::{doc, Document};
use futures::TryStreamExt as _;
use mongodb::error::Result as MongoResult;
use mongodb::{options::FindOptions, Client};
use serde_json::Value;
use std::{collections::HashSet, sync::Arc};
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use crate::{error, info};

use super::{
    actions::MongoAction,
    state::{MongoError, MongoMessage},
};

pub async fn list_database_names_in_connection(
    tx: &Sender<MongoMessage>,
    client: &Client,
) -> Vec<String> {
    let ls = match client.list_database_names(None, None).await {
        Ok(ls) => ls,
        _ => vec![],
    };
    let _ = tx.send(MongoMessage::Databases(ls.to_owned())).await;

    ls
}

pub async fn list_database_collections<'a>(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
) -> Vec<String> {
    let db = client.database(db_name);
    let ls = match db.list_collection_names(None).await {
        Ok(cs) => cs,
        _ => vec![],
    };
    let _ = tx.send(MongoMessage::Collections(ls.to_owned())).await;

    ls
}

/**
 * Debido a nuestro uso, no podemos parametrizar más que con `Document`.
 * A futuro podemos ver cómo dejar que el usuario defina un struct y ver
 * si podemos parsearlo e incluirlo en la consulta, pero vamos, tampoco
 * es importante, por si nos aburrimos.
 */
pub async fn list_collection_documents(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
) -> mongodb::error::Result<(Vec<Document>, Vec<Value>)> {
    find(tx, client, db_name, col_name, doc! {}, MongoAction::Find).await
}

pub async fn find(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
    filter: Document,
    action: MongoAction,
) -> mongodb::error::Result<(Vec<Document>, Vec<Value>)> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);
    let opts = FindOptions::builder()
        // .sort(doc! { "unit_price": -1 })
        .batch_size(50)
        .build();

    let mut docs: Vec<Document> = vec![];

    if action == MongoAction::FindOne {
        if let Some(d) = collection.find_one(filter, None).await? {
            docs.push(d);
        }
    } else {
        let mut cursor = collection.find(filter, opts).await?;
        // Acumulo en un vector para no tener que enviar tantos mensajes.
        // Opción si queremos predefinir el tamaño del vector.
        //     let count = collection.count_documents(doc! {}, None).await?;
        //     let mut docs: Vec<Document> = Vec::with_capacity(count as usize);
        while let Some(result) = cursor.try_next().await.unwrap_or(None) {
            docs.push(result.to_owned());
        }
    };

    let jsons: Vec<Value> = docs.iter().map(|doc| serde_json::json!(doc)).collect();
    let first_level_keys = docs
        .iter()
        .flat_map(|doc| doc.keys())
        .map(|s| s.to_owned())
        .collect::<HashSet<String>>();

    let _ = tx
        .send(MongoMessage::Documents((docs.to_owned(), jsons.to_owned())))
        .await;
    let _ = tx
        .send(MongoMessage::FirstLevelCollectionKeys(first_level_keys))
        .await;

    info!(
        "find many {:?} in db [{}], col [{}]",
        docs, db_name, col_name
    );

    Ok((docs, jsons))
}

pub async fn insert(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
    docs: Vec<Document>,
    action: MongoAction,
) -> MongoResult<()> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);
    let mut msg = MongoMessage::InsertionSuccess;

    if action == MongoAction::InsertOne {
        if docs.len() == 1 {
            let _ = collection.insert_one(&docs[0], None).await?;
        } else {
            msg = MongoMessage::Error("Insert One solo acepta un único elemento".into());
        }
    } else {
        let _ = collection.insert_many(docs, None).await?;
    };

    let _ = tx.send(msg).await;

    Ok(())
}

pub struct MongoPresenter {
    pub rt: Arc<Runtime>,
    pub client: Option<Client>,
}

impl MongoPresenter {
    pub fn new(rt: Arc<Runtime>, client: Client) -> Self {
        Self {
            rt,
            client: Some(client),
        }
    }

    pub fn get_db_stats(&self, db_name: &str) -> Result<(), MongoError> {
        if self.client.is_none() {
            error!("Client not initialized");
            return Err(MongoError::ClientNotInitialized);
        }

        let db = self.client.as_ref().unwrap().database(db_name);

        self.rt.block_on(async {
            let stats = db.run_command(doc! {"dbStats": 1, "scale": 1}, None).await;
            if let Err(err) = stats {
                let msg = format!("{:?}", err);
                error!("{msg}");
                return Err(MongoError::CommandError(msg));
            }
            info!("{:?}", stats);
            Ok(())
        })
    }
}
