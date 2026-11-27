use crate::providers::ProviderId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const STORE_VERSION: u32 = 2;

fn default_true() -> bool {
  true
}

fn default_relay_hops() -> u8 {
  2
}

fn default_silent_sync_interval_mins() -> u16 {
  15
}

fn default_webdav_filename() -> String {
  "maam-sync.maam".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavConfig {
  #[serde(default)]
  pub url: String,
  #[serde(default)]
  pub username: String,
  #[serde(default)]
  pub password: String,
  #[serde(default = "default_webdav_filename")]
  pub filename: String,
}

impl Default for WebDavConfig {
  fn default() -> Self {
    Self {
      url: String::new(),
      username: String::new(),
      password: String::new(),
      filename: default_webdav_filename(),
    }
  }
}

pub fn default_global_shortcut() -> String {
  #[cfg(target_os = "macos")]
  {
    "CommandOrControl+Shift+M".to_string()
  }
  #[cfg(not(target_os = "macos"))]
  {
    "CommandOrControl+Shift+M".to_string()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
  #[serde(default = "default_true")]
  pub minimize_to_tray: bool,
  #[serde(default = "default_global_shortcut")]
  pub global_shortcut: String,
  #[serde(default)]
  pub auto_approve: bool,
  #[serde(default = "default_true")]
  pub isolate_same_proxy_keep_alive: bool,
  #[serde(default = "default_silent_sync_interval_mins")]
  pub silent_sync_interval_mins: u16,
  #[serde(default)]
  pub webdav: WebDavConfig,
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      minimize_to_tray: true,
      global_shortcut: default_global_shortcut(),
      auto_approve: false,
      isolate_same_proxy_keep_alive: true,
      silent_sync_interval_mins: default_silent_sync_interval_mins(),
      webdav: WebDavConfig::default(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
  pub provider_id: ProviderId,
  pub uuid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuotaDetail {
  #[serde(default)]
  pub plan_label: String,
  #[serde(default)]
  pub plan_value: String,
  #[serde(default)]
  pub plan_subtext: Option<String>,
  #[serde(default)]
  pub plan_used_percent: Option<u8>,
  #[serde(default)]
  pub extra_label: Option<String>,
  #[serde(default)]
  pub extra_value: Option<String>,
  #[serde(default)]
  pub extra_subtext: Option<String>,
  #[serde(default)]
  pub extra_used_percent: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthState {
  #[default]
  Unknown,
  Authenticated,
  LoggedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecord {
  pub provider_id: ProviderId,
  pub uuid: String,
  pub alias: String,
  pub created_at_ms: u64,
  #[serde(default)]
  pub auto_alias: bool,
  #[serde(default)]
  pub proxy_url: Option<String>,
  #[serde(default)]
  pub timezone: Option<String>,
  #[serde(default)]
  pub avatar_url: Option<String>,
  #[serde(default)]
  pub quota_text: Option<String>,
  #[serde(default)]
  pub quota_percent: Option<u8>,
  #[serde(default)]
  pub quota_detail: Option<QuotaDetail>,
  #[serde(default)]
  pub auth_state: AuthState,
  #[serde(default)]
  pub cookie_expires_at_ms: Option<u64>,
  #[serde(default)]
  pub cookie_session_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictItem {
  pub uuid: String,
  pub providers: Vec<ProviderId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupReplyStatus {
  Queued,
  Sending,
  Streaming,
  #[default]
  Done,
  NeedsAuth,
  Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSharedSkill {
  pub id: String,
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub content: String,
  #[serde(default = "default_true")]
  pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberRole {
  pub member: AccountKey,
  #[serde(default)]
  pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupWorkflowMode {
  #[default]
  Parallel,
  Review,
  A2aRelay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupThreadBinding {
  pub member: AccountKey,
  pub thread_url: String,
  #[serde(default)]
  pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAgentKind {
  OpenaiApi,
  LocalCli,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentRecord {
  pub id: String,
  pub kind: ExternalAgentKind,
  pub name: String,
  #[serde(default)]
  pub base_url: String,
  #[serde(default)]
  pub api_key: String,
  #[serde(default)]
  pub model: String,
  #[serde(default)]
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
  #[serde(default)]
  pub workdir: Option<String>,
  #[serde(default)]
  pub created_at_ms: u64,
}

impl ExternalAgentRecord {
  pub fn account_key(&self) -> AccountKey {
    AccountKey {
      provider_id: match self.kind {
        ExternalAgentKind::OpenaiApi => ProviderId::api(),
        ExternalAgentKind::LocalCli => ProviderId::cli(),
      },
      uuid: self.id.clone(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupChatMessage {
  pub id: String,
  pub role: String,
  #[serde(default)]
  pub sender: Option<AccountKey>,
  #[serde(default)]
  pub sender_name: String,
  #[serde(default)]
  pub content: String,
  #[serde(default)]
  pub created_at_ms: u64,
  #[serde(default)]
  pub status: GroupReplyStatus,
  #[serde(default)]
  pub targets: Vec<AccountKey>,
  #[serde(default)]
  pub error: Option<String>,
  #[serde(default)]
  pub stage_label: Option<String>,
  #[serde(default)]
  pub hop_depth: u8,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub auto_routed_channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupChatRecord {
  pub id: String,
  pub name: String,
  #[serde(default)]
  pub members: Vec<AccountKey>,
  #[serde(default)]
  pub created_at_ms: u64,
  #[serde(default)]
  pub messages: Vec<GroupChatMessage>,
  #[serde(default)]
  pub shared_context: String,
  #[serde(default)]
  pub shared_skills: Vec<GroupSharedSkill>,
  #[serde(default)]
  pub member_roles: Vec<GroupMemberRole>,
  #[serde(default)]
  pub workflow_mode: GroupWorkflowMode,
  #[serde(default = "default_relay_hops")]
  pub max_relay_hops: u8,
  #[serde(default)]
  pub thread_bindings: Vec<GroupThreadBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountsStoreFile {
  pub version: u32,
  pub accounts: Vec<AccountRecord>,
  pub last_committed: Option<AccountKey>,
  pub pending_cleanup: Vec<AccountKey>,
  #[serde(default)]
  pub group_chats: Vec<GroupChatRecord>,
  #[serde(default)]
  pub external_agents: Vec<ExternalAgentRecord>,
  #[serde(default)]
  pub settings: AppSettings,
}

impl Default for AccountsStoreFile {
  fn default() -> Self {
    Self {
      version: STORE_VERSION,
      accounts: Vec::new(),
      last_committed: None,
      pending_cleanup: Vec::new(),
      group_chats: Vec::new(),
      external_agents: Vec::new(),
      settings: AppSettings::default(),
    }
  }
}

pub struct LoadedAccountsStore {
  pub store: AccountsStoreFile,
  pub conflicts: Vec<ConflictItem>,
  pub startup_notice: Option<String>,
}

pub fn now_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_millis() as u64)
    .unwrap_or(0)
}

pub fn normalize_uuid(raw: &str) -> Option<String> {
  Uuid::parse_str(raw.trim()).ok().map(|u| u.hyphenated().to_string())
}

pub fn detect_conflicts(accounts: &[AccountRecord]) -> Vec<ConflictItem> {
  let mut by_uuid: BTreeMap<String, Vec<ProviderId>> = BTreeMap::new();
  for acc in accounts {
    let entry = by_uuid.entry(acc.uuid.clone()).or_default();
    if !entry.contains(&acc.provider_id) {
      entry.push(acc.provider_id.clone());
    }
  }
  let mut count_by_uuid: BTreeMap<String, usize> = BTreeMap::new();
  for acc in accounts {
    *count_by_uuid.entry(acc.uuid.clone()).or_insert(0) += 1;
  }
  let mut conflicts = Vec::new();
  for (uuid, providers) in by_uuid {
    let total = count_by_uuid.get(&uuid).copied().unwrap_or(0);
    if providers.len() > 1 || total > 1 {
      conflicts.push(ConflictItem { uuid, providers });
    }
  }
  conflicts
}

fn parse_key_or_legacy_muse(val: &Value) -> Option<AccountKey> {
  if let Some(raw_uuid) = val.as_str() {
    return normalize_uuid(raw_uuid).map(|uuid| AccountKey {
      provider_id: ProviderId::muse(),
      uuid,
    });
  }
  let obj = val.as_object()?;
  let raw_uuid = obj
    .get("uuid")
    .or_else(|| obj.get("accountUuid"))
    .or_else(|| obj.get("id"))
    .and_then(|v| v.as_str())?;
  let uuid = normalize_uuid(raw_uuid)?;
  let provider_id = match obj.get("providerId").or_else(|| obj.get("provider_id")) {
    Some(v) => ProviderId::parse(v.as_str()?)?,
    None => ProviderId::muse(),
  };
  Some(AccountKey { provider_id, uuid })
}

fn migrate_from_value(root: Value) -> Result<(AccountsStoreFile, bool), String> {
  let obj = root
    .as_object()
    .ok_or_else(|| "accounts.json 根节点不是对象".to_string())?;

  let file_version = obj
    .get("version")
    .and_then(|v| v.as_u64())
    .map(|v| v as u32)
    .unwrap_or(1);

  let mut dirty = file_version < STORE_VERSION;
  let is_legacy = dirty;

  if let Some(v) = obj
    .get("customProviders")
    .or_else(|| obj.get("custom_providers"))
  {
    if !v.is_null() {
      dirty = true;
    }
  }

  let mut accounts = Vec::new();

  if let Some(arr) = obj.get("accounts").and_then(|v| v.as_array()) {
    for item in arr {
      let item_obj = item
        .as_object()
        .ok_or_else(|| "账号条目格式错误".to_string())?;
      let raw_uuid = item_obj
        .get("uuid")
        .or_else(|| item_obj.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "账号缺少 UUID".to_string())?;
      let uuid = normalize_uuid(raw_uuid).ok_or_else(|| "账号 UUID 非法".to_string())?;
      let provider_id = match item_obj.get("providerId").or_else(|| item_obj.get("provider_id")) {
        Some(v) => {
          let s = v.as_str().ok_or_else(|| "providerId 非法".to_string())?;
          if s.trim().to_ascii_lowercase().starts_with("custom-") {
            dirty = true;
            continue;
          }
          ProviderId::parse(s).ok_or_else(|| format!("未知服务标识: {s}"))?
        }
        None if is_legacy => ProviderId::muse(),
        None => return Err("新版配置账号缺少 providerId".to_string()),
      };
      if !provider_id.is_builtin() {
        dirty = true;
        continue;
      }
      let alias = item_obj
        .get("alias")
        .or_else(|| item_obj.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("账号 {}", &uuid[..8]));
      let created_at_ms = item_obj
        .get("createdAtMs")
        .or_else(|| item_obj.get("created_at_ms"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(now_ms);

      let auto_alias = item_obj
        .get("autoAlias")
        .or_else(|| item_obj.get("auto_alias"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      let proxy_url = item_obj
        .get("proxyUrl")
        .or_else(|| item_obj.get("proxy_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
      let timezone = item_obj
        .get("timezone")
        .or_else(|| item_obj.get("timeZone"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
      let avatar_url = item_obj
        .get("avatarUrl")
        .or_else(|| item_obj.get("avatar_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
      let auth_state = item_obj
        .get("authState")
        .or_else(|| item_obj.get("auth_state"))
        .and_then(|v| serde_json::from_value::<AuthState>(v.clone()).ok())
        .unwrap_or_default();
      let mut quota_detail = item_obj
        .get("quotaDetail")
        .or_else(|| item_obj.get("quota_detail"))
        .and_then(|v| serde_json::from_value::<QuotaDetail>(v.clone()).ok())
        .filter(|d| !d.plan_label.trim().is_empty() || !d.plan_value.trim().is_empty());
      let mut quota_text = item_obj
        .get("quotaText")
        .or_else(|| item_obj.get("quota_text"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());
      let mut quota_percent = item_obj
        .get("quotaPercent")
        .or_else(|| item_obj.get("quota_percent"))
        .and_then(|v| v.as_u64())
        .map(|n| n.min(100) as u8);
      if quota_detail.is_none() && quota_text.as_deref() == Some("剩余额度 100%") {
        quota_text = None;
        quota_percent = None;
      }
      let mut cookie_expires_at_ms = item_obj
        .get("cookieExpiresAtMs")
        .or_else(|| item_obj.get("cookie_expires_at_ms"))
        .and_then(|v| v.as_u64());
      let mut cookie_session_only = item_obj
        .get("cookieSessionOnly")
        .or_else(|| item_obj.get("cookie_session_only"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      if cookie_expires_at_ms.is_some() {
        cookie_session_only = false;
      }
      if auth_state == AuthState::LoggedOut {
        quota_detail = None;
        quota_text = None;
        quota_percent = None;
        cookie_expires_at_ms = None;
        cookie_session_only = false;
      }

      accounts.push(AccountRecord {
        provider_id,
        uuid,
        alias,
        created_at_ms,
        auto_alias,
        proxy_url,
        timezone,
        avatar_url,
        quota_text,
        quota_percent,
        quota_detail,
        auth_state,
        cookie_expires_at_ms,
        cookie_session_only,
      });
    }
  }

  let last_committed_raw = obj
    .get("lastCommitted")
    .or_else(|| obj.get("last_committed"))
    .or_else(|| obj.get("lastOpened"))
    .or_else(|| obj.get("last_opened"))
    .filter(|v| !v.is_null());
  let last_committed = last_committed_raw
    .and_then(parse_key_or_legacy_muse)
    .filter(|k| k.provider_id.is_builtin());
  if last_committed_raw.is_some() && last_committed.is_none() {
    dirty = true;
  }

  let mut pending_cleanup = Vec::new();
  if let Some(arr) = obj
    .get("pendingCleanup")
    .or_else(|| obj.get("pending_cleanup"))
    .and_then(|v| v.as_array())
  {
    for v in arr {
      if let Some(key) = parse_key_or_legacy_muse(v).filter(|k| k.provider_id.is_builtin()) {
        if !pending_cleanup.contains(&key) {
          pending_cleanup.push(key);
        }
      } else if !v.is_null() {
        dirty = true;
      }
    }
  }

  let mut settings = AppSettings::default();
  if let Some(settings_obj) = obj.get("settings").and_then(|v| v.as_object()) {
    if let Some(minimize) = settings_obj
      .get("minimizeToTray")
      .or_else(|| settings_obj.get("minimize_to_tray"))
      .and_then(|v| v.as_bool())
    {
      settings.minimize_to_tray = minimize;
    }
    if let Some(shortcut) = settings_obj
      .get("globalShortcut")
      .or_else(|| settings_obj.get("global_shortcut"))
      .and_then(|v| v.as_str())
    {
      settings.global_shortcut = shortcut.trim().to_string();
    }
    if let Some(auto_approve) = settings_obj
      .get("autoApprove")
      .or_else(|| settings_obj.get("auto_approve"))
      .and_then(|v| v.as_bool())
    {
      settings.auto_approve = auto_approve;
    }
    if let Some(isolate) = settings_obj
      .get("isolateSameProxyKeepAlive")
      .or_else(|| settings_obj.get("isolate_same_proxy_keep_alive"))
      .and_then(|v| v.as_bool())
    {
      settings.isolate_same_proxy_keep_alive = isolate;
    }
    if let Some(interval) = settings_obj
      .get("silentSyncIntervalMins")
      .or_else(|| settings_obj.get("silent_sync_interval_mins"))
      .and_then(|v| v.as_u64())
    {
      settings.silent_sync_interval_mins = interval.min(120) as u16;
    }
    if let Some(webdav_obj) = settings_obj.get("webdav").and_then(|v| v.as_object()) {
      if let Some(u) = webdav_obj.get("url").and_then(|v| v.as_str()) {
        settings.webdav.url = u.trim().to_string();
      }
      if let Some(u) = webdav_obj.get("username").and_then(|v| v.as_str()) {
        settings.webdav.username = u.trim().to_string();
      }
      if let Some(p) = webdav_obj.get("password").and_then(|v| v.as_str()) {
        settings.webdav.password = p.to_string();
      }
      if let Some(f) = webdav_obj
        .get("filename")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
      {
        settings.webdav.filename = f.to_string();
      }
    }
  }

  let mut external_agents = Vec::new();
  let mut seen_external_ids = HashSet::new();
  if let Some(ext_arr) = obj
    .get("externalAgents")
    .or_else(|| obj.get("external_agents"))
    .and_then(|v| v.as_array())
  {
    for ext_val in ext_arr {
      let Some(ext_obj) = ext_val.as_object() else {
        dirty = true;
        continue;
      };
      let Some(kind) = ext_obj
        .get("kind")
        .and_then(|v| serde_json::from_value::<ExternalAgentKind>(v.clone()).ok())
      else {
        dirty = true;
        continue;
      };
      let raw_ext_id = ext_obj
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(normalize_uuid);
      let ext_id = match raw_ext_id {
        Some(eid) if !seen_external_ids.contains(&eid) => eid,
        _ => {
          dirty = true;
          continue;
        }
      };
      let ext_name: String = ext_obj
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().chars().take(40).collect())
        .unwrap_or_default();
      if ext_name.is_empty() {
        dirty = true;
        continue;
      }
      seen_external_ids.insert(ext_id.clone());
      let base_url = ext_obj
        .get("baseUrl")
        .or_else(|| ext_obj.get("base_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
      let api_key = ext_obj
        .get("apiKey")
        .or_else(|| ext_obj.get("api_key"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
      let model = ext_obj
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().chars().take(120).collect())
        .unwrap_or_default();
      let command = ext_obj
        .get("command")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().chars().take(260).collect())
        .unwrap_or_default();
      let args = ext_obj
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .take(24)
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();
      let workdir = ext_obj
        .get("workdir")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
      let ext_created = ext_obj
        .get("createdAtMs")
        .or_else(|| ext_obj.get("created_at_ms"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(now_ms);
      external_agents.push(ExternalAgentRecord {
        id: ext_id,
        kind,
        name: ext_name,
        base_url,
        api_key,
        model,
        command,
        args,
        workdir,
        created_at_ms: ext_created,
      });
    }
  }

  let mut valid_account_keys: HashSet<(ProviderId, String)> = accounts
    .iter()
    .map(|a| (a.provider_id.clone(), a.uuid.clone()))
    .collect();
  for ext in &external_agents {
    let k = ext.account_key();
    valid_account_keys.insert((k.provider_id, k.uuid));
  }
  let mut group_chats = Vec::new();
  let mut seen_group_ids = HashSet::new();
  if let Some(arr) = obj
    .get("groupChats")
    .or_else(|| obj.get("group_chats"))
    .and_then(|v| v.as_array())
  {
    for item in arr {
      let Some(item_obj) = item.as_object() else {
        dirty = true;
        continue;
      };
      let raw_name = item_obj
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .unwrap_or("");
      if raw_name.is_empty() {
        dirty = true;
        continue;
      }
      let name: String = raw_name.chars().take(32).collect();
      let raw_id = item_obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
      let id = match raw_id {
        Some(existing) if !seen_group_ids.contains(&existing) => existing,
        _ => {
          dirty = true;
          Uuid::new_v4().hyphenated().to_string()
        }
      };
      seen_group_ids.insert(id.clone());

      let mut members = Vec::new();
      if let Some(m_arr) = item_obj.get("members").and_then(|v| v.as_array()) {
        for mv in m_arr {
          if let Some(key) = parse_key_or_legacy_muse(mv) {
            if valid_account_keys.contains(&(key.provider_id.clone(), key.uuid.clone()))
              && !members.contains(&key)
            {
              members.push(key);
            } else {
              dirty = true;
            }
          } else {
            dirty = true;
          }
        }
      }

      let created_at_ms = item_obj
        .get("createdAtMs")
        .or_else(|| item_obj.get("created_at_ms"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(now_ms);

      let mut messages = Vec::new();
      if let Some(msg_arr) = item_obj.get("messages").and_then(|v| v.as_array()) {
        for msg_val in msg_arr {
          let Some(msg_obj) = msg_val.as_object() else {
            dirty = true;
            continue;
          };
          let role = msg_obj
            .get("role")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| *s == "user" || *s == "assistant")
            .unwrap_or("assistant")
            .to_string();
          let msg_id = msg_obj
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());
          let sender = msg_obj
            .get("sender")
            .filter(|v| !v.is_null())
            .and_then(parse_key_or_legacy_muse);
          let sender_name = msg_obj
            .get("senderName")
            .or_else(|| msg_obj.get("sender_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
          let content = msg_obj
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
          let msg_created = msg_obj
            .get("createdAtMs")
            .or_else(|| msg_obj.get("created_at_ms"))
            .and_then(|v| v.as_u64())
            .unwrap_or(created_at_ms);
          let raw_status = msg_obj
            .get("status")
            .and_then(|v| serde_json::from_value::<GroupReplyStatus>(v.clone()).ok())
            .unwrap_or(GroupReplyStatus::Done);
          let mut error = msg_obj
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
          let status = match raw_status {
            GroupReplyStatus::Queued | GroupReplyStatus::Sending | GroupReplyStatus::Streaming => {
              dirty = true;
              if content.trim().is_empty() {
                error.get_or_insert_with(|| "上次应用退出时未完成生成".to_string());
                GroupReplyStatus::Error
              } else {
                GroupReplyStatus::Done
              }
            }
            other => other,
          };
          let mut targets = Vec::new();
          if let Some(t_arr) = msg_obj.get("targets").and_then(|v| v.as_array()) {
            for tv in t_arr {
              if let Some(k) = parse_key_or_legacy_muse(tv) {
                if !targets.contains(&k) {
                  targets.push(k);
                }
              }
            }
          }
          let stage_label = msg_obj
            .get("stageLabel")
            .or_else(|| msg_obj.get("stage_label"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
          let hop_depth = msg_obj
            .get("hopDepth")
            .or_else(|| msg_obj.get("hop_depth"))
            .and_then(|v| v.as_u64())
            .map(|n| n.min(8) as u8)
            .unwrap_or(0);
          let auto_routed_channel = msg_obj
            .get("autoRoutedChannel")
            .or_else(|| msg_obj.get("auto_routed_channel"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
          messages.push(GroupChatMessage {
            id: msg_id,
            role,
            sender,
            sender_name,
            content,
            created_at_ms: msg_created,
            status,
            targets,
            error,
            stage_label,
            hop_depth,
            auto_routed_channel,
          });
        }
      }
      if messages.len() > 120 {
        let excess = messages.len() - 120;
        messages.drain(..excess);
        dirty = true;
      }

      let shared_context: String = item_obj
        .get("sharedContext")
        .or_else(|| item_obj.get("shared_context"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().chars().take(8000).collect())
        .unwrap_or_default();

      let mut shared_skills = Vec::new();
      let mut seen_skill_ids = HashSet::new();
      if let Some(sk_arr) = item_obj
        .get("sharedSkills")
        .or_else(|| item_obj.get("shared_skills"))
        .and_then(|v| v.as_array())
      {
        for sk_val in sk_arr {
          if shared_skills.len() >= 30 {
            dirty = true;
            break;
          }
          let Some(sk_obj) = sk_val.as_object() else {
            dirty = true;
            continue;
          };
          let sk_name: String = sk_obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().chars().take(40).collect())
            .unwrap_or_default();
          let sk_content: String = sk_obj
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().chars().take(2000).collect())
            .unwrap_or_default();
          if sk_name.is_empty() || sk_content.is_empty() {
            dirty = true;
            continue;
          }
          let sk_desc: String = sk_obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().chars().take(120).collect())
            .unwrap_or_default();
          let sk_enabled = sk_obj
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
          let raw_sk_id = sk_obj
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
          let sk_id = match raw_sk_id {
            Some(existing) if !seen_skill_ids.contains(&existing) => existing,
            _ => {
              dirty = true;
              Uuid::new_v4().hyphenated().to_string()
            }
          };
          seen_skill_ids.insert(sk_id.clone());
          shared_skills.push(GroupSharedSkill {
            id: sk_id,
            name: sk_name,
            description: sk_desc,
            content: sk_content,
            enabled: sk_enabled,
          });
        }
      }

      let mut member_roles = Vec::new();
      if let Some(r_arr) = item_obj
        .get("memberRoles")
        .or_else(|| item_obj.get("member_roles"))
        .and_then(|v| v.as_array())
      {
        for r_val in r_arr {
          let Some(r_obj) = r_val.as_object() else {
            dirty = true;
            continue;
          };
          let Some(m_key) = r_obj.get("member").and_then(parse_key_or_legacy_muse) else {
            dirty = true;
            continue;
          };
          if !members.contains(&m_key)
            || member_roles
              .iter()
              .any(|mr: &GroupMemberRole| mr.member == m_key)
          {
            dirty = true;
            continue;
          }
          let role_str: String = r_obj
            .get("role")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().chars().take(120).collect())
            .unwrap_or_default();
          if !role_str.is_empty() {
            member_roles.push(GroupMemberRole {
              member: m_key,
              role: role_str,
            });
          }
        }
      }

      let workflow_mode = item_obj
        .get("workflowMode")
        .or_else(|| item_obj.get("workflow_mode"))
        .and_then(|v| serde_json::from_value::<GroupWorkflowMode>(v.clone()).ok())
        .unwrap_or_default();
      let max_relay_hops = item_obj
        .get("maxRelayHops")
        .or_else(|| item_obj.get("max_relay_hops"))
        .and_then(|v| v.as_u64())
        .map(|n| n.clamp(1, 3) as u8)
        .unwrap_or_else(default_relay_hops);

      let mut thread_bindings = Vec::new();
      if let Some(tb_arr) = item_obj
        .get("threadBindings")
        .or_else(|| item_obj.get("thread_bindings"))
        .and_then(|v| v.as_array())
      {
        for tb_val in tb_arr {
          let Some(tb_obj) = tb_val.as_object() else {
            dirty = true;
            continue;
          };
          let Some(m_key) = tb_obj.get("member").and_then(parse_key_or_legacy_muse) else {
            dirty = true;
            continue;
          };
          let thread_url = tb_obj
            .get("threadUrl")
            .or_else(|| tb_obj.get("thread_url"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| s.starts_with("https://") || s.starts_with("http://"))
            .unwrap_or_default();
          if thread_url.is_empty()
            || !members.contains(&m_key)
            || thread_bindings
              .iter()
              .any(|tb: &GroupThreadBinding| tb.member == m_key)
          {
            dirty = true;
            continue;
          }
          let updated_at_ms = tb_obj
            .get("updatedAtMs")
            .or_else(|| tb_obj.get("updated_at_ms"))
            .and_then(|v| v.as_u64())
            .unwrap_or_else(now_ms);
          thread_bindings.push(GroupThreadBinding {
            member: m_key,
            thread_url,
            updated_at_ms,
          });
        }
      }

      if let Some(mm) = item_obj
        .get("memberModels")
        .or_else(|| item_obj.get("member_models"))
      {
        if mm.as_array().is_some_and(|a| !a.is_empty()) {
          dirty = true;
        }
      }

      group_chats.push(GroupChatRecord {
        id,
        name,
        members,
        created_at_ms,
        messages,
        shared_context,
        shared_skills,
        member_roles,
        workflow_mode,
        max_relay_hops,
        thread_bindings,
      });
    }
  }

  Ok((
    AccountsStoreFile {
      version: STORE_VERSION,
      accounts,
      last_committed,
      pending_cleanup,
      group_chats,
      external_agents,
      settings,
    },
    dirty,
  ))
}

pub fn save_store_atomic(path: &Path, store: &AccountsStoreFile) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
  }
  let bytes = serde_json::to_vec_pretty(store).map_err(|e| format!("序列化配置失败: {e}"))?;
  let tmp_name = format!(
    ".accounts.json.tmp.{}",
    Uuid::new_v4().simple()
  );
  let tmp_path = path.with_file_name(tmp_name);

  {
    let mut file = File::create(&tmp_path).map_err(|e| format!("创建临时配置文件失败: {e}"))?;
    file
      .write_all(&bytes)
      .map_err(|e| format!("写入临时配置文件失败: {e}"))?;
    file
      .sync_all()
      .map_err(|e| format!("同步临时配置文件失败: {e}"))?;
  }

  fs::rename(&tmp_path, path).map_err(|e| {
    let _ = fs::remove_file(&tmp_path);
    format!("原子替换 accounts.json 失败: {e}")
  })
}

pub fn load_or_migrate_store(path: &Path) -> LoadedAccountsStore {
  if !path.exists() {
    return LoadedAccountsStore {
      store: AccountsStoreFile::default(),
      conflicts: Vec::new(),
      startup_notice: None,
    };
  }

  let raw = match fs::read_to_string(path) {
    Ok(content) => content,
    Err(err) => {
      return LoadedAccountsStore {
        store: AccountsStoreFile::default(),
        conflicts: Vec::new(),
        startup_notice: Some(format!("读取 accounts.json 失败，已回退管理页: {err}")),
      };
    }
  };

  let parsed: Value = match serde_json::from_str(&raw) {
    Ok(v) => v,
    Err(_) => {
      return LoadedAccountsStore {
        store: AccountsStoreFile::default(),
        conflicts: Vec::new(),
        startup_notice: Some("本地 accounts.json 配置损坏，已回退管理页".to_string()),
      };
    }
  };

  let (mut store, migrated) = match migrate_from_value(parsed) {
    Ok(res) => res,
    Err(reason) => {
      return LoadedAccountsStore {
        store: AccountsStoreFile::default(),
        conflicts: Vec::new(),
        startup_notice: Some(format!("解析 accounts.json 失败（{reason}），已回退管理页")),
      };
    }
  };

  let conflicts = detect_conflicts(&store.accounts);
  if !conflicts.is_empty() {
    store.last_committed = None;
    return LoadedAccountsStore {
      store,
      conflicts,
      startup_notice: Some("检测到账号 UUID 归属冲突，已停留在管理页等待处理".to_string()),
    };
  }

  if migrated {
    if let Err(err) = save_store_atomic(path, &store) {
      return LoadedAccountsStore {
        store,
        conflicts: Vec::new(),
        startup_notice: Some(format!("旧版 Muse 配置原子迁移写盘失败: {err}")),
      };
    }
  }

  LoadedAccountsStore {
    store,
    conflicts: Vec::new(),
    startup_notice: None,
  }
}

#[cfg(test)]
mod tests {
  use super::migrate_from_value;
  use serde_json::json;

  #[test]
  fn migrate_proxy_url_accepts_camel_snake_and_malformed_types() {
    let raw = json!({
      "version": 2,
      "accounts": [
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000001",
          "alias": "camel",
          "proxyUrl": " http://127.0.0.1:7890 "
        },
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000002",
          "alias": "snake",
          "proxy_url": "http://127.0.0.1:8888"
        },
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000003",
          "alias": "null_and_empty",
          "proxyUrl": null,
          "proxy_url": "   "
        },
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000004",
          "alias": "bad_type",
          "proxyUrl": 12345
        }
      ]
    });

    let (store, _) = migrate_from_value(raw).expect("migrate should tolerate malformed proxyUrl");
    assert_eq!(store.accounts.len(), 4);
    assert_eq!(store.accounts[0].proxy_url.as_deref(), Some("http://127.0.0.1:7890"));
    assert_eq!(store.accounts[1].proxy_url.as_deref(), Some("http://127.0.0.1:8888"));
    assert_eq!(store.accounts[2].proxy_url, None);
    assert_eq!(store.accounts[3].proxy_url, None);
    assert!(store.settings.minimize_to_tray);
    assert!(!store.settings.global_shortcut.is_empty());
  }

  #[test]
  fn migrate_custom_providers_settings_and_drops_orphans() {
    let raw = json!({
      "version": 2,
      "customProviders": [
        {
          "id": "custom-abcd1234",
          "name": "Gemini",
          "entryUrl": "https://gemini.google.com/"
        }
      ],
      "lastCommitted": {
        "providerId": "custom-abcd1234",
        "uuid": "00000000-0000-4000-8000-000000000010"
      },
      "settings": {
        "minimizeToTray": false,
        "globalShortcut": "Alt+Space"
      },
      "accounts": [
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000001",
          "alias": "m1"
        },
        {
          "providerId": "custom-abcd1234",
          "uuid": "00000000-0000-4000-8000-000000000010",
          "alias": "gemini1"
        },
        {
          "providerId": "custom-deleted99",
          "uuid": "00000000-0000-4000-8000-000000000011",
          "alias": "orphan"
        }
      ]
    });

    let (store, dirty) = migrate_from_value(raw).expect("legacy custom provider store should migrate cleanly");
    assert!(dirty);
    assert!(store.last_committed.is_none());
    assert_eq!(store.accounts.len(), 1);
    assert_eq!(store.accounts[0].alias, "m1");
    assert!(!store.settings.minimize_to_tray);
    assert_eq!(store.settings.global_shortcut, "Alt+Space");
  }

  #[test]
  fn migrate_cookie_expiry_supports_camel_snake_and_clears_logged_out() {
    let raw = json!({
      "version": 2,
      "accounts": [
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000021",
          "alias": "auth_camel",
          "authState": "authenticated",
          "cookieExpiresAtMs": 1800000000000u64,
          "cookieSessionOnly": false
        },
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000022",
          "alias": "session_snake",
          "auth_state": "authenticated",
          "cookie_session_only": true
        },
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000023",
          "alias": "logged_out_cleared",
          "authState": "logged_out",
          "cookieExpiresAtMs": 1800000000000u64,
          "cookieSessionOnly": true
        }
      ]
    });

    let (store, _) = migrate_from_value(raw).expect("migrate cookie expiry should succeed");
    assert_eq!(store.accounts.len(), 3);
    assert_eq!(store.accounts[0].cookie_expires_at_ms, Some(1_800_000_000_000));
    assert!(!store.accounts[0].cookie_session_only);
    assert_eq!(store.accounts[1].cookie_expires_at_ms, None);
    assert!(store.accounts[1].cookie_session_only);
    assert_eq!(store.accounts[2].cookie_expires_at_ms, None);
    assert!(!store.accounts[2].cookie_session_only);
  }

  #[test]
  fn migrate_group_chats_prunes_orphans_and_recovers_inflight_messages() {
    use super::GroupReplyStatus;

    let raw = json!({
      "version": 2,
      "accounts": [
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000031",
          "alias": "m1",
          "authState": "authenticated"
        }
      ],
      "groupChats": [
        {
          "id": "g1",
          "name": "研究群",
          "members": [
            { "providerId": "muse", "uuid": "00000000-0000-4000-8000-000000000031" },
            { "providerId": "muse", "uuid": "00000000-0000-4000-8000-000000000031" },
            { "providerId": "hark", "uuid": "00000000-0000-4000-8000-000000000099" }
          ],
          "sharedContext": "  使用简体中文回答  ",
          "sharedSkills": [
            {
              "id": "sk1",
              "name": "严谨校验",
              "description": "检查边界条件",
              "content": "给出结论前说明前置条件",
              "enabled": true
            }
          ],
          "memberRoles": [
            {
              "member": { "providerId": "muse", "uuid": "00000000-0000-4000-8000-000000000031" },
              "role": "主架构师"
            },
            {
              "member": { "providerId": "hark", "uuid": "00000000-0000-4000-8000-000000000099" },
              "role": "已移除成员角色"
            }
          ],
          "messages": [
            {
              "id": "m_stream_partial",
              "role": "assistant",
              "senderName": "m1",
              "content": "partial reply",
              "status": "streaming"
            },
            {
              "id": "m_sending_empty",
              "role": "assistant",
              "senderName": "m1",
              "content": "",
              "status": "sending"
            },
            {
              "id": "m_queued_empty",
              "role": "assistant",
              "senderName": "m1",
              "content": "",
              "status": "queued"
            }
          ]
        }
      ]
    });

    let (store, dirty) = migrate_from_value(raw).expect("migrate groupChats should succeed");
    assert!(dirty);
    assert_eq!(store.group_chats.len(), 1);
    let g = &store.group_chats[0];
    assert_eq!(g.members.len(), 1);
    assert_eq!(g.members[0].uuid, "00000000-0000-4000-8000-000000000031");
    assert_eq!(g.shared_context, "使用简体中文回答");
    assert_eq!(g.shared_skills.len(), 1);
    assert_eq!(g.shared_skills[0].name, "严谨校验");
    assert_eq!(g.member_roles.len(), 1);
    assert_eq!(g.member_roles[0].role, "主架构师");
    assert_eq!(g.messages.len(), 3);
    assert_eq!(g.messages[0].status, GroupReplyStatus::Done);
    assert_eq!(g.messages[1].status, GroupReplyStatus::Error);
    assert!(g.messages[1].error.is_some());
    assert_eq!(g.messages[2].status, GroupReplyStatus::Error);
  }

  #[test]
  fn migrate_external_agents_workflow_and_thread_bindings_roundtrip() {
    use super::{ExternalAgentKind, GroupWorkflowMode};

    let raw = json!({
      "version": 2,
      "accounts": [
        {
          "providerId": "muse",
          "uuid": "00000000-0000-4000-8000-000000000041",
          "alias": "m1",
          "authState": "authenticated"
        }
      ],
      "externalAgents": [
        {
          "id": "00000000-0000-4000-8000-000000000051",
          "kind": "openai_api",
          "name": "DeepSeek-R",
          "baseUrl": "https://api.deepseek.com/v1",
          "apiKey": "sk-test",
          "model": "deepseek-chat"
        },
        {
          "id": "00000000-0000-4000-8000-000000000052",
          "kind": "local_cli",
          "name": "ClaudeCLI",
          "command": "claude",
          "args": ["-p"],
          "workdir": "/tmp"
        },
        {
          "id": "invalid-id",
          "kind": "local_cli",
          "name": "BadAgent"
        }
      ],
      "groupChats": [
        {
          "id": "g_hybrid",
          "name": "混合协同群",
          "members": [
            { "providerId": "muse", "uuid": "00000000-0000-4000-8000-000000000041" },
            { "providerId": "api", "uuid": "00000000-0000-4000-8000-000000000051" },
            { "providerId": "cli", "uuid": "00000000-0000-4000-8000-000000000052" }
          ],
          "workflowMode": "review",
          "maxRelayHops": 9,
          "threadBindings": [
            {
              "member": { "providerId": "muse", "uuid": "00000000-0000-4000-8000-000000000041" },
              "threadUrl": "https://muse.ai/c/thread-123",
              "updatedAtMs": 12345
            },
            {
              "member": { "providerId": "hark", "uuid": "00000000-0000-4000-8000-000000000099" },
              "threadUrl": "https://app.hark.com/chat"
            }
          ],
          "messages": [
            {
              "id": "m1",
              "role": "assistant",
              "sender": { "providerId": "api", "uuid": "00000000-0000-4000-8000-000000000051" },
              "senderName": "DeepSeek-R",
              "content": "review ok",
              "stageLabel": "交叉评审",
              "hopDepth": 1
            }
          ]
        }
      ]
    });

    let (store, dirty) = migrate_from_value(raw).expect("migrate hybrid store should succeed");
    assert!(dirty);
    assert_eq!(store.external_agents.len(), 2);
    assert_eq!(store.external_agents[0].kind, ExternalAgentKind::OpenaiApi);
    assert_eq!(store.external_agents[1].kind, ExternalAgentKind::LocalCli);
    assert_eq!(store.group_chats.len(), 1);
    let g = &store.group_chats[0];
    assert_eq!(g.members.len(), 3);
    assert_eq!(g.workflow_mode, GroupWorkflowMode::Review);
    assert_eq!(g.max_relay_hops, 3);
    assert_eq!(g.thread_bindings.len(), 1);
    assert_eq!(g.thread_bindings[0].thread_url, "https://muse.ai/c/thread-123");
    assert_eq!(g.messages[0].stage_label.as_deref(), Some("交叉评审"));
    assert_eq!(g.messages[0].hop_depth, 1);

    let reserialized = serde_json::to_value(&store).expect("serialize store");
    let (roundtrip, dirty2) = migrate_from_value(reserialized).expect("roundtrip migrate");
    assert!(!dirty2);
    assert_eq!(roundtrip.external_agents, store.external_agents);
    assert_eq!(roundtrip.group_chats[0].workflow_mode, GroupWorkflowMode::Review);
    assert_eq!(roundtrip.group_chats[0].thread_bindings, g.thread_bindings);
    assert!(!roundtrip.settings.auto_approve);

    let with_auto = serde_json::json!({
      "version": 1,
      "accounts": [],
      "settings": { "minimizeToTray": false, "autoApprove": true }
    });
    let (migrated_auto, _) = migrate_from_value(with_auto).expect("migrate settings with autoApprove");
    assert!(!migrated_auto.settings.minimize_to_tray);
    assert!(migrated_auto.settings.auto_approve);
  }
}
