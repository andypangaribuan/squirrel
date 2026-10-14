/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Inactive,
    Connecting,
    Connected,
    Disconnected,
}

impl TunnelStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            TunnelStatus::Inactive => "inactive",
            TunnelStatus::Connecting => "connecting",
            TunnelStatus::Connected => "connected",
            TunnelStatus::Disconnected => "disconnected",
        }
    }
}

fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortOrStr {
        Num(u16),
        Str(String),
    }

    match PortOrStr::deserialize(deserializer)? {
        PortOrStr::Num(n) => Ok(n),
        PortOrStr::Str(s) => s.trim().parse::<u16>().map_err(serde::de::Error::custom),
    }
}

fn deserialize_port_optional<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortOrStr {
        Num(u16),
        Str(String),
    }

    match Option::<PortOrStr>::deserialize(deserializer)? {
        Some(PortOrStr::Num(n)) => Ok(n),
        Some(PortOrStr::Str(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() { Ok(0) } else { trimmed.parse::<u16>().map_err(serde::de::Error::custom) }
        }
        None => Ok(0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelItem {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(rename = "type", alias = "tunnel_type", alias = "tunnel-type")]
    pub tunnel_type: String,

    pub name: String,

    #[serde(default)]
    pub description: String,

    // SSH tunnel specific
    #[serde(default)]
    pub host: Option<String>,

    #[serde(default = "default_local_bind", rename = "local-bind", alias = "local_bind")]
    pub local_bind: String,

    #[serde(deserialize_with = "deserialize_port", rename = "local-port", alias = "local_port")]
    pub local_port: u16,

    #[serde(default = "default_remote_bind", rename = "remote-bind", alias = "remote_bind")]
    pub remote_bind: String,

    #[serde(default, deserialize_with = "deserialize_port_optional", rename = "remote-port", alias = "remote_port")]
    pub remote_port: u16,

    // Docker / gcloud specific
    #[serde(default)]
    pub project: Option<String>,

    #[serde(default)]
    pub zone: Option<String>,

    #[serde(default)]
    pub instance: Option<String>,

    #[serde(
        default,
        rename = "service-account-key",
        alias = "service_account_key",
        alias = "service-account",
        alias = "sa_key",
        alias = "sa-key"
    )]
    pub service_account_key: Option<String>,

    #[serde(default)]
    pub image: Option<String>,
}

fn default_local_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_remote_bind() -> String {
    "127.0.0.1".to_string()
}

