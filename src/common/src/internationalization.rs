// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::i18n_clickhouse::I18nClickHouse;
use crate::i18n_docker::I18nDocker;
use crate::i18n_http::I18nHttp;
use crate::i18n_kafka::I18nKafka;
use crate::i18n_mongo::I18nMongo;
use crate::i18n_redis::I18nRedis;
use crate::i18n_sqlx::I18nSqlx;
use crate::i18n_config::I18nConfig;

pub struct I18nPack {
    pub http: I18nHttp,
    pub docker: I18nDocker,
    pub sqlx: I18nSqlx,
    pub clickhouse: I18nClickHouse,
    pub mongo: I18nMongo,
    pub redis: I18nRedis,
    pub kafka: I18nKafka,
    pub config: I18nConfig,
}

impl I18nPack {
    pub fn new_en() -> Self {
        I18nPack::new(
            I18nHttp::new_en(),
            I18nDocker::new_en(),
            I18nSqlx::new_en(),
            I18nClickHouse::new_en(),
            I18nMongo::new_en(),
            I18nRedis::new_en(),
            I18nKafka::new_en(),
            I18nConfig::new_en(),
        )
    }

    pub fn new_es() -> Self {
        I18nPack::new(
            I18nHttp::new_es(),
            I18nDocker::new_es(),
            I18nSqlx::new_es(),
            I18nClickHouse::new_es(),
            I18nMongo::new_es(),
            I18nRedis::new_es(),
            I18nKafka::new_es(),
            I18nConfig::new_es(),
        )
    }

    pub fn new(
        http: I18nHttp,
        docker: I18nDocker,
        sqlx: I18nSqlx,
        clickhouse: I18nClickHouse,
        mongo: I18nMongo,
        redis: I18nRedis,
        kafka: I18nKafka,
        config: I18nConfig,
    ) -> Self {
        I18nPack {
            http,
            docker,
            sqlx,
            clickhouse,
            mongo,
            redis,
            kafka,
            config
        }
    }
}

pub struct LanguagePack {
    english: OnceLock<I18nPack>,
    spanish: OnceLock<I18nPack>,
}

impl LanguagePack {
    fn new() -> Self {
        LanguagePack {
            english: OnceLock::new(),
            spanish: OnceLock::new(),
        }
    }

    pub fn get_english(&self) -> &I18nPack {
        self.english.get_or_init(I18nPack::new_en)
    }

    pub fn get_spanish(&self) -> &I18nPack {
        self.spanish.get_or_init(I18nPack::new_es)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub enum I18nOptions {
    ES,
    #[default]
    EN,
}


static LANGUAGE_PACK: OnceLock<LanguagePack> = OnceLock::new();

pub fn language_selector(lang: I18nOptions) -> &'static I18nPack {
    let pack = LANGUAGE_PACK.get_or_init(LanguagePack::new);

    match lang {
        I18nOptions::EN => pack.get_english(),
        I18nOptions::ES => pack.get_spanish(),
    }
}
