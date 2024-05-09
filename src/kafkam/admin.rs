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



pub async fn create_topic(
    broker_url: &str,
    name: &str,
    num_partitions: i32,
    replication: i32,
    config: Vec<(String, String)>,
) -> Result<String, (String, RDKafkaErrorCode)> {
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

    log::info!("{result:?}");
    match result {
        Ok(v) => {
            // Sólo creamos un topic luego solo necesitamos `first`.
            match v.first() {
                Some(fst) => fst.to_owned(),
                None => Err(("Topic no se creó".to_string(), RDKafkaErrorCode::Fail)),
            }
        }
        Err(err) => {
            log::error!("{err:?}");
            Err(("Topic no se creó".to_string(), RDKafkaErrorCode::Fail))
        }
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

// TODO: No en uso, no sé muy bien qué hacer con ella
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

// TODO: No en uso, posiblmente nunca lo acabe usando.
pub async fn create_partition(
    broker_url: &str,
    name: &str,
    partition_count: usize,
) -> Result<Result<String, (String, RDKafkaErrorCode)>, KafkaError> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", broker_url)
        .create()
        .expect("Error al crear AdminClient");
    let opts = AdminOptions::default().request_timeout(Some(Duration::from_secs(2)));
    let new_partition = NewPartitions {
        topic_name: name,
        new_partition_count: partition_count,
        assignment: None,
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
