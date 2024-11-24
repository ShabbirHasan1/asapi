// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use bollard::secret::{
    BollardDate, ContainerSummary, ContainerSummaryHostConfig, ContainerSummaryNetworkSettings, MountPoint, Network, NetworkContainer, PeerInfo, Port
};

#[derive(Debug)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub ports: Vec<Port>,
    pub size_root_fs: i64,
    pub state: String,
    pub status: String,
    pub host_config: ContainerSummaryHostConfig,
    pub network_settings: ContainerSummaryNetworkSettings,
    pub mounts: Vec<MountPoint>,
}

impl ContainerInfo {
    pub fn from_summary(cs: ContainerSummary) -> ContainerInfo {
        let name =
            if let Some(container_path) = cs.names.clone().as_deref().unwrap_or_default().first() {
                let remaining_str = &container_path[1..];
                if remaining_str.len() > 1 {
                    remaining_str
                } else {
                    ""
                }
            } else {
                ""
            }
            .to_string();

        ContainerInfo {
            id: cs.id.unwrap_or_default(),
            name,
            image: cs.image.unwrap_or_default(),
            image_id: cs.image_id.unwrap_or_default(),
            command: cs.command.unwrap_or_default(),
            created: cs.created.unwrap_or_default(),
            ports: cs.ports.unwrap_or_default(),
            size_root_fs: cs.size_root_fs.unwrap_or_default(),
            state: cs.state.unwrap_or_default(),
            status: cs.status.unwrap_or_default(),
            host_config: cs.host_config.unwrap_or_default(),
            network_settings: cs.network_settings.unwrap_or_default(),
            mounts: cs.mounts.unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub struct NetworkInfo {
    pub name: String,
    pub id: String,
    pub created: BollardDate,
    pub scope: String,
    pub driver: String,
    pub enable_ipv6: bool,
    pub internal: bool,
    pub containers: HashMap<String, NetworkContainer>,
    pub options: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub peers: Vec<PeerInfo>,
    pub n_containers: usize,
    pub n_peers: usize,
}

impl NetworkInfo {
    pub fn from_network(net: Network) -> NetworkInfo {
        let containers= net.containers.unwrap_or_default();
        let peers= net.peers.unwrap_or_default();
        let n_containers = containers.len();
        let n_peers = peers.len();

        NetworkInfo {
            name: net.name.unwrap_or_default(),
            id: net.id.unwrap_or_default(),
            created: net.created.unwrap_or_default(),
            scope: net.scope.unwrap_or_default(),
            driver: net.driver.unwrap_or_default(),
            enable_ipv6: net.enable_ipv6.unwrap_or(false),
            internal: net.internal.unwrap_or(true),
            containers,
            options: net.options.unwrap_or_default(),
            labels: net.labels.unwrap_or_default(),
            peers,
            n_containers,
            n_peers
        }
    }
}
