// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use eframe::egui;
use rdkafka::{statistics::Window, Statistics};

use crate::{
    common::internationalization::I18n, ui_title_and_value_grid_row,
    ui_title_and_value_grid_row_with_hint,
};

pub fn show_stats(ui: &mut egui::Ui, last_update: &String, stats: &Statistics, i18n: &I18n) {
    ui.label(format!("{}: {last_update}", &i18n.kafka_last_update));
    egui::ScrollArea::horizontal().show(ui, |ui| {
        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui_title_and_value_grid_row!(ui, "Name", &stats.name);
            ui_title_and_value_grid_row!(ui, "Client Id", &stats.client_id);
            ui_title_and_value_grid_row!(ui, "Client Type", &stats.client_type);
            ui_title_and_value_grid_row!(
                ui,
                "#Operations waiting in queue",
                &stats.replyq.to_string()
            );
            ui_title_and_value_grid_row!(
                ui,
                "The current number of messages in producer queues",
                &stats.msg_cnt.to_string()
            );
            ui_title_and_value_grid_row!(
                ui,
                "The current total size of messages in producer queues",
                &stats.msg_size.to_string()
            );
            ui_title_and_value_grid_row!(
                ui,
                "The maximum number of messages allowed in the producer queues.",
                &stats.msg_max.to_string()
            );
            ui_title_and_value_grid_row!(
                ui,
                "The maximum total size of messages allowed in the producer queues.",
                &stats.msg_size_max.to_string()
            );
            ui_title_and_value_grid_row!(
                ui,
                "The total number of requests sent to brokers.",
                &stats.tx.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of bytes transmitted to brokers.",
                &stats.tx_bytes.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of responses received from brokers.",
                &stats.rx.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of bytes received from brokers.",
                &stats.rx_bytes.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of messages transmitted (produced) to brokers.",
                &stats.txmsgs.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of bytes transmitted (produced) to brokers.",
                &stats.txmsg_bytes.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of messages consumed from brokers, not including ignored messages.",
                &stats.rxmsgs.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "The total number of bytes (including framing) consumed from brokers.",
                &stats.rxmsg_bytes.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "Internal tracking of legacy vs. new consumer API state.",
                &stats.simple_cnt.to_string()
            );

            ui_title_and_value_grid_row!(
                ui,
                "Number of topics in the metadata cache.",
                &stats.metadata_cache_cnt.to_string()
            );
        });
    });

    egui::ScrollArea::horizontal()
        .id_source("scroll_brokers")
        .show(ui, |ui| {

            // StripBuilder::new(ui)
            // .size(Size::remainder())
            // .size(Size::remainder())
            // .horizontal(|mut strip| {
            // info!("{}", stats.brokers.len());
            // let mut names: HashSet<&str> = HashSet::default();
            // let names = stats.brokers.iter().map(|(k, v)| v.name.as_str()).collect::<HashSet<&str>>();

            for (broker_id, broker) in &stats.brokers {
                if broker.nodeid < 0 {
                    continue;
                }
                // strip.cell(|ui| {
                ui.collapsing(format!("Broker {}", broker.nodeid), |ui| {
                    egui::Grid::new(format!("broker-grid{}", broker.nodeid))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui_title_and_value_grid_row!(ui, "Name", &broker.name);
                            ui_title_and_value_grid_row!(ui, "Broker ID", broker_id);

                            ui_title_and_value_grid_row!(
                                ui,
                                "Node ID",
                                &broker.nodeid.to_string()
                            );

                            ui_title_and_value_grid_row!(ui, "Node Name", &broker.nodename);

                            // The broker source (learned, configured, internal, or logical).
                            ui_title_and_value_grid_row!(ui, "Broker Source", &broker.source);

                            // The broker state (INIT, DOWN, CONNECT, AUTH, APIVERSION_QUERY, AUTH_HANDSHAKE, UP, UPDATE).
                            ui_title_and_value_grid_row!(ui, "Broker State", &broker.state);

                            ui_title_and_value_grid_row!(ui, "Time since the last broker state change (ms).", &broker.stateage.to_string());

                            ui_title_and_value_grid_row!(ui, "Number of requests awaiting transmission to the broker", &broker.outbuf_cnt.to_string());

                            ui_title_and_value_grid_row!(ui, "Number of messages awaiting transmission to the broker", &broker.outbuf_msg_cnt.to_string());

                            ui_title_and_value_grid_row!(ui, "Number of requests in-flight to the broker that are awaiting a response", &broker.waitresp_cnt.to_string());

                            ui_title_and_value_grid_row!(ui, "Number of messages in-flight to the broker that are awaiting a response", &broker.waitresp_msg_cnt.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of requests sent to the broker", &broker.tx.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of bytes sent to the broker", &broker.txbytes.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of transmission errors", &broker.txerrs.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of request retries", &broker.txretries.to_string());

                            ui_title_and_value_grid_row_with_hint!(ui, "Microseconds since last socket send", &broker.txidle.to_string(), "-1 if no sends yet for the current connection");

                            ui_title_and_value_grid_row!(ui, "Total number of requests that timed out", &broker.req_timeouts.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of requests received the broker", &broker.rx.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of bytes received from the broker", &broker.rxbytes.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of receive errors", &broker.rxerrs.to_string());

                            ui_title_and_value_grid_row!(ui, "Number of unmatched correlation IDs in response, typically for timed out requests", &broker.rxcorriderrs.to_string());

                            ui_title_and_value_grid_row_with_hint!(ui, "Total number of partial message sets received", &broker.rxpartial.to_string(), "The broker may return partial responses if the full message set could not fit in the remaining fetch response size");

                            ui_title_and_value_grid_row_with_hint!(ui, "Microseconds since last socket receive", &broker.rxidle.to_string(), "-1 if no receives yet for the current connection");


                            ui_title_and_value_grid_row!(ui, "Total number of decompression buffer size increases", &broker.zbuf_grow.to_string());

                            ui_title_and_value_grid_row!(ui, "Total number of buffer size increases (deprecated and unused)", &broker.buf_grow.to_string());

                            // TODO: Todos estos faltan por mostrar.
                            // req: HashMap<String, i64>
                            // Request type counters. The object key is the name of the request type and the value is the number of requests of that type that have been sent.

                            // wakeups: Option<u64>
                            // The number of broker thread poll wakeups.

                            // connects: Option<i64>
                            // The number of connection attempts, including successful and failed attempts, and name resolution failures.

                            // disconnects: Option<i64>
                            // The number of disconnections, whether triggered by the broker, the network, the load balancer, or something else.

                            // int_latency: Option<Window>
                            // Rolling window statistics for the internal producer queue latency, in microseconds.

                            // outbuf_latency: Option<Window>
                            // Rolling window statistics for the internal request queue latency, in microseconds.
                            // This is the time between when a request is enqueued on the transmit (outbuf) queue and the time the request is written to the TCP socket. Additional buffering and latency may be incurred by the TCP stack and network.

                            // rtt: Option<Window>
                            // Rolling window statistics for the broker latency/round-trip time, in microseconds.

                            // throttle: Option<Window>
                            // Rolling window statistics for the broker throttling time, in milliseconds.

                            // toppars: HashMap<String, TopicPartition>
                            // The partitions that are handled by this broker handle.

                            // ui.end_row();
                            // });
                            ui.separator();
                        });
                });
            }

            for (topic_id, topic) in &stats.topics {
                ui.collapsing(format!("topic {}", topic_id), |ui| {
                    egui::Grid::new(format!("topic-grid{}", topic_id))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui_title_and_value_grid_row!(ui, "Name", &topic.topic);
                            ui_title_and_value_grid_row!(ui, "Age of the client’s metadata (ms)", &topic.metadata_age.to_string());

                            statistics_window_show(ui, &topic.batchsize, "Rolling window statistics for batch sizes, in bytes");

                            statistics_window_show(ui, &topic.batchcnt, "Rolling window statistics for batch message counts");
                        });
                });
            }
        });
}

