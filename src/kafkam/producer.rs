// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui::Context;
use log::info;
use rdkafka::client::Client;
use rdkafka::client::ClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::metadata::Metadata;
use rdkafka::producer::BaseProducer;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::Producer;
use rdkafka::statistics::Statistics;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use crate::kafkam::state::KafkaMessage;

/// Recuperamos metadatos, usamos contexto que no registra callback en `stats
pub fn get_cluster_metadata(
    rt: &Runtime,
    tx: &Sender<KafkaMessage>,
    ctx: &Context,
    broker_url: String,
    idx: usize,
) {
    let tx_cloned = tx.clone();
    let ctx_cloned = ctx.clone();

    rt.spawn(async move {
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", broker_url)
            .create()
            .expect("Producer creation failed");
        let client = producer.client();
        let metadata = client.fetch_metadata(None, Duration::from_secs(20));

        match metadata {
            Ok(data) => {
                let count = get_n_messages_per_topic(&data, client);
                let _ = tx_cloned
                    .send(KafkaMessage::ClusterMetadata((idx, data, count)))
                    .await;
            }
            Err(error) => {
                log::error!("Error: {error:?}");
                let _ = tx_cloned.send(KafkaMessage::Error(error)).await;
            }
        }

        ctx_cloned.request_repaint();
    });
}

/// Recuperamos datos de cluster y registramos callback para recibir estadísticas
pub fn get_cluster_metadata_and_stats(
    rt: &Runtime,
    tx: &Sender<KafkaMessage>,
    ctx: &Context,
    broker_url: String,
    idx: usize,
) {
    let tx_cloned = tx.clone();
    let ctx_cloned = ctx.clone();

    rt.spawn(async move {
        let ctx_cloned_again = ctx_cloned.clone();
        let producer = KafkaStatsProducerPresenter::new(ctx_cloned, &broker_url);
        let client = producer.client.client();
        let metadata = client.fetch_metadata(None, Duration::from_secs(20));

        match metadata {
            Ok(data) => {
                let count = get_n_messages_per_topic(&data, client);
                let _ = tx_cloned
                    .send(KafkaMessage::ClusterMetadata((idx, data, count)))
                    .await;
                ctx_cloned_again.request_repaint();
            }
            Err(error) => {
                log::error!("Error: {error:?}");
                let _ = tx_cloned.send(KafkaMessage::Error(error)).await;
                ctx_cloned_again.request_repaint();
            }
        }
    });
}

// =================================
// Productor
// =================================
// Podría (lo tenía de hecho) tenerlo en archivo propio, pero prefiero tenerlo
// todo en los menos archivos mejor y con estructura similar: view|presenter|state
use chrono::{Local, TimeZone, Timelike};
use eframe::egui;

fn get_now() -> String {
    let now = Local::now();
    let time = Local.timestamp_opt(now.timestamp(), 0);

    match time {
        chrono::offset::LocalResult::Single(time) => {
            let hour = time.hour();
            let minute = time.minute();
            let second = time.second();

            format!("{:02}:{:02}:{:02}", hour, minute, second)
        }
        chrono::offset::LocalResult::Ambiguous(_earliest, latest) => {
            let hour = latest.hour();
            let minute = latest.minute();
            let second = latest.second();

            format!("{:02}:{:02}:{:02}", hour, minute, second)
        }
        chrono::offset::LocalResult::None => String::default(),
    }
}

pub struct StatsProducerContext {
    pub stats: Arc<Mutex<Vec<(String, Statistics)>>>,
    // para debug
    pub print: AtomicBool,
    pub ctx: egui::Context,
}

impl ClientContext for StatsProducerContext {
    fn stats(&self, statistics: Statistics) {
        // Realmente con Option<> puedo, pero por si acaso en algún momento quiero mantener histórico
        // de estadísticas (no sé para qué, pero puede ser que venga bien, si no todo, sí al menos)
        // ciertas cosas, como consumo de memoria, etc.
        let mut stats = self.stats.lock().unwrap(); // Adquiere el lock una vez aquí
        if stats.is_empty() {
            stats.push((get_now(), statistics));
        } else {
            stats[0] = (get_now(), statistics);
        }
        if self.print.load(std::sync::atomic::Ordering::SeqCst) {
            self.print.store(false, std::sync::atomic::Ordering::SeqCst);
        }
        self.ctx.request_repaint();
    }
}

pub struct KafkaStatsProducerPresenter {
    pub client: FutureProducer<StatsProducerContext>,
    pub stats: Arc<Mutex<Vec<(String, Statistics)>>>,
}

impl KafkaStatsProducerPresenter {
    pub fn new(ctx: egui::Context, brokers: &str) -> Self {
        let stats = Arc::new(Mutex::new(Vec::with_capacity(1)));
        let context = StatsProducerContext {
            stats: stats.clone(),
            print: AtomicBool::new(true),
            ctx,
        };
        let client = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("statistics.interval.ms", "10000") // 10s
            .set("api.version.fallback.ms", "0") // por si corregía error con fetch_metadata, no lo ha conseguido
            .create_with_context(context)
            .expect("Producer creation failed");

        Self { client, stats }
    }
}

fn get_n_messages_per_topic<T: ClientContext>(
    data: &Metadata,
    client: &Client<T>,
) -> HashMap<String, i64> {
    let mut count: HashMap<String, i64> = HashMap::default();
    for topic in data.topics() {
        let mut message_count: i64 = 0;
        for partition in topic.partitions() {
            let (low, high) = client
                .fetch_watermarks(topic.name(), partition.id(), Duration::from_secs(1))
                .unwrap_or((-1, -1));

            message_count += high - low;
        }
        count.insert(topic.name().to_owned(), message_count);
    }
    count
}
