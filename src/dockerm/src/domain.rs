// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

use bollard::{
    container::{CPUStats, MemoryStats, StorageStats},
    secret::{
        BollardDate, ContainerSummary, ContainerSummaryHostConfig, ContainerSummaryNetworkSettings,
        ImageConfig as BollardImageConfig, MountPoint, Network, NetworkContainer, PeerInfo, Port,
    },
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

use bollard::secret::{
    DistributionInspect, HistoryResponseItem, ImageInspect, ImageSummary, Volume,
};

#[derive(Default, PartialEq)]
pub enum DockerViewMode {
    #[default]
    Default,
    Chart,
}

// Guardo valores que hay que implementar cada iteración iguales
pub struct DockerDefaults {
    pub empty_dt: DateTime<Utc>,
}

impl Default for DockerDefaults {
    fn default() -> Self {
        let and_hms_opt = NaiveDate::from_ymd_opt(1, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0);
        Self {
            empty_dt: DateTime::<Utc>::from_naive_utc_and_offset(and_hms_opt.unwrap(), Utc),
        }
    }
}

#[derive(Default)]
pub struct DockerLocalState {
    pub images: Arc<Mutex<Vec<ImageSummary>>>,
    pub containers: Arc<Mutex<Vec<ContainerInfo>>>,
    pub volumes: Arc<Mutex<Vec<Volume>>>,
    pub networks: Arc<Mutex<Vec<NetworkInfo>>>,
    pub current_selection: Option<DockerSelection>,
    pub selected_image_info: ImageInfo,
    pub container: DockerContainerState,
}


#[derive(Default, Debug)]
pub struct DockerContainerStats {
    pub dates: Vec<DateTime<Utc>>,
    pub cpu: Vec<CPUStats>,
    pub mem: Vec<MemoryStats>,
    pub disk: Vec<StorageStats>,
}

#[derive(Default)]
pub struct DockerContainerState {
    pub info: ContainerInfo,
    pub logs: Vec<String>,
    pub show_stats: bool,
    pub stats: HashMap<String, DockerContainerStats>,
}

#[derive(Default, Serialize, Clone, Debug, Deserialize)]
pub struct DockerAppState {
    pub show_sidebar: bool,
}

#[derive(Debug)]
pub enum DockerInfo {
    Image(ImageInfo),
    Container(ContainerInfo),
    ContainerAll,
}

#[derive(Debug)]
pub enum DockerMessage {
    Error(String),
    Loading,
    Select((usize, DockerInfo)),
    LogStdIn(String),
    LogStdOut(String),
    LogStdErr(String),
    LogConsole(String),
    StatsReady, // podemos leer estadísticas porque ya tenemos nombres de contenedores.
    Stats(
        (
            CPUStats,
            MemoryStats,
            StorageStats,
            chrono::DateTime<chrono::Utc>,
        ),
    ),
}

#[derive(Debug)]
pub struct DockerSelection {
    pub selected_view: DockerElementSelection,
    pub selected_idx: usize, // índice del elemento dentro del vector de su tipo
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DockerElementSelection {
    Image,
    Container,
    ContainerAll,
    Volume,
    Network,
}

#[derive(Default, Debug)]
pub struct ImageInfo(
    pub String,
    pub ImageInspectInfo,
    pub ImageSummary,
    pub ImageDistributionInspect,
    pub Vec<HistoryResponseItem>,
);

#[derive(Default, Debug)]
pub struct ImageInfoWrapper(pub Option<ImageInfo>);

#[derive(Default, Debug)]
pub struct ImageConfig {
    pub hostname: String,
    pub domainname: String,
    pub user: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub exposed_ports: HashMap<String, HashMap<(), ()>>,
    pub tty: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub args_escaped: bool,
    pub image: String,
    pub volumes: HashMap<String, HashMap<(), ()>>,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub network_disabled: bool,
    pub mac_address: String,
    pub on_build: Vec<String>,
    pub labels: HashMap<String, String>,
    pub stop_signal: String,
    pub stop_timeout: i64,
    pub shell: Vec<String>,
}

impl ImageConfig {
    pub fn from_bollard(ic: BollardImageConfig) -> Self {
        ImageConfig {
            hostname: ic.hostname.unwrap_or_default(),
            domainname: ic.domainname.unwrap_or_default(),
            user: ic.user.unwrap_or_default(),
            attach_stdin: ic.attach_stdin.unwrap_or_default(),
            attach_stdout: ic.attach_stdout.unwrap_or_default(),
            attach_stderr: ic.attach_stderr.unwrap_or_default(),
            exposed_ports: ic.exposed_ports.unwrap_or_default(),
            tty: ic.tty.unwrap_or_default(),
            open_stdin: ic.open_stdin.unwrap_or_default(),
            stdin_once: ic.stdin_once.unwrap_or_default(),
            env: ic.env.unwrap_or_default(),
            cmd: ic.cmd.unwrap_or_default(),
            args_escaped: ic.args_escaped.unwrap_or_default(),
            image: ic.image.unwrap_or_default(),
            volumes: ic.volumes.unwrap_or_default(),
            working_dir: ic.working_dir.unwrap_or_default(),
            entrypoint: ic.entrypoint.unwrap_or_default(),
            network_disabled: ic.network_disabled.unwrap_or_default(),
            mac_address: ic.mac_address.unwrap_or_default(),
            on_build: ic.on_build.unwrap_or_default(),
            labels: ic.labels.unwrap_or_default(),
            stop_signal: ic.stop_signal.unwrap_or_default(),
            stop_timeout: ic.stop_timeout.unwrap_or_default(),
            shell: ic.shell.unwrap_or_default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct ImageInspectInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub parent: String,
    pub comment: String,
    pub created: String,
    pub docker_version: String,
    pub author: String,
    pub config: ImageConfig,
    pub architecture: String,
    pub variant: String,
    pub os: String,
    pub os_version: String,
    pub size: i64,
    pub virtual_size: i64,
    pub graph_driver: (String, HashMap<String, String>),
    pub root_fs: (String, Vec<String>),
    pub metadata: String,
}

impl ImageInspectInfo {
    pub fn from_image_inspect(ii: ImageInspect) -> Self {
        ImageInspectInfo {
            id: ii.id.unwrap_or_default(),
            repo_tags: ii.repo_tags.unwrap_or_default(),
            repo_digests: ii.repo_digests.unwrap_or_default(),
            parent: ii.parent.unwrap_or_default(),
            comment: ii.comment.unwrap_or_default(),
            created: ii.created.unwrap_or_default().to_rfc2822(),
            docker_version: ii.docker_version.unwrap_or_default(),
            author: ii.author.unwrap_or_default(),
            config: ImageConfig::from_bollard(ii.config.unwrap_or_default()),
            architecture: ii.architecture.unwrap_or_default(),
            variant: ii.variant.unwrap_or_default(),
            os: ii.os.unwrap_or_default(),
            os_version: ii.os_version.unwrap_or_default(),
            size: ii.size.unwrap_or_default(),
            virtual_size: ii.virtual_size.unwrap_or_default(),
            graph_driver: (
                ii.graph_driver.clone().unwrap_or_default().name,
                ii.graph_driver.clone().unwrap_or_default().data,
            ),
            root_fs: (
                ii.root_fs.clone().unwrap_or_default().typ,
                ii.root_fs
                    .clone()
                    .unwrap_or_default()
                    .layers
                    .unwrap_or_default(),
            ),
            metadata: ii
                .metadata
                .unwrap_or_default()
                .last_tag_time
                .unwrap()
                .to_rfc2822(),
        }
    }
}

#[derive(Default, Debug)]
pub struct ImageOciDescriptor {
    pub media_type: String,
    pub digest: String,
    pub size: i64,
}

#[derive(Default, Debug)]
pub struct ImageOciPlatform {
    pub architecture: String,
    pub os: String,
    pub os_version: String,
    pub os_features: Vec<String>,
    pub variant: String,
}

#[derive(Default, Debug)]
pub struct ImageDistributionInspect {
    pub descriptor: ImageOciDescriptor,
    pub platforms: Vec<ImageOciPlatform>,
}

impl ImageDistributionInspect {
    pub fn from_bollard(di: DistributionInspect) -> Self {
        Self {
            descriptor: ImageOciDescriptor {
                media_type: di.descriptor.media_type.unwrap_or_default(),
                digest: di.descriptor.digest.unwrap_or_default(),
                size: di.descriptor.size.unwrap_or_default(),
            },
            platforms: di
                .platforms
                .iter()
                .map(|p| ImageOciPlatform {
                    architecture: p.architecture.clone().unwrap_or_default(),
                    os: p.os.clone().unwrap_or_default(),
                    os_version: p.os_version.clone().unwrap_or_default(),
                    os_features: p.os_features.clone().unwrap_or_default(),
                    variant: p.variant.clone().unwrap_or_default(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub ports: Vec<Port>,
    pub ports_string: String,
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
            ports_string: cs
                .ports
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|p| {
                    format!(
                        "{}:{}",
                        p.public_port
                            .map_or_else(|| "".to_string(), |e| e.to_string()),
                        p.private_port.to_string()
                    )
                })
                .collect::<Vec<String>>()
                .join(", "),
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
        let containers = net.containers.unwrap_or_default();
        let peers = net.peers.unwrap_or_default();
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
            n_peers,
        }
    }
}
