// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Para poder conectarme a señales del sistema.
// use signal_hook::consts::signal::*;
// use signal_hook::flag;
use rdkafka::admin::AdminClient;
use rdkafka::client::{Client, DefaultClientContext};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::metadata::Metadata;
use rdkafka::producer::FutureProducer;
use rdkafka::statistics::Statistics;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::ClientContext;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// =================================
// Dominio
// =================================
pub struct KafkaConsumerMessage {
    pub key: String,
    pub topic: String,
    pub offset: String,
    pub timestamp: String,
    pub partition: String,
    pub payload: String,
}

pub enum KafkaMessage {
    Str(String),
    // ConsumerMessage(KafkaMessageBody),
    ClusterMetadata((usize, Metadata)),
}

// =================================
// Genéricas, referidas a structs/trait comunes
// =================================
pub struct Kafka {}

impl Kafka {
    pub fn extract_cluster_metadata_from_client<T: ClientContext>(
        client: &Client<T>,
    ) -> Option<Metadata> {
        match client.fetch_metadata(None, Duration::from_secs(20)) {
            Ok(metadata) => Some(metadata), // topics_info(&metadata);
            Err(err) => {
                println!("{:?}", err);
                None
            }
        }
    }
}

// =================================
// Admin
// =================================
// Ahora mismo nada en uso, dejo porque a futuro seguro que necesitamos cuando
// queramos implementar funcionalidades extra.
pub struct KafkaAdmin {}

impl KafkaAdmin {
    pub fn create(broker: &str) {
        // let broker = "localhost:9095";
        let client: AdminClient<DefaultClientContext> = ClientConfig::new()
            // let client = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .create()
            .expect("Error al crear AdminClient");
        let mut message_count = 0;

        match client.inner().fetch_metadata(None, Duration::from_secs(20)) {
            Ok(metadata) => {
                println!("Cluster information:");
                println!("  Broker count: {}", metadata.brokers().len());
                println!("  Topics count: {}", metadata.topics().len());
                println!("  Metadata broker name: {}", metadata.orig_broker_name());
                println!("  Metadata broker id: {}\n", metadata.orig_broker_id());

                println!("Brokers:");
                for broker in metadata.brokers() {
                    println!(
                        "  Id: {}  Host: {}:{}  ",
                        broker.id(),
                        broker.host(),
                        broker.port()
                    );
                }

                println!("\nTopics:");
                for topic in metadata.topics() {
                    println!("  Topic: {}  Err: {:?}", topic.name(), topic.error());
                    for partition in topic.partitions() {
                        println!(
                            "     Partition: {}  Leader: {}  Replicas: {:?}  ISR: {:?}  Err: {:?}",
                            partition.id(),
                            partition.leader(),
                            partition.replicas(),
                            partition.isr(),
                            partition.error()
                        );
                        if true {
                            // if fetch_offsets {
                            let consumer: BaseConsumer = ClientConfig::new()
                                .set("bootstrap.servers", "localhost:9095")
                                .create()
                                .expect("Consumer creation failed");
                            let (low, high) = consumer
                                .fetch_watermarks(
                                    topic.name(),
                                    partition.id(),
                                    Duration::from_secs(1),
                                )
                                .unwrap_or((-1, -1));
                            println!(
                                "       Low watermark: {}  High watermark: {} (difference: {})",
                                low,
                                high,
                                high - low
                            );
                            message_count += high - low;
                        }
                    }
                    // if fetch_offsets {
                    if true {
                        println!("     Total message count: {}", message_count);
                    }
                }
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

// =================================
// Productor
// =================================
// Podría (lo tenía de hecho) tenerlo en archivo propio, pero prefiero tenerlo
// todo en los menos archivos mejor y con estructura similar: view|presenter|state
pub struct CustomProducerContext;

impl ClientContext for CustomProducerContext {
    fn stats(&self, statistics: Statistics) {
        // Procesar las estadísticas aquí
        println!("Received statistics: {:?}", statistics);
    }
}

pub struct KafkaProducer {}

impl KafkaProducer {
    pub fn stats_listener(brokers: &str) -> FutureProducer<CustomProducerContext> {
        ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("statistics.interval.ms", "1000")
            .create_with_context(CustomProducerContext)
            .expect("Producer creation failed")
    }

    pub fn run_producer_loop(
        producer: FutureProducer<CustomProducerContext>,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::Relaxed) {
            // Aquí puedes realizar acciones con el productor, como enviar mensajes

            // Poll para asegurar que las devoluciones de llamada se ejecuten
            producer.poll(Duration::from_millis(100));

            // Pausa para reducir el uso de CPU
            thread::sleep(Duration::from_millis(100));
        }
    }
}

// fn main() {
// let producer = create_stats_producer();

// Configuración para manejo de señales
// let running = Arc::new(AtomicBool::new(true));
// flag::register_usize(SIGINT, Arc::clone(&running), 0).unwrap();

// run_producer_loop(producer, running);

// println!("Shutting down");
// }

// =================================
// Consumidor
// =================================
// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}

pub struct StatsContext;

impl ConsumerContext for StatsContext {}

impl ClientContext for StatsContext {
    fn stats(&self, statistics: Statistics) {
        // Convertir las estadísticas a JSON para un análisis más fácil
        // let stats_json: Value = serde_json::from_str(&statistics.to_json()).unwrap();

        // Aquí puedes procesar las estadísticas como prefieras
        println!("Statistics JSON: {statistics:?}");
    }
}


pub trait Create<T> {
    fn create(config: &str) -> T;
}

pub struct KafkaConsumer;

impl Create<StreamConsumer<CustomContext>> for KafkaConsumer {
    fn create(brokers: &str) -> StreamConsumer<CustomContext> {
        let context = CustomContext;
        ClientConfig::new()
            // .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "30000")
            .set("enable.auto.commit", "false")
            .set("statistics.interval.ms", "1000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed")
    }
}

impl KafkaConsumer {
    // https://github.com/fede1024/rust-rdkafka/blob/master/examples/simple_consumer.rs
    //   y (aunque viejo y API bastante cambiada)
    // https://github.com/fede1024/kafka-view/blob/master/examples/consumer_offsets_reader.rs
    pub async fn create_consumer(
        brokers: &str,
        group_id: &str,
        auto_commit: bool,
    ) -> StreamConsumer {
        if auto_commit {
            ClientConfig::new()
                .set("group.id", group_id)
                .set("bootstrap.servers", brokers)
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "30000")
                .create()
                .expect("Consumer creation failed")
        } else {
            ClientConfig::new()
                .set("group.id", group_id)
                .set("bootstrap.servers", brokers)
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "smallest")
                .set_log_level(RDKafkaLogLevel::Debug)
                .create()
                .expect("Consumer creation failed")
        }
    }

    pub async fn subscribe(
        consumer: &StreamConsumer,
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
                            // Envío a través de canales.
                            // let _ = tx.send(msg).await;

                            // ENVÍO A TRAVÉS DE ACTUALIZACIÓN DE VARIABLE COMPARTIDA. Dejo así para tener un modelo distinto.
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
