// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::default::Default;
use std::sync::{Arc, Mutex};

use bollard::network::ListNetworksOptions;
use bollard::secret::{ImageSummary, Network, Volume};
use bollard::volume::ListVolumesOptions;
use log;

use bollard::container::ListContainersOptions;
use bollard::image::ListImagesOptions;
use bollard::models::ContainerSummary;
use bollard::Docker;

use crate::domain::{ContainerInfo, NetworkInfo};

#[derive(Default)]
pub struct DockerPresenter {}

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

async fn list_containers(docker: Option<Docker>) -> Result<Vec<ContainerSummary>, String> {
    match docker {
        Some(docker) => {
            let containers = &docker
                .list_containers(Some(ListContainersOptions::<String> {
                    all: true,
                    ..Default::default()
                }))
                .await;
            match containers {
                Ok(containers) => Ok(containers.to_vec()),
                Err(e) => Err(e.to_string()),
            }
        }
        None => Err("Connection".to_string()),
    }
}

async fn list_networks(docker: Option<Docker>) -> Result<Vec<Network>, String> {
    match docker {
        Some(docker) => {
            let networks = &docker.list_networks(Some(ListNetworksOptions::<String> {
                    ..Default::default()
                })).await;

            match networks {
                Ok(ns) => Ok(ns.to_vec()),
                Err(e) => Err(e.to_string()),
            }
        }
        None => Err("Connection".to_string()),
    }
}

async fn list_volumes(docker: Option<Docker>) -> Result<Vec<Volume>, String> {
    match docker {
        Some(docker) => {
            let volumes = &docker.list_volumes(Some(ListVolumesOptions::<String> {
                    ..Default::default()
                })).await;

            match volumes {
                Ok(vs) => Ok(vs.volumes.clone().unwrap_or_default().to_vec()),
                Err(e) => Err(e.to_string()),
            }
        }
        None => Err("Connection".to_string()),
    }
}

impl DockerPresenter {
    pub async fn populate_state(
        docker: Option<Docker>,
        images: Arc<Mutex<Vec<ImageSummary>>>,
        containers: Arc<Mutex<Vec<ContainerInfo>>>,
        networks: Arc<Mutex<Vec<NetworkInfo>>>,
        volumes: Arc<Mutex<Vec<Volume>>>,
    ) {
        let dcl1 = docker.clone();
        let dcl2 = docker.clone();
        let dcl3 = docker.clone();

        let images_task = tokio::spawn(async move {
            DockerPresenter::populate_images(dcl1, images.clone()).await;
        });
        let containers_task = tokio::spawn(async move {
            DockerPresenter::populate_containers(dcl2, containers).await;
        });
        let networks_task = tokio::spawn(async move {
            DockerPresenter::populate_networks(dcl3, networks).await;
        });
        let volumes_task = tokio::spawn(async move {
            DockerPresenter::populate_volumes(docker, volumes).await;
        });

        if let Err(e) = images_task.await {
            log::error!("Error in images task: {:?}", e);
        }
        if let Err(e) = containers_task.await {
            log::error!("Error in containers task: {:?}", e);
        }
        if let Err(e) = networks_task.await {
            log::error!("Error in networks task: {:?}", e);
        }
        if let Err(e) = volumes_task.await {
            log::error!("Error in volumes task: {:?}", e);
        }
    }

    pub async fn populate_images(docker: Option<Docker>, images: Arc<Mutex<Vec<ImageSummary>>>) {
        let data = Arc::clone(&images);
        let result = list_images(docker).await;

        match result {
            Ok(docker_images) => {
                let mut images = data.lock().unwrap();
                for img in docker_images {
                    images.push(img);
                }
            }
            Err(err) => {
                log::error!("Error trying to list docker images: {err:?}");
            }
        }
    }

    pub async fn populate_containers(docker: Option<Docker>, cs: Arc<Mutex<Vec<ContainerInfo>>>) {
        let data = Arc::clone(&cs);
        let result = list_containers(docker).await;

        match result {
            Ok(docker_containers) => {
                let mut containers = data.lock().unwrap();
                for cnt in docker_containers {
                    containers.push(ContainerInfo::from_summary(cnt));
                }
            }
            Err(err) => {
                log::error!("Error trying to list docker containers: {err:?}");
            }
        }
    }

     pub async fn populate_networks(docker: Option<Docker>, ns: Arc<Mutex<Vec<NetworkInfo>>>) {
        let data = Arc::clone(&ns);
        let result = list_networks(docker).await;

        match result {
            Ok(docker_networks) => {
                let mut networks = data.lock().unwrap();
                for net in docker_networks {
                    networks.push(NetworkInfo::from_network(net));
                }
            }
            Err(err) => {
                log::error!("Error trying to list docker networks: {err:?}");
            }
        }
     }

    pub async fn populate_volumes(docker: Option<Docker>, volumes: Arc<Mutex<Vec<Volume>>>) {
        let data = Arc::clone(&volumes);
        let result = list_volumes(docker).await;

        match result {
            Ok(docker_volumes) => {
                let mut volumes = data.lock().unwrap();
                for vlm in docker_volumes {
                    log::info!("===========================");
                    log::info!("{:?}", vlm.usage_data);
                    log::info!("{:?}", vlm.options);
                    log::info!("{:?}", vlm.labels);
                    log::info!("{:?}", vlm.status);
                    log::info!("{:?}", vlm.usage_data);

                    volumes.push(vlm);
                }
            }
            Err(err) => {
                log::error!("Error trying to list docker volumes: {err:?}");
            }
        }
    }
}

pub fn connect() -> Option<Docker> {
    Docker::connect_with_local_defaults().ok()
}

// async fn conc(arg: (&Docker, &ContainerSummary)) {
//     let (docker, container) = arg;
//     log::info!(
//         "{:?}",
//         docker
//             .inspect_container(
//                 container.id.as_ref().unwrap(),
//                 None::<InspectContainerOptions>
//             )
//             .await
//             .unwrap()
//     )
// }

// // Miucha información por contenedor
// pub async fn info(docker: &Docker) -> Result<(), Box<dyn std::error::Error + 'static>> {
//     // let docker = Docker::connect_with_socket_defaults().unwrap();

//     let mut list_container_filters = HashMap::new();
//     list_container_filters.insert("status", vec!["running"]);

//     let containers = &docker
//         .list_containers(Some(ListContainersOptions {
//             all: true,
//             filters: list_container_filters,
//             ..Default::default()
//         }))
//         .await?;

//     let docker_stream = stream::repeat(docker);
//     docker_stream
//         .zip(stream::iter(containers))
//         .for_each_concurrent(2, conc)
//         .await;

//     Ok(())
// }

// pub async fn stats(docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
//     // loop {
//     let mut filter = HashMap::new();
//     filter.insert(String::from("status"), vec![String::from("running")]);
//     let containers = &docker
//         .list_containers(Some(ListContainersOptions {
//             all: true,
//             filters: filter,
//             ..Default::default()
//         }))
//         .await?;

//     if containers.is_empty() {
//         panic!("no running containers");
//     } else {
//         for container in containers {
//             let container_id = container.id.as_ref().unwrap();
//             let stream = &mut docker
//                 .stats(
//                     container_id,
//                     Some(StatsOptions {
//                         stream: false,
//                         ..Default::default()
//                     }),
//                 )
//                 .take(1);

//             while let Some(Ok(stats)) = stream.next().await {
//                 log::info!(
//                     "{} - {:?}: {:?} {:?}",
//                     container_id,
//                     &container.names,
//                     container.image,
//                     stats
//                 );
//             }
//         }
//     }
//     Ok(())
//     // }
// }