impl TunnelItem {
    pub fn item_id(&self) -> &str {
        if let Some(ref id) = self.id {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        &self.name
    }

    pub fn remote_port(&self) -> u16 {
        if self.remote_port > 0 { self.remote_port } else { self.local_port }
    }

    #[allow(dead_code)]
    pub fn is_ssh(&self) -> bool {
        let t = self.tunnel_type.to_lowercase();
        t == "ssh" || t == "ssh-tunnel" || t.contains("ssh")
    }

    pub fn is_cloud_sql_proxy(&self) -> bool {
        let t = self.tunnel_type.to_lowercase();
        t == "cloud-sql-proxy"
            || t == "cloudsql-proxy"
            || t == "docker-cloud-sql-proxy"
            || t == "docker-cloudsql-proxy"
            || t.contains("cloud-sql")
            || t.contains("cloudsql")
    }

    pub fn is_gcloud(&self) -> bool {
        let t = self.tunnel_type.to_lowercase();
        !self.is_cloud_sql_proxy() && (t.contains("gcloud") || t.contains("google-cloud") || t.contains("iap"))
    }

    pub fn is_docker(&self) -> bool {
        self.is_cloud_sql_proxy() || self.is_gcloud()
    }

    pub fn docker_image(&self) -> String {
        if let Some(ref img) = self.image {
            let trimmed = img.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if self.is_cloud_sql_proxy() {
            "gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.26.0".to_string()
        } else {
            "google/cloud-sdk:alpine".to_string()
        }
    }

    pub fn safe_name(&self) -> String {
        self.item_id()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.item_id().to_lowercase().contains(&q)
            || self.name.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
            || self.tunnel_type.to_lowercase().contains(&q)
            || self.host.as_deref().unwrap_or("").to_lowercase().contains(&q)
            || self.instance.as_deref().unwrap_or("").to_lowercase().contains(&q)
            || self.project.as_deref().unwrap_or("").to_lowercase().contains(&q)
            || self.local_bind.to_lowercase().contains(&q)
            || self.remote_bind.to_lowercase().contains(&q)
            || self.local_port.to_string().contains(&q)
            || self.remote_port.to_string().contains(&q)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRecord {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub worker_pid: u32,
    #[serde(default)]
    pub tunnel_pid: Option<u32>,
    pub status: TunnelStatus,
    #[serde(default)]
    pub updated_at: u64,
    #[serde(default)]
    pub connected_at: Option<u64>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub retry_count: u32,
}

pub fn format_duration(connected_at: Option<u64>, now: u64) -> String {
    let Some(ts) = connected_at else {
        return "-".to_string();
    };
    if now < ts {
        return "0 seconds".to_string();
    }
    let diff = now - ts;
    if diff < 60 {
        if diff == 1 { "1 second".to_string() } else { format!("{} seconds", diff) }
    } else if diff < 3600 {
        let mins = diff / 60;
        if mins == 1 { "1 minute".to_string() } else { format!("{} minutes", mins) }
    } else if diff < 86400 {
        let hours = diff / 3600;
        if hours == 1 { "1 hour".to_string() } else { format!("{} hours", hours) }
    } else {
        let days = diff / 86400;
        if days == 1 { "1 day".to_string() } else { format!("{} days", days) }
    }
}

pub fn get_home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(stripped) = p.strip_prefix("~") { get_home_dir().join(stripped) } else { p.to_path_buf() }
}

pub fn tunnel_dir() -> PathBuf {
    get_home_dir().join(".squirrel").join("tunnel")
}

pub fn ssh_config_path() -> PathBuf {
    tunnel_dir().join("ssh-keys.config")
}

pub fn items_path() -> PathBuf {
    tunnel_dir().join("items.json")
}

pub fn active_path() -> PathBuf {
    tunnel_dir().join("active.json")
}

pub fn logs_dir() -> PathBuf {
    tunnel_dir().join("logs")
}

pub const MAX_LOG_LINES: usize = 10_000;
pub const TRIM_LOG_LINES: usize = 5_000;

pub fn trim_log_file<P: AsRef<Path>>(path: P, max_lines: usize, keep_lines: usize) -> std::io::Result<bool> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(false);
    }

    use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
    let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;

    let mut reader = BufReader::new(&file);
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut buf = Vec::new();
    while reader.read_until(b'\n', &mut buf)? > 0 {
        lines.push(std::mem::take(&mut buf));
    }

    if lines.len() <= max_lines {
        return Ok(false);
    }

    let start_idx = lines.len().saturating_sub(keep_lines);
    let trimmed = &lines[start_idx..];

    file.seek(SeekFrom::Start(0))?;
    let mut writer = std::io::BufWriter::new(file);
    for line in trimmed {
        writer.write_all(line)?;
    }
    let mut file = writer.into_inner().map_err(|e| e.into_error())?;

    let new_len = file.stream_position()?;
    file.set_len(new_len)?;
    file.flush()?;

    Ok(true)
}

pub fn trim_all_log_files(max_lines: usize, keep_lines: usize) -> std::io::Result<usize> {
    let dir = logs_dir();
    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("log")
                && let Ok(true) = trim_log_file(&path, max_lines, keep_lines)
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn is_pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let res = unsafe { libc::kill(pid as i32, 0) };
    if res == 0 { true } else { std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) }
}

