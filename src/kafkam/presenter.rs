// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::epaint::stats;
// Para poder conectarme a señales del sistema.
// use signal_hook::consts::signal::*;
// use signal_hook::flag;
use log::info;
use rdkafka::admin::AdminClient;
use rdkafka::client::ClientContext;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::producer::BaseProducer;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::ProducerContext;
use rdkafka::statistics::Statistics;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::Timeout;
use std::cell::Cell;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::common::traits::Create;
use crate::kafkam::state::KafkaConsumerMessage;

// =================================
// Admin
// =================================
// Ahora mismo nada en uso, dejo porque a futuro seguro que necesitamos cuando
// queramos implementar funcionalidades extra.

// =================================
// Productor
// =================================
// Podría (lo tenía de hecho) tenerlo en archivo propio, pero prefiero tenerlo
// todo en los menos archivos mejor y con estructura similar: view|presenter|state
pub struct CustomProducerContext {
    pub stats: Arc<Mutex<Vec<Statistics>>>,
    // para debug
    pub print: AtomicBool
}

impl ClientContext for CustomProducerContext {
    fn stats(&self, statistics: Statistics) {
        // Realmente con Option<> puedo, pero por si acaso en algún momento quiero mantener histórico
        // de estadísticas (no sé para qué, pero puede ser que venga bien, si no todo, sí al menos)
        // ciertas cosas, como consumo de memoria, etc.
        let mut stats = self.stats.lock().unwrap(); // Adquiere el lock una vez aquí
        if stats.is_empty() {
            stats.push(statistics);
        } else {
            stats[0] = statistics;
        }
        if self.print.load(std::sync::atomic::Ordering::SeqCst) {
            // info!("Estadísticas recibidas (primera vez)");
            // info!("====================================");
            // info!("{:?}", stats[0]);

            self.print.store(false, std::sync::atomic::Ordering::SeqCst);
        }
        // info!("#Estadísticas: {}", stats.len());
    }
}

pub struct KafkaProducerPresenter {
    pub client: FutureProducer<CustomProducerContext>,
    pub stats: Arc<Mutex<Vec<Statistics>>>,
}

impl KafkaProducerPresenter {
    pub fn new(brokers: &str) -> Self {
        let stats = Arc::new(Mutex::new(Vec::with_capacity(1)));
        let context = CustomProducerContext {
            stats: stats.clone(),
            print: AtomicBool::new(true)
        };
        let client = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("statistics.interval.ms", "1000")
            .set("api.version.fallback.ms", "0") // por si corregía error con fetch_metadata, no lo ha conseguido
            .create_with_context(context)
            .expect("Producer creation failed");

        Self { client, stats }
    }
}

// =================================
// Consumidor
// =================================
// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomConsumerContext;

impl ClientContext for CustomConsumerContext {
    fn stats(&self, statistics: Statistics) {
        info!("New Stats");
    }
}

impl ConsumerContext for CustomConsumerContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        // println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        // println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        // println!("Committing offsets: {:?}", result);
    }
}

pub struct KafkaConsumer {
    pub consumer: StreamConsumer<CustomConsumerContext>,
}

impl KafkaConsumer {
    pub fn groups(&self, groups: Option<&str>) -> KafkaResult<()> {
        let group_list = self
            .consumer
            .fetch_group_list(groups, Timeout::After(Duration::from_secs(10)))?;

        for group in group_list.groups() {
            println!("Group Name: {}", group.name());
            println!("State: {}", group.state());
            println!("Protocol: {}", group.protocol());
            println!("Protocol Type: {}", group.protocol_type());
            println!("Members: {}", group.members().len());
            for member in group.members() {
                println!("  Member ID: {}", member.id());
                println!("  Client ID: {}", member.client_id());
                println!("  Client Host: {}", member.client_host());
                println!("  Assignment: {:?}", member.assignment());
                println!("  Metadata: {:?}", member.metadata());
            }
        }

        Ok(())
    }

