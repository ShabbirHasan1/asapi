// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;
use std::default::Default;
use std::sync::{Arc, Mutex};

use bollard::secret::ImageSummary;
use log;

use bollard::container::{InspectContainerOptions, ListContainersOptions, StatsOptions};
use bollard::errors;
use bollard::image::ListImagesOptions;
use bollard::models::ContainerSummary;
use bollard::Docker;

use futures_util::stream;
use futures_util::stream::StreamExt;

#[derive(Default)]
pub struct DockerPresenter {
    pub connection: Option<Docker>,
}

async fn list_images(docker: Option<Docker>) -> Result<Vec<ImageSummary>, String> {
    match docker {
        Some(docker) => {
            let images = &docker
                .list_images(Some(ListImagesOptions::<String> {
                    all: true,
                    ..Default::default()
                }))
                .await;
            match images {
                Ok(images) => Ok(images.to_vec()),
                Err(e) => Err(e.to_string()),
            }
        }

        None => Err("Connection".to_string()),
    }
}

impl DockerPresenter {
    pub async fn populate_images(docker: Option<Docker>, images: Arc<Mutex<Vec<ImageSummary>>>) {
        let data = Arc::clone(&images);
        let result = list_images(docker).await;

        match result {
            Ok(docker_images) => {
                let mut images = data.lock().unwrap();
                for img in docker_images {
                    log::info!("{img:?}");
                    images.push(img);
                }
            }
            Err(err) => {
                log::error!("Error trying to list docker images: {err:?}");
            }
        }
    }
}

pub fn connect() -> Option<Docker> {
    Docker::connect_with_local_defaults().ok()
}

async fn conc(arg: (&Docker, &ContainerSummary)) {
    let (docker, container) = arg;
    log::info!(
        "{:?}",
        docker
            .inspect_container(
                container.id.as_ref().unwrap(),
                None::<InspectContainerOptions>
            )
            .await
            .unwrap()
    )
}

// Miucha información por contenedor
pub async fn info(docker: &Docker) -> Result<(), Box<dyn std::error::Error + 'static>> {
    // let docker = Docker::connect_with_socket_defaults().unwrap();

    let mut list_container_filters = HashMap::new();
    list_container_filters.insert("status", vec!["running"]);

    let containers = &docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: list_container_filters,
            ..Default::default()
        }))
        .await?;

    let docker_stream = stream::repeat(docker);
    docker_stream
        .zip(stream::iter(containers))
        .for_each_concurrent(2, conc)
        .await;

    Ok(())
}

pub async fn stats(docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
    // loop {
    let mut filter = HashMap::new();
    filter.insert(String::from("status"), vec![String::from("running")]);
    let containers = &docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;

    if containers.is_empty() {
        panic!("no running containers");
    } else {
        for container in containers {
            let container_id = container.id.as_ref().unwrap();
            let stream = &mut docker
                .stats(
                    container_id,
                    Some(StatsOptions {
                        stream: false,
                        ..Default::default()
                    }),
                )
                .take(1);

            while let Some(Ok(stats)) = stream.next().await {
                log::info!(
                    "{} - {:?}: {:?} {:?}",
                    container_id,
                    &container.names,
                    container.image,
                    stats
                );
            }
        }
    }
    Ok(())
    // }
}
