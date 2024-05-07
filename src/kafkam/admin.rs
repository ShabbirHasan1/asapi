// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use rdkafka::admin::{AdminClient, AdminOptions, NewPartitions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::types::RDKafkaErrorCode;
use std::time::Duration;

use super::presenter;

// pub struct KafkaAdmin {}

// impl KafkaAdmin {
//     pub fn create() {
//         let broker = "localhost:9095"; // Ajusta esto a tu broker de Kafka
//         let client: AdminClient<DefaultClientContext> = ClientConfig::new()
//             .set("bootstrap.servers", broker)
//             .create()
//             .expect("Error al crear AdminClient");

//         let metadata = client
//             .inner()
//             .fetch_metadata(None, Duration::from_secs(2))
//             .expect("Error al fetch metadata");

//         for topic in metadata.topics() {
//             println!("Topic: {}", topic.name());
//             for partition in topic.partitions() {
//                 println!("  Partition: {}", partition.id());
//                 // Aquí puedes agregar más lógica para obtener detalles sobre cada partición
//             }
//         }
//     }
// }

pub async fn create_topic(
    broker_url: &str,
    name: &str,
    num_partitions: i32,
    replication: i32,
    config: Vec<(String, String)>,
) -> Result<Result<String, (String, RDKafkaErrorCode)>, KafkaError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .create()
        .expect("Error al crear AdminClient");
    let opts = AdminOptions::default().request_timeout(Some(Duration::from_secs(2)));
    let c = config
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect::<Vec<(&str, &str)>>();
    let topic = NewTopic {
        name,
        num_partitions,
        replication: TopicReplication::Fixed(replication),
        config: c,
    };
    let result = admin.create_topics([&topic], &opts).await;

    match result {
        Ok(v) => {
            // Sólo creamos un topic.
            if v.is_empty() {
                Err(KafkaError::AdminOp(RDKafkaErrorCode::Fail))
            } else {
                Ok(v.first().unwrap().to_owned())
            }
        }
        Err(err) => {
            log::error!("{err:?}");
            Err(err)
        },
    }
}

pub async fn delete_topic(
    broker_url: &str,
    name: &str,
) -> Result<Result<String, (String, RDKafkaErrorCode)>, KafkaError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .create()
        .expect("Error al crear AdminClient");
    let opts = AdminOptions::default().request_timeout(Some(Duration::from_secs(2)));
    let result = admin.delete_topics(&[name], &opts).await;

    match result {
        Ok(v) => {
            if v.is_empty() {
                Err(KafkaError::AdminOp(RDKafkaErrorCode::Fail))
            } else {
                Ok(v.first().unwrap().to_owned())
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn delete_groups(
    broker_url: &str,
    group: &str,
) -> Result<Result<String, (String, RDKafkaErrorCode)>, KafkaError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .create()
        .expect("Error al crear AdminClient");
    let opts = AdminOptions::default().request_timeout(Some(Duration::from_secs(2)));
    let result = admin.delete_groups(&[group], &opts).await;

    match result {
        Ok(v) => {
            if v.is_empty() {
                Err(KafkaError::AdminOp(RDKafkaErrorCode::Fail))
            } else {
                Ok(v.first().unwrap().to_owned())
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn create_partition(
    broker_url: &str,
    name: &str,
    partition_count: usize
) -> Result<Result<String, (String, RDKafkaErrorCode)>, KafkaError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .create()
        .expect("Error al crear AdminClient");
    let opts = AdminOptions::default().request_timeout(Some(Duration::from_secs(2)));
    let new_partition = NewPartitions {
        topic_name: name,
        new_partition_count: partition_count,
        assignment: None
    };
    let result = admin.create_partitions(&[new_partition], &opts).await;

    match result {
        Ok(v) => {
            if v.is_empty() {
                Err(KafkaError::AdminOp(RDKafkaErrorCode::Fail))
            } else {
                Ok(v.first().unwrap().to_owned())
            }
        }
        Err(err) => Err(err),
    }
}
