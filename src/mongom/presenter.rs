// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use bson::{doc, Document};
use futures::TryStreamExt as _;
use log;
use mongodb::error::Result as MongoResult;
use mongodb::{options::FindOptions, Client};
use serde_json::Value;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::common::internationalization::I18n;
use crate::mongom::parser::doc_to_serde_value;

use super::{
    actions::MongoAction,
    state::{MongoError, MongoMessage},
};

pub async fn list_database_names_in_connection(
    tx: &Sender<MongoMessage>,
    client: &Client,
) -> Vec<String> {
    let timeout_duration = Duration::from_secs(5);
    let ls = match tokio::time::timeout(timeout_duration, client.list_database_names(None, None))
        .await
    {
        Ok(Ok(database_names)) => {
            let _ = tx
                .send(MongoMessage::Databases(database_names.to_owned()))
                .await;
            database_names
        }
        Ok(Err(e)) => {
            let error_message = format!("Error al listar las bases de datos: {}", e);
            log::error!("{error_message}");
            let _ = tx.send(MongoMessage::Error(error_message)).await;
            vec![]
        }
        Err(e) => {
            let error_message = format!(
                "La operación de listar bases de datos excedió el tiempo límite: {}",
                e
            );
            log::error!("{error_message}");
            let _ = tx.send(MongoMessage::Error(error_message)).await;
            vec![]
        }
    };

    ls
}

pub async fn list_database_collections<'a>(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
) -> Vec<String> {
    let db = client.database(db_name);
    let ls = match db.list_collection_names(None).await {
        Ok(cs) => {
            let _ = tx.send(MongoMessage::Collections(cs.to_owned())).await;
            cs
        }
        Err(e) => {
            let error_message = format!(
                "La operación de listar bases de datos excedió el tiempo límite: {}",
                e
            );
            log::error!("{error_message}");
            let _ = tx.send(MongoMessage::Error(error_message)).await;
            vec![]
        }
    };

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

    let jsons: Vec<Value> = docs.iter().map(doc_to_serde_value).collect();

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

    Ok((docs, jsons))
}

pub async fn insert(
    tx: &Sender<MongoMessage>,
    i18n: &I18n,
    client: &Client,
    db_name: &str,
    col_name: &str,
    docs: Vec<Document>,
    action: MongoAction,
) -> MongoResult<()> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);
    let mut msg = MongoMessage::InsertionSuccess;

    // Esta comprobación es redundante si el cliente es solo MongoView.insert
    if action == MongoAction::InsertOne {
        if docs.len() == 1 {
            let _ = collection.insert_one(&docs[0], None).await?;
        } else {
            msg = MongoMessage::Error(i18n.mongo_insert_one_error.clone());
        }
    } else {
        let _ = collection.insert_many(docs, None).await?;
    };

    let _ = tx.send(msg).await;

    Ok(())
}

pub async fn update(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
    filter: Document,
    doc: Document,
    action: MongoAction,
) -> MongoResult<()> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);

    log::info!("filter\n{:?}", filter);

    // Esta comprobación es redundante si el cliente es solo MongoView.insert
    if action == MongoAction::UpdateOne {
        let _ = collection
            .update_one(filter, doc! { "$set": doc }, None)
            .await?;
    } else {
        let _ = collection
            .update_many(filter, doc! { "$set": doc }, None)
            .await?;
    };

    let _ = tx.send(MongoMessage::UpdateSuccess).await;

    Ok(())
}

pub async fn replace(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
    filter: Document,
    doc: &Document,
) -> MongoResult<()> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);
    let msg = MongoMessage::ReplaceSuccess;

    // Esta comprobación es redundante si el cliente es solo MongoView.insert
    let _ = collection.replace_one(filter, doc, None).await?;

    let _ = tx.send(msg).await;

    Ok(())
}

pub async fn delete(
    tx: &Sender<MongoMessage>,
    client: &Client,
    db_name: &str,
    col_name: &str,
    doc: Document,
    action: MongoAction,
) -> MongoResult<()> {
    let db = client.database(db_name);
    let collection = db.collection::<Document>(col_name);
    let msg = MongoMessage::DeleteSuccess;

    // Esta comprobación es redundante si el cliente es solo MongoView.insert
    if action == MongoAction::DeleteOne {
        log::info!("Documento a borrar\n{:?}", doc);
        let _ = collection.delete_one(doc, None).await?;
    } else {
        let _ = collection.delete_many(doc, None).await?;
    }

    let _ = tx.send(msg).await;

    Ok(())
}

pub async fn run_command(
    client: &Client,
    db_name: &str,
    query: Document,
) -> Result<Document, MongoError> {
    let db = client.database(db_name);
    let stats = db.run_command(query, None).await;

    match stats {
        Ok(data) => Ok(data),
        Err(err) => {
            let msg = format!("{:?}", err);
            log::error!("{msg}");
            Err(MongoError::CommandError(msg))
        }
    }
}

pub async fn get_db_stats(client: &Client, db_name: &str) -> Result<Document, MongoError> {
    run_command(client, db_name, doc! {"dbStats": 1, "scale": 1}).await
}

pub async fn get_collection_stats(
    client: &Client,
    db_name: &str,
    col_name: &str,
) -> Result<Document, MongoError> {
    run_command(client, db_name, doc! {"collStats": col_name}).await
}