pub fn ensure_dirs_and_templates() -> Result<(), String> {
    let dir = tunnel_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;

    let ldir = logs_dir();
    fs::create_dir_all(&ldir).map_err(|e| format!("Failed to create logs directory {}: {}", ldir.display(), e))?;
    let _ = trim_all_log_files(MAX_LOG_LINES, TRIM_LOG_LINES);

    let ssh_cfg = ssh_config_path();
    if !ssh_cfg.exists() {
        let content = r#"# Squirrel SSH Tunnel Configuration
# Add your SSH host configs here. For example:
#
# Host remote-server
#     User super-user
#     Hostname remote-server.open.com
#     IdentityFile ~/.ssh/super.ssh.key
#     IdentitiesOnly yes
#     ProxyCommand /opt/homebrew/bin/cloudflared access ssh --hostname %h
"#;
        let _ = fs::write(&ssh_cfg, content);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&ssh_cfg, fs::Permissions::from_mode(0o600));
        }
    }

    let items_file = items_path();
    if !items_file.exists() {
        let sample_items = r#"[
  {
    "id": "db_gate_main_01",
    "type": "ssh-tunnel",
    "name": "db gate main",
    "description": "database main on gateway server",
    "host": "remote-server",
    "local-bind": "127.0.0.1",
    "local-port": 5002,
    "remote-bind": "192.168.1.10",
    "remote-port": 5432
  }
]
"#;
        let _ = fs::write(&items_file, sample_items);
    }

    let active_file = active_path();
    if !active_file.exists() {
        let _ = fs::write(&active_file, "{}\n");
    }

    Ok(())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ItemsFileFormat {
    List(Vec<TunnelItem>),
    Single(Box<TunnelItem>),
    Wrapped { items: Vec<TunnelItem> },
}

pub fn load_items() -> Result<Vec<TunnelItem>, String> {
    ensure_dirs_and_templates()?;
    let path = items_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "[]" || trimmed == "{}" {
        return Ok(Vec::new());
    }

    match serde_json::from_str::<ItemsFileFormat>(trimmed) {
        Ok(ItemsFileFormat::List(list)) => Ok(list),
        Ok(ItemsFileFormat::Single(item)) => Ok(vec![*item]),
        Ok(ItemsFileFormat::Wrapped { items }) => Ok(items),
        Err(e) => Err(format!("Failed to parse {}: {}", path.display(), e)),
    }
}

pub fn active_lock_path() -> PathBuf {
    tunnel_dir().join(".active.lock")
}

#[cfg(unix)]
struct FileLock {
    file: fs::File,
}

#[cfg(unix)]
impl FileLock {
    fn acquire_exclusive(path: &Path) -> Result<Self, String> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|e| format!("Failed to open lock file {}: {}", path.display(), e))?;

        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if ret != 0 {
            return Err(format!("Failed to acquire exclusive lock on {}: {}", path.display(), std::io::Error::last_os_error()));
        }
        Ok(Self { file })
    }

    fn acquire_shared(path: &Path) -> Result<Self, String> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|e| format!("Failed to open lock file {}: {}", path.display(), e))?;

        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_SH) };
        if ret != 0 {
            return Err(format!("Failed to acquire shared lock on {}: {}", path.display(), std::io::Error::last_os_error()));
        }
        Ok(Self { file })
    }
}

#[cfg(unix)]
impl Drop for FileLock {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        let fd = self.file.as_raw_fd();
        unsafe {
            libc::flock(fd, libc::LOCK_UN);
        }
    }
}

#[cfg(not(unix))]
struct FileLock;
#[cfg(not(unix))]
impl FileLock {
    fn acquire_exclusive(_path: &Path) -> Result<Self, String> {
        Ok(Self)
    }
    fn acquire_shared(_path: &Path) -> Result<Self, String> {
        Ok(Self)
    }
}

fn load_active_unlocked() -> Result<BTreeMap<String, ActiveRecord>, String> {
    ensure_dirs_and_templates()?;
    let path = active_path();
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Ok(BTreeMap::new()),
    };

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(BTreeMap::new());
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ActiveFormat {
        Map(BTreeMap<String, ActiveRecord>),
        List(Vec<ActiveRecord>),
    }

    let records: BTreeMap<String, ActiveRecord> = match serde_json::from_str::<ActiveFormat>(trimmed) {
        Ok(ActiveFormat::Map(m)) => m,
        Ok(ActiveFormat::List(l)) => l
            .into_iter()
            .map(|r| {
                let key = if !r.id.is_empty() { r.id.clone() } else { r.name.clone() };
                (key, r)
            })
            .collect(),
        Err(_) => BTreeMap::new(),
    };

    Ok(records)
}

