// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use rdkafka::admin::AdminClient;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use std::time::Duration;

use crate::info;

pub struct KafkaAdmin {}

impl KafkaAdmin {
    pub fn create() {
        let broker = "localhost:9095"; // Ajusta esto a tu broker de Kafka
        let client: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .create()
            .expect("Error al crear AdminClient");

        let metadata = client
            .inner()
            .fetch_metadata(None, Duration::from_secs(2))
            .expect("Error al fetch metadata");

        for topic in metadata.topics() {
            info!("Topic: {}", topic.name());
            for partition in topic.partitions() {
                info!("  Partition: {}", partition.id());
                // Aquí puedes agregar más lógica para obtener detalles sobre cada partición
            }
        }
    }
}
