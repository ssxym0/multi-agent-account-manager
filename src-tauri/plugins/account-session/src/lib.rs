#[cfg(mobile)]
use serde::{Deserialize, Serialize};
#[cfg(mobile)]
use tauri::{
  ipc::Channel,
  plugin::{Builder, PluginHandle, TauriPlugin},
  AppHandle, Manager, Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_account_session);

#[cfg(mobile)]
pub struct MobileSessionBridge<R: Runtime>(pub PluginHandle<R>);

#[cfg(mobile)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenMobileAccountPayload {
  pub provider_id: String,
  pub provider_name: String,
  pub uuid: String,
  pub alias: String,
  pub auto_alias: bool,
  pub entry_url: String,
  pub allowed_hosts: Vec<String>,
  pub proxy_url: Option<String>,
  pub timezone: Option<String>,
  pub timezone_init_script: Option<String>,
  pub switch_seq: u64,
  pub view_instance_id: u64,
  pub is_cold_start: bool,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileCapabilitiesResponse {
  pub multi_profile_supported: bool,
  pub profile_names: Vec<String>,
}

#[cfg(mobile)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MobileSessionEvent {
  #[serde(rename_all = "camelCase")]
  PageCommitVisible {
    provider_id: String,
    uuid: String,
    switch_seq: u64,
    view_instance_id: u64,
    url: String,
  },
  #[serde(rename_all = "camelCase")]
  IdentityDetected {
    provider_id: String,
    uuid: String,
    switch_seq: u64,
    view_instance_id: u64,
    identity: String,
    #[serde(default)]
    avatar_url: Option<String>,
    #[serde(default)]
    quota_text: Option<String>,
    #[serde(default)]
    quota_percent: Option<u8>,
    #[serde(default)]
    quota_detail_json: Option<String>,
  },
  #[serde(rename_all = "camelCase")]
  SessionFailed {
    switch_seq: u64,
    view_instance_id: u64,
    reason: String,
  },
  #[serde(rename_all = "camelCase")]
  ActivityClosed {
    switch_seq: u64,
    view_instance_id: u64,
  },
  #[serde(rename_all = "camelCase")]
  ExternalAuthUnavailable {
    switch_seq: u64,
    view_instance_id: u64,
    reason: String,
  },
  #[serde(rename_all = "camelCase")]
  RecreateRequested {
    previous_switch_seq: u64,
  },
  #[serde(rename_all = "camelCase")]
  UpdateProgress {
    stage: String,
    downloaded: u64,
    #[serde(default)]
    total: Option<u64>,
    #[serde(default)]
    percent: Option<u8>,
    message: String,
  },
  #[serde(rename_all = "camelCase")]
  SessionLoggedOut {
    provider_id: String,
    uuid: String,
    switch_seq: u64,
    view_instance_id: u64,
  },
}

#[cfg(mobile)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileFetchTextResponse {
  pub status: u16,
  pub body: String,
}

#[cfg(target_os = "android")]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidInstallApkResponse {
  pub status: String,
  pub message: String,
}

#[cfg(mobile)]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("account-session")
    .setup(|app, api| {
      #[cfg(target_os = "android")]
      let handle =
        api.register_android_plugin("com.ldjx.accountmanager", "AccountSessionPlugin")?;
      #[cfg(target_os = "ios")]
      let handle = api.register_ios_plugin(init_plugin_account_session)?;
      app.manage(MobileSessionBridge(handle));
      Ok(())
    })
    .build()
}

#[cfg(mobile)]
pub async fn subscribe_events(
  app: &AppHandle,
  channel: Channel<serde_json::Value>,
) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "subscribeEvents",
      serde_json::json!({ "channel": channel }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn check_capabilities(app: &AppHandle) -> Result<MobileCapabilitiesResponse, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async("checkCapabilities", serde_json::json!({}))
    .await
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn open_account_session(
  app: &AppHandle,
  payload: OpenMobileAccountPayload,
) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>("openAccountSession", payload)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn update_account_title(
  app: &AppHandle,
  switch_seq: u64,
  title: &str,
) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "updateAccountTitle",
      serde_json::json!({ "switchSeq": switch_seq, "title": title }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn close_active_session(app: &AppHandle, switch_seq: u64) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "closeActiveSession",
      serde_json::json!({ "switchSeq": switch_seq }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn delete_profile(app: &AppHandle, uuid: &str) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "deleteProfile",
      serde_json::json!({ "uuid": uuid }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn copy_profile_cookie(app: &AppHandle, uuid: &str, url: &str) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "copyProfileCookie",
      serde_json::json!({ "uuid": uuid, "url": url }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn import_profile_cookies(
  app: &AppHandle,
  uuid: &str,
  url: &str,
  cookies: Vec<String>,
) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "importProfileCookies",
      serde_json::json!({ "uuid": uuid, "url": url, "cookies": cookies }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileExportedCookie {
  pub name: String,
  pub value: String,
  pub domain: String,
  pub path: String,
  pub secure: bool,
  pub http_only: bool,
  pub same_site: Option<String>,
  pub max_age_secs: Option<i64>,
}

#[cfg(mobile)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileExportCookiesResponse {
  cookies: Vec<MobileExportedCookie>,
}

#[cfg(mobile)]
pub async fn export_profile_cookies(
  app: &AppHandle,
  uuid: &str,
  allowed_hosts: Vec<String>,
  urls: Vec<String>,
) -> Result<Vec<MobileExportedCookie>, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  let resp: MobileExportCookiesResponse = bridge
    .0
    .run_mobile_plugin_async(
      "exportProfileCookies",
      serde_json::json!({ "uuid": uuid, "allowedHosts": allowed_hosts, "urls": urls }),
    )
    .await
    .map_err(|e| e.to_string())?;
  Ok(resp.cookies)
}

#[cfg(mobile)]
pub async fn open_external_url(app: &AppHandle, url: &str) -> Result<(), String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async::<serde_json::Value>(
      "openExternalUrl",
      serde_json::json!({ "url": url }),
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn fetch_text(
  app: &AppHandle,
  url: &str,
  user_agent: &str,
  accept: Option<&str>,
) -> Result<MobileFetchTextResponse, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async(
      "fetchText",
      serde_json::json!({
        "url": url,
        "userAgent": user_agent,
        "accept": accept,
      }),
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
pub async fn http_request(
  app: &AppHandle,
  method: &str,
  url: &str,
  headers: std::collections::HashMap<String, String>,
  body: Option<String>,
) -> Result<MobileFetchTextResponse, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async(
      "httpRequest",
      serde_json::json!({
        "method": method,
        "url": url,
        "headers": headers,
        "body": body,
      }),
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg(mobile)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileScanQrResponse {
  pub content: String,
  pub cancelled: bool,
}

#[cfg(mobile)]
pub async fn scan_qr_code(app: &AppHandle) -> Result<MobileScanQrResponse, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async("scanQrCode", serde_json::json!({}))
    .await
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "android")]
pub async fn download_and_install_apk(
  app: &AppHandle,
  url: &str,
  sha256: Option<&str>,
  filename: &str,
  user_agent: &str,
) -> Result<AndroidInstallApkResponse, String> {
  let bridge = app.state::<MobileSessionBridge<tauri::Wry>>();
  bridge
    .0
    .run_mobile_plugin_async(
      "downloadAndInstallApk",
      serde_json::json!({
        "url": url,
        "sha256": sha256,
        "filename": filename,
        "userAgent": user_agent,
      }),
    )
    .await
    .map_err(|e| e.to_string())
}