fn statistics_window_show(ui: &mut egui::Ui, w: &Window, title: &str) {
    ui.collapsing(title, |ui| {
        ui_title_and_value_grid_row!(ui, "Smallest", &w.min.to_string());
        ui_title_and_value_grid_row!(ui, "Largest", &w.max.to_string());
        ui_title_and_value_grid_row!(ui, "Mean", &w.avg.to_string());
        ui_title_and_value_grid_row!(ui, "Sum", &w.sum.to_string());
        ui_title_and_value_grid_row!(ui, "Total Number of Values", &w.cnt.to_string());
        ui_title_and_value_grid_row!(ui, "Standard Deviation", &w.stddev.to_string());
        ui_title_and_value_grid_row!(
            ui,
            "The memory size of the underlying HDR histogram.",
            &w.hdrsize.to_string()
        );
        ui_title_and_value_grid_row!(ui, "50th percentile", &w.p50.to_string());
        ui_title_and_value_grid_row!(ui, "75th percentile", &w.p75.to_string());
        ui_title_and_value_grid_row!(ui, "90th percentile", &w.p90.to_string());
        ui_title_and_value_grid_row!(ui, "95th percentile", &w.p95.to_string());
        ui_title_and_value_grid_row!(ui, "99th percentile", &w.p99.to_string());
        ui_title_and_value_grid_row!(ui, "99.99th percentile", &w.p99_99.to_string());
        ui_title_and_value_grid_row_with_hint!(
        ui,
        "Out of Range",
        &w.outofrange.to_string(),
        "Number of values not included in te unerlying histogram because the were out of range."
    );
    });
}
