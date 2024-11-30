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

use crate::domain::{
    ContainerInfo, ImageDistributionInspect, ImageInfo, ImageInspectInfo, NetworkInfo,
};

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
            let networks = &docker
                .list_networks(Some(ListNetworksOptions::<String> {
                    ..Default::default()
                }))
                .await;

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
            let volumes = &docker
                .list_volumes(Some(ListVolumesOptions::<String> {
                    ..Default::default()
                }))
                .await;

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

pub struct DockerImagePresenter {}

impl DockerImagePresenter {
    pub async fn get_image_info(
        conn: &Docker,
        image_summary: ImageSummary,
        image_name: String,
    ) -> Result<ImageInfo, String> {
        let image_inspect = conn.inspect_image(&image_name).await;
        let image_registry = conn.inspect_registry_image(&image_name, None).await;
        let image_history = conn.image_history(&image_name).await;

        match (image_inspect, image_registry, image_history) {
            (Ok(info), Ok(registry), Ok(history)) => Ok(ImageInfo(
                image_name,
                ImageInspectInfo::from_image_inspect(info),
                image_summary,
                ImageDistributionInspect::from_bollard(registry),
                history,
            )),

            _ => Err("Error getting image info".to_string()),
        }
        // No me está funcionando pero hace toda la petición del tirón.
        // let c1 = conn.clone();
        // let c2 = conn.clone();
        // let c3 = conn.clone();

        // let in1 = image_name.clone();
        // let in2 = image_name.clone();

        // let image_inspect = tokio::spawn(async move { c1.inspect_image(&in1).await });
        // log::info!("foo");
        // let image_registry =
        //     tokio::spawn(async move { c2.inspect_registry_image(&in2, None).await });
        // log::info!("bar");
        // let image_history = tokio::spawn(async move { c3.image_history(&image_name).await });
        // log::info!("baz");

        // let (inspect_res, registry_res, history_res) =
        //     tokio::join!(image_inspect, image_registry, image_history,);

        // match (
        //     inspect_res,  // .map_err(|e| format!("Inspect task error: {}", e)),
        //     registry_res, // .map_err(|e| format!("Registry task error: {}", e)),
        //     history_res,  // .map_err(|e| format!("History task error: {}", e)),
        // ) {
        //     (Ok(inspect), Ok(registry_info), Ok(history)) => {
        //         log::info!("{inspect:?}");
        //         Ok(ImageInfo(
        //             inspect.unwrap_or_default(),
        //             registry_info.unwrap_or_default(),
        //             history.unwrap_or_default(),
        //         ))
        //     }
        //     (_, _, _) => Err("Failed to get image info".to_string()),
        // }
    }
}

pub struct DockerContainerPresenter {}

// impl DockerContainerPresenter {
//     pub async fn get_container_info(
//         conn: &Docker,
//         container_info: ContainerInfo,
//     ) -> Result<ImageInfo, String> {
//     }
// }