fn save_active_unlocked(records: &BTreeMap<String, ActiveRecord>) -> Result<(), String> {
    ensure_dirs_and_templates()?;
    let path = active_path();
    let mut json = serde_json::to_string_pretty(records).map_err(|e| format!("Failed to serialize active records: {}", e))?;
    json.push('\n');

    // Skip disk write if file content is already identical
    if let Ok(existing) = fs::read_to_string(&path)
        && existing == json
    {
        return Ok(());
    }

    let tmp_path = tunnel_dir().join(format!(".active.json.tmp.{}", std::process::id()));
    fs::write(&tmp_path, &json).map_err(|e| format!("Failed to write {}: {}", tmp_path.display(), e))?;
    fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to rename {} to {}: {}", tmp_path.display(), path.display(), e))?;
    Ok(())
}

pub fn modify_active<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut BTreeMap<String, ActiveRecord>) -> R,
{
    ensure_dirs_and_templates()?;
    let _lock = FileLock::acquire_exclusive(&active_lock_path())?;
    let mut records = load_active_unlocked().unwrap_or_default();
    let res = f(&mut records);
    save_active_unlocked(&records)?;
    Ok(res)
}

pub fn load_active() -> Result<BTreeMap<String, ActiveRecord>, String> {
    ensure_dirs_and_templates()?;
    let _lock = FileLock::acquire_shared(&active_lock_path()).ok();
    load_active_unlocked()
}

#[allow(dead_code)]
pub fn save_active(records: &BTreeMap<String, ActiveRecord>) -> Result<(), String> {
    modify_active(|cur| {
        *cur = records.clone();
    })
}

