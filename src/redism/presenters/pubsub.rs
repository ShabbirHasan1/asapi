// -------------------------------------------------------------------------
// Copyright (C)  Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use redis::{self, Commands, Msg as PubSubMsg, RedisError, RedisResult};

use crate::info;

use super::create_conn_with_default;

pub fn publish_to_channel(
    host: &str,
    port: &str,
    channel: &str,
    message: &str,
) -> RedisResult<bool> {
    create_conn_with_default(host, port).and_then(|mut conn| conn.publish(channel, message))
}

pub fn subscribe_to_channel_std_thread(
    host: &str,
    port: &str,
    channel: &str,
    tx: &std::sync::mpsc::Sender<PubSubMsg>,
) -> Result<(), RedisError> {
    let mut conn = create_conn_with_default(host, port)?;
    let owned_channel = channel.to_owned();
    let tx_owned = tx.to_owned();

    std::thread::spawn(move || -> Result<(), RedisError> {
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(&owned_channel).unwrap();

        loop {
            let msg = pubsub.get_message()?;
            let payload: String = msg.get_payload()?;

            match payload.as_ref() {
                "#break#" => {
                    info!(
                        ">>> Finishing subscription to channel {} <<<",
                        msg.get_channel_name()
                    );
                    break;
                } //ControlFlow::Break(()),
                _ => (), // ControlFlow::Continue
            }

            let _ = tx_owned.send(msg);
        }
        pubsub.unsubscribe(&owned_channel)?;

        Ok(())
    });
    Ok(())
}