    // https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_consumer.rs
    //   y (aunque viejo y API bastante cambiada)
    // https://github.com/fede1024/kafka-view/blob/master/examples/consumer_offsets_reader.rs
    pub fn create_async_consumer(brokers: &str, group_id: Option<&str>, auto_commit: bool) -> Self {
        let consumer = if auto_commit && group_id.is_some() {
            ClientConfig::new()
                .set("group.id", group_id.unwrap())
                .set("bootstrap.servers", brokers)
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "30000")
                .set("statistics.interval.ms", "1000")
                .create_with_context(CustomConsumerContext)
                .expect("Consumer creation failed")
        } else if auto_commit {
            ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("group.id", "fake")
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "smallest")
                .set_log_level(RDKafkaLogLevel::Debug)
                .create_with_context(CustomConsumerContext)
                .expect("Consumer creation failed")
        } else {
            ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("group.id", "fake")
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "30000")
                .set("statistics.interval.ms", "1000")
                .create_with_context(CustomConsumerContext)
                .expect("Consumer creation failed")
        };

        Self { consumer }
    }

    // TODO: Pasarlo a método porque el consumer lo tengo en `self`.
    pub async fn subscribe(
        consumer: &StreamConsumer<CustomConsumerContext>,
        topics: &[&str],
        messages: Arc<Mutex<Vec<KafkaConsumerMessage>>>,
    ) {
        consumer
            .subscribe(topics)
            .expect("Can't subscribe to specified topics");

        loop {
            println!("Waiting for message");
            match consumer.recv().await {
                Err(e) => println!("Kafka error: {}", e),
                Ok(m) => {
                    let payload = match m.payload_view::<str>() {
                        None => {
                            println!("No result");
                            ""
                        }
                        Some(Ok(s)) => {
                            let mut messages = messages.lock().unwrap();
                            let key = if m.key().is_none() {
                                String::from("")
                            } else {
                                format!("{:?}", m.key())
                            };
                            let msg = KafkaConsumerMessage {
                                key,
                                payload: String::from(s),
                                topic: format!("{:?}", m.topic()),
                                partition: format!("{:?}", m.partition()),
                                offset: format!("{:?}", m.offset()),
                                timestamp: format!("{:?}", m.timestamp()),
                            };
                            messages.push(msg);
                            s
                        }
                        Some(Err(e)) => {
                            println!("Error while deserializing message payload: {:?}", e);
                            ""
                        }
                    };
                    println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

                    // --> Para extraer cabeceras (esta API ya ha cambiado respecto a la documentación) <--
                    // Volvió a cambiar (240215). Ojo con esta api que es muy inestable.
                    if let Some(headers) = m.headers() {
                        let len = headers.count();
                        for idx in 0..len - 1 {
                            let opt_header = headers.try_get(idx);
                            if let Some(h) = opt_header {
                                println!("  Header {:#?}: {:?}", h.key, h.value);
                            }
                        }
                    }

                    // Para hacer el 'commit' explícito a partir del offset de un mensaje.
                    // consumer.commit_message(&m, CommitMode::Async).unwrap();
                }
            };
        }
    }

    // pub async fn create_consumer_and_subscribe(
    //     brokers: &str,
    //     group_id: &str,
    //     topics: &[&str],
    //     tx: &Sender<KafkaMessage>,
    // ) {
    //     // let context = CustomContext;
    //     let consumer: StreamConsumer =
    //         KafkaConsumer::create_consumer(&brokers, &group_id, true).await;
    //     KafkaConsumer::subscribe(&consumer, &topics, tx).await;
    // }
}

// fn topics_info(metadata: &Metadata) -> i64 {
//     println!("\nTopics:");
//     let mut message_count = 0;

//     for topic in metadata.topics() {
//         println!("  Topic: {}  Err: {:?}", topic.name(), topic.error());
//         for partition in topic.partitions() {
//             println!(
//                 "     Partition: {}  Leader: {}  Replicas: {:?}  ISR: {:?}  Err: {:?}",
//                 partition.id(),
//                 partition.leader(),
//                 partition.replicas(),
//                 partition.isr(),
//                 partition.error()
//             );
//             if true {
//                 // if fetch_offsets {
//                 let consumer: BaseConsumer = ClientConfig::new()
//                     .set("bootstrap.servers", "localhost:9095")
//                     .create()
//                     .expect("Consumer creation failed");
//                 let (low, high) = consumer
//                     .fetch_watermarks(topic.name(), partition.id(), Duration::from_secs(1))
//                     .unwrap_or((-1, -1));
//                 println!(
//                     "       Low watermark: {}  High watermark: {} (difference: {})",
//                     low,
//                     high,
//                     high - low
//                 );
//                 message_count += high - low;
//             }
//         }
//         // if fetch_offsets {
//         if true {
//             println!("     Total message count: {}", message_count);
//         }
//     }

//     message_count
// }
