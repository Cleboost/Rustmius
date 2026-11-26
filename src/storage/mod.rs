pub mod gist_sync;
pub mod ssh_config;
pub mod store;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostSource {
    Manual,
    SshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    Password,
    PrivateKey { path: String },
    Agent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OsKind {
    #[default]
    Unknown,
    Linux,
    Debian,
    Ubuntu,
    Arch,
    Fedora,
    NixOS,
    Alpine,
    CentOs,
    RedHat,
    Rocky,
    Alma,
    Kali,
    Mint,
    OpenSuse,
    Gentoo,
    Manjaro,
    FreeBsd,
    MacOs,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: Uuid,
    pub label: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
    #[serde(default)]
    pub password: Option<String>,
    pub group_id: Option<Uuid>,
    pub source: HostSource,
    #[serde(default)]
    pub os: OsKind,
    pub updated_at: DateTime<Utc>,
}

impl Host {
    pub fn new(label: impl Into<String>, hostname: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            hostname: hostname.into(),
            port: 22,
            username: username.into(),
            auth: AuthMethod::Agent,
            password: None,
            group_id: None,
            source: HostSource::Manual,
            os: OsKind::Unknown,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vault {
    pub version: u32,
    pub hosts: Vec<Host>,
    pub groups: Vec<Group>,
    #[serde(default)]
    pub deleted_hosts: HashMap<Uuid, DateTime<Utc>>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            version: 1,
            hosts: Vec::new(),
            groups: Vec::new(),
            deleted_hosts: HashMap::new(),
        }
    }
}