pub fn get_item_status(id_or_name: &str, active: &BTreeMap<String, ActiveRecord>) -> (TunnelStatus, Option<ActiveRecord>) {
    let rec = active
        .get(id_or_name)
        .or_else(|| active.values().find(|r| r.id == id_or_name))
        .or_else(|| active.values().find(|r| r.name == id_or_name));
    if let Some(record) = rec {
        if is_pid_alive(record.worker_pid) { (record.status, Some(record.clone())) } else { (TunnelStatus::Inactive, None) }
    } else {
        (TunnelStatus::Inactive, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_example_items_json() {
        let json = r#"{
  "type": "ssh-tunnel",
  "name": "db gate main",
  "description": "database main on gateway server",
  "host": "dc-gateway",
  "local-bind": "127.0.0.1",
  "local-port": 35310,
  "remote-bind": "192.168.1.10",
  "remote-port": 5432
}"#;
        let parsed: ItemsFileFormat = serde_json::from_str(json).expect("Failed to parse user example");
        let items = match parsed {
            ItemsFileFormat::Single(i) => vec![*i],
            ItemsFileFormat::List(l) => l,
            ItemsFileFormat::Wrapped { items } => items,
        };
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.name, "db gate main");
        assert_eq!(item.item_id(), "db gate main");
        assert_eq!(item.tunnel_type, "ssh-tunnel");
        assert_eq!(item.host, Some("dc-gateway".to_string()));
        assert_eq!(item.local_bind, "127.0.0.1");
        assert_eq!(item.local_port, 35310);
        assert_eq!(item.remote_bind, "192.168.1.10");
        assert_eq!(item.remote_port, 5432);
        assert!(item.is_ssh());
        assert_eq!(item.safe_name(), "db-gate-main");
    }

    #[test]
    fn test_parse_item_with_id() {
        let json = r#"{
  "id": "db_v3_id_stg_01",
  "type": "ssh-tunnel",
  "name": "v3-db id",
  "description": "v3 database staging for indonesia",
  "host": "dc-gateway",
  "local-bind": "127.0.0.1",
  "local-port": 35511,
  "remote-bind": "id3-stg",
  "remote-port": 6541
}"#;
        let parsed: ItemsFileFormat = serde_json::from_str(json).expect("Failed to parse item with id");
        let items = match parsed {
            ItemsFileFormat::Single(i) => vec![*i],
            _ => panic!("Expected single item"),
        };
        assert_eq!(items[0].id, Some("db_v3_id_stg_01".to_string()));
        assert_eq!(items[0].item_id(), "db_v3_id_stg_01");
        assert_eq!(items[0].safe_name(), "db_v3_id_stg_01");
    }

    #[test]
    fn test_parse_items_array_and_ports_as_strings() {
        let json = r#"[
  {
    "type": "ssh-tunnel",
    "name": "db gate 1",
    "host": "dc-gateway",
    "local_bind": "127.0.0.1",
    "local_port": "35310",
    "remote_bind": "192.168.1.10",
    "remote_port": "5432"
  },
  {
    "id": "gcp_tunnel_01",
    "type": "docker-gcloud-tunnel",
    "name": "gcp tunnel",
    "project": "my-proj",
    "zone": "asia-southeast2-a",
    "instance": "bastion",
    "service_account_key": "~/.ssh/sa.json",
    "local-port": 3306,
    "remote-port": 3306
  }
]"#;
        let parsed: ItemsFileFormat = serde_json::from_str(json).expect("Failed to parse array");
        let items = match parsed {
            ItemsFileFormat::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].local_port, 35310);
        assert_eq!(items[1].local_port, 3306);
        assert_eq!(items[1].item_id(), "gcp_tunnel_01");
        assert!(items[1].is_gcloud());
        assert_eq!(items[1].safe_name(), "gcp_tunnel_01");
    }

    #[test]
    fn test_active_json_roundtrip_with_id_key() {
        let mut map = BTreeMap::new();
        map.insert(
            "db_v3_id_stg_01".to_string(),
            ActiveRecord {
                id: "db_v3_id_stg_01".to_string(),
                name: "v3-db id".to_string(),
                worker_pid: 999999,
                tunnel_pid: Some(1000000),
                status: TunnelStatus::Connected,
                updated_at: 1728891234,
                connected_at: Some(1728891200),
                last_error: None,
                retry_count: 0,
            },
        );

        let json = serde_json::to_string(&map).unwrap();
        let deserialized: BTreeMap<String, ActiveRecord> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        let rec = deserialized.get("db_v3_id_stg_01").unwrap();
        assert_eq!(rec.id, "db_v3_id_stg_01");
        assert_eq!(rec.name, "v3-db id");
        assert_eq!(rec.status, TunnelStatus::Connected);
        assert_eq!(rec.status.as_str(), "connected");
        assert_eq!(rec.worker_pid, 999999);
        assert_eq!(rec.connected_at, Some(1728891200));
    }

    #[test]
    fn test_active_json_keys_are_sorted() {
        let mut map = BTreeMap::new();
        map.insert(
            "z_tunnel".to_string(),
            ActiveRecord {
                id: "z_tunnel".to_string(),
                name: "z tunnel".to_string(),
                worker_pid: 1,
                tunnel_pid: None,
                status: TunnelStatus::Connected,
                updated_at: 100,
                connected_at: Some(100),
                last_error: None,
                retry_count: 0,
            },
        );
        map.insert(
            "a_tunnel".to_string(),
            ActiveRecord {
                id: "a_tunnel".to_string(),
                name: "a tunnel".to_string(),
                worker_pid: 2,
                tunnel_pid: None,
                status: TunnelStatus::Connecting,
                updated_at: 100,
                connected_at: None,
                last_error: None,
                retry_count: 0,
            },
        );

        let json = serde_json::to_string_pretty(&map).unwrap();
        let pos_a = json.find("a_tunnel").unwrap();
        let pos_z = json.find("z_tunnel").unwrap();
        assert!(pos_a < pos_z, "Keys must be serialized in sorted alphabetical order");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(None, 1000), "-");
        assert_eq!(format_duration(Some(1000), 1000), "0 seconds");
        assert_eq!(format_duration(Some(1000), 1001), "1 second");
        assert_eq!(format_duration(Some(1000), 1010), "10 seconds");
        assert_eq!(format_duration(Some(1000), 1059), "59 seconds");
        assert_eq!(format_duration(Some(1000), 1060), "1 minute");
        assert_eq!(format_duration(Some(1000), 1120), "2 minutes");
        assert_eq!(format_duration(Some(1000), 4599), "59 minutes");
        assert_eq!(format_duration(Some(1000), 4600), "1 hour");
        assert_eq!(format_duration(Some(1000), 15400), "4 hours");
        assert_eq!(format_duration(Some(1000), 87399), "23 hours");
        assert_eq!(format_duration(Some(1000), 87400), "1 day");
        assert_eq!(format_duration(Some(1000), 173800), "2 days");
        assert_eq!(format_duration(Some(1000), 865000), "10 days");
    }

    #[test]
    fn test_parse_cloud_sql_proxy_item() {
        let json = r#"{
  "id": "prod_cloudsql",
  "type": "cloud-sql-proxy",
  "name": "prod cloud sql",
  "instance": "my-project:asia-southeast2:my-db",
  "service-account-key": "~/.ssh/sa.json",
  "local-port": 5432
}"#;
        let parsed: ItemsFileFormat = serde_json::from_str(json).expect("Failed to parse cloud-sql-proxy item");
        let items = match parsed {
            ItemsFileFormat::Single(i) => vec![*i],
            _ => panic!("Expected single item"),
        };
        let item = &items[0];
        assert!(item.is_cloud_sql_proxy());
        assert!(item.is_docker());
        assert!(!item.is_gcloud());
        assert_eq!(item.remote_port(), 5432);
        assert_eq!(item.docker_image(), "gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.26.0");
    }

    #[test]
    fn test_cloud_sql_proxy_custom_image() {
        let json = r#"{
  "id": "prod_cloudsql",
  "type": "cloud-sql-proxy",
  "name": "prod cloud sql",
  "instance": "my-project:asia-southeast2:my-db",
  "image": "gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.25.0",
  "local-port": 5432,
  "remote-port": 5433
}"#;
        let parsed: ItemsFileFormat = serde_json::from_str(json).expect("Failed to parse cloud-sql-proxy item");
        let items = match parsed {
            ItemsFileFormat::Single(i) => vec![*i],
            _ => panic!("Expected single item"),
        };
        let item = &items[0];
        assert_eq!(item.remote_port(), 5433);
        assert_eq!(item.docker_image(), "gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.25.0");
    }

    #[test]
    fn test_get_item_status_with_duplicate_name() {
        let mut active = BTreeMap::new();
        active.insert(
            "db_v2_id_prod_01_bastion".to_string(),
            ActiveRecord {
                id: "db_v2_id_prod_01_bastion".to_string(),
                name: "db2 prod main".to_string(),
                worker_pid: std::process::id(),
                tunnel_pid: None,
                status: TunnelStatus::Connected,
                updated_at: 1000,
                connected_at: Some(1000),
                last_error: None,
                retry_count: 0,
            },
        );

        // Querying the inactive item that has the same name
        let (status_direct, _) = get_item_status("db_v2_id_prod_01", &active);
        assert_eq!(status_direct, TunnelStatus::Inactive, "Item db_v2_id_prod_01 must be inactive");

        // Querying the active bastion item
        let (status_bastion, rec) = get_item_status("db_v2_id_prod_01_bastion", &active);
        assert_eq!(status_bastion, TunnelStatus::Connected, "Bastion item must be connected");
        assert_eq!(rec.unwrap().id, "db_v2_id_prod_01_bastion");
    }

    #[test]
    fn test_trim_log_file_when_exceeding_max_lines() {
        let temp_dir = std::env::temp_dir().join(format!("sq_test_logs_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let log_file = temp_dir.join("test_trim.log");

        // Write 15 lines
        let mut content = String::new();
        for i in 1..=15 {
            content.push_str(&format!("log line {}\n", i));
        }
        fs::write(&log_file, &content).unwrap();

        // Max 10 lines, trim to 5
        let trimmed = trim_log_file(&log_file, 10, 5).unwrap();
        assert!(trimmed, "File should have been trimmed");

        let after = fs::read_to_string(&log_file).unwrap();
        let lines: Vec<&str> = after.lines().collect();
        assert_eq!(lines.len(), 5, "Must have exactly 5 lines remaining");
        assert_eq!(lines[0], "log line 11", "Oldest lines must be removed, first line should be line 11");
        assert_eq!(lines[4], "log line 15", "Last line should be line 15");

        // Running trim again should do nothing since 5 <= 10
        let trimmed_again = trim_log_file(&log_file, 10, 5).unwrap();
        assert!(!trimmed_again, "File should not be trimmed again");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
