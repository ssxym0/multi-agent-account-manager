mod group_transport;
mod platform;
mod providers;
mod session_manager;
mod site_adapters;
mod storage;
mod sync_bundle;
mod updater;

use platform::ProxyTestResult;
use session_manager::{
  ImportSyncBundleResponse, ManagerSnapshot, ScanQrResult, SessionManager, APP_TITLE,
};
use site_adapters::SiteAdaptersSnapshot;
use storage::{
  AccountKey, ExternalAgentKind, GroupMemberRole, GroupSharedSkill, GroupWorkflowMode,
};
use sync_bundle::{ExportSyncBundleResponse, LanSyncShareInfo};
#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use updater::{InstallUpdateOutcome, InstallUpdateRequest, UpdateCheckResult};

#[cfg(desktop)]
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
async fn get_manager_state(mgr: State<'_, Arc<SessionManager>>) -> Result<ManagerSnapshot, String> {
  Ok(mgr.get_snapshot().await)
}

#[tauri::command]
async fn select_provider(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
) -> Result<ManagerSnapshot, String> {
  mgr.select_provider(&app, &provider_id).await
}

#[tauri::command]
async fn add_account(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
  alias: String,
  proxy_url: Option<String>,
  timezone: Option<String>,
  raw_cookies: Option<String>,
) -> Result<ManagerSnapshot, String> {
  mgr
    .add_account(
      &app,
      &provider_id,
      &alias,
      proxy_url.as_deref(),
      timezone.as_deref(),
      raw_cookies.as_deref(),
    )
    .await
}

#[tauri::command]
async fn rename_account(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
  uuid: String,
  alias: String,
  proxy_url: Option<String>,
  timezone: Option<String>,
) -> Result<ManagerSnapshot, String> {
  mgr
    .update_account_settings(
      &app,
      &provider_id,
      &uuid,
      &alias,
      proxy_url.as_deref(),
      timezone.as_deref(),
    )
    .await
}

#[tauri::command]
async fn copy_account_cookie(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  uuid: String,
) -> Result<(), String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以复制 Cookie".to_string());
  }
  mgr.copy_account_cookie(&app, &provider_id, &uuid).await
}

#[tauri::command]
async fn import_account_cookies(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  uuid: String,
  raw_cookies: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以导入 Cookie".to_string());
  }
  mgr
    .import_account_cookies(&app, &provider_id, &uuid, &raw_cookies)
    .await
}

#[tauri::command]
async fn remove_account(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
  uuid: String,
) -> Result<ManagerSnapshot, String> {
  mgr.remove_account(&app, &provider_id, &uuid).await
}

#[tauri::command]
async fn reorder_accounts(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  ordered_uuids: Vec<String>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以调整账号顺序".to_string());
  }
  mgr.reorder_accounts(&provider_id, ordered_uuids).await
}

#[tauri::command]
async fn reveal_downloaded_file(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  download_id: u64,
) -> Result<(), String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以打开下载文件夹".to_string());
  }
  mgr.reveal_downloaded_file(download_id).await
}

#[tauri::command]
async fn switch_account(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
  uuid: String,
) -> Result<ManagerSnapshot, String> {
  mgr.switch_account(&app, &provider_id, &uuid).await
}

#[tauri::command]
async fn set_modal_active(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  active: bool,
) -> Result<ManagerSnapshot, String> {
  mgr.set_modal_active(&app, active).await
}

#[tauri::command]
async fn open_official_site(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  provider_id: String,
) -> Result<ManagerSnapshot, String> {
  mgr.open_official_site(&app, &provider_id).await
}

#[tauri::command]
async fn resolve_uuid_conflict(
  mgr: State<'_, Arc<SessionManager>>,
  keep_provider_id: String,
  uuid: String,
) -> Result<ManagerSnapshot, String> {
  mgr.resolve_uuid_conflict(&keep_provider_id, &uuid).await
}

#[tauri::command]
async fn dismiss_notice(mgr: State<'_, Arc<SessionManager>>) -> Result<ManagerSnapshot, String> {
  Ok(mgr.dismiss_notice().await)
}

#[tauri::command]
async fn trigger_quota_sync(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
) -> Result<ManagerSnapshot, String> {
  mgr.trigger_silent_sync(&app, true).await;
  Ok(mgr.get_snapshot().await)
}

#[tauri::command]
async fn test_account_proxy(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  proxy_url: String,
) -> Result<ProxyTestResult, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以测试代理连接".to_string());
  }
  mgr.test_account_proxy(&provider_id, &proxy_url).await
}

#[tauri::command]
async fn reload_account(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  uuid: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以刷新账号".to_string());
  }
  mgr.reload_account(&app, &provider_id, &uuid).await
}

#[tauri::command]
async fn clear_account_data(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  provider_id: String,
  uuid: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以清理账号缓存".to_string());
  }
  mgr.clear_account_data(&app, &provider_id, &uuid).await
}

#[tauri::command]
async fn update_app_settings(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  minimize_to_tray: bool,
  global_shortcut: String,
  auto_approve: Option<bool>,
  isolate_same_proxy_keep_alive: Option<bool>,
  silent_sync_interval_mins: Option<u16>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以修改应用设置".to_string());
  }
  mgr
    .update_app_settings(
      &app,
      minimize_to_tray,
      &global_shortcut,
      auto_approve,
      isolate_same_proxy_keep_alive,
      silent_sync_interval_mins,
    )
    .await
}

#[tauri::command]
async fn export_sync_bundle(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  passphrase: String,
) -> Result<ExportSyncBundleResponse, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以导出同步备份".to_string());
  }
  mgr.export_sync_bundle(&app, passphrase).await
}

#[tauri::command]
async fn import_sync_bundle(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  bundle_json: String,
  passphrase: String,
) -> Result<ImportSyncBundleResponse, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以导入同步备份".to_string());
  }
  mgr.import_sync_bundle(&app, bundle_json, passphrase).await
}

#[tauri::command]
async fn start_lan_sync_share(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  passphrase: Option<String>,
) -> Result<LanSyncShareInfo, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以开启局域网同步".to_string());
  }
  mgr.start_lan_sync_share(&app, passphrase).await
}

#[tauri::command]
async fn start_relay_sync_share(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  relay_url: Option<String>,
) -> Result<LanSyncShareInfo, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以开启加密中转同步".to_string());
  }
  mgr.start_relay_sync_share(&app, relay_url).await
}

#[tauri::command]
async fn stop_lan_sync_share(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
) -> Result<(), String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以停止同步共享".to_string());
  }
  mgr.stop_lan_sync_share(&app).await
}

#[tauri::command]
async fn pull_lan_sync_share(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  address: String,
  passphrase: String,
) -> Result<ImportSyncBundleResponse, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以拉取局域网同步".to_string());
  }
  mgr.pull_lan_sync_share(&app, address, passphrase).await
}

#[tauri::command]
async fn scan_sync_qr_code(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
) -> Result<ScanQrResult, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以调用扫码同步".to_string());
  }
  mgr.scan_sync_qr_code(&app).await
}

#[tauri::command]
async fn select_group_chat(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: Option<String>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以切换群聊".to_string());
  }
  mgr.select_group_chat(&app, group_id).await
}

#[tauri::command]
async fn create_group_chat(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  name: String,
  members: Vec<AccountKey>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以创建群聊".to_string());
  }
  mgr.create_group_chat(&app, name, members).await
}

#[tauri::command]
async fn update_group_chat(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  name: String,
  members: Vec<AccountKey>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以编辑群聊".to_string());
  }
  mgr.update_group_chat(&app, group_id, name, members).await
}

#[tauri::command]
async fn delete_group_chat(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以删除群聊".to_string());
  }
  mgr.delete_group_chat(&app, group_id).await
}

#[tauri::command]
async fn send_group_message(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  content: String,
  target_members: Option<Vec<AccountKey>>,
  auto_channels: Option<Vec<String>>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以发送群聊消息".to_string());
  }
  mgr
    .send_group_message(&app, group_id, content, target_members, auto_channels)
    .await
}

#[tauri::command]
async fn retry_group_message(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  msg_id: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以重试群聊消息".to_string());
  }
  mgr.retry_group_message(&app, group_id, msg_id).await
}

#[tauri::command]
async fn sync_group_message_reply(
  app: AppHandle,
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  msg_id: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以同步群聊回复".to_string());
  }
  mgr.sync_group_message_reply(&app, group_id, msg_id).await
}

#[tauri::command]
async fn clear_group_messages(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以清空群聊记录".to_string());
  }
  mgr.clear_group_messages(group_id).await
}

#[tauri::command]
async fn start_group_new_topic(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  keep_shared_context: bool,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以开启新话题".to_string());
  }
  mgr.start_group_new_topic(group_id, keep_shared_context).await
}

#[tauri::command]
async fn update_group_knowledge(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  shared_context: String,
  shared_skills: Vec<GroupSharedSkill>,
  member_roles: Vec<GroupMemberRole>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以保存群聊知识配置".to_string());
  }
  mgr
    .update_group_knowledge(group_id, shared_context, shared_skills, member_roles)
    .await
}

#[tauri::command]
async fn append_group_shared_context(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  snippet: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以沉淀群聊上下文".to_string());
  }
  mgr.append_group_shared_context(group_id, snippet).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn upsert_external_agent(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  id: Option<String>,
  kind: ExternalAgentKind,
  name: String,
  base_url: String,
  api_key: String,
  model: String,
  command: String,
  args: Vec<String>,
  workdir: Option<String>,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以管理 API / CLI 成员".to_string());
  }
  mgr
    .upsert_external_agent(id, kind, name, base_url, api_key, model, command, args, workdir)
    .await
}

#[tauri::command]
async fn delete_external_agent(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  id: String,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以删除 API / CLI 成员".to_string());
  }
  mgr.delete_external_agent(id).await
}

#[tauri::command]
async fn update_group_workflow(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  workflow_mode: GroupWorkflowMode,
  max_relay_hops: u8,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以修改协作模式".to_string());
  }
  mgr
    .update_group_workflow(group_id, workflow_mode, max_relay_hops)
    .await
}

#[tauri::command]
async fn reset_group_thread_binding(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  group_id: String,
  member: AccountKey,
) -> Result<ManagerSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以重置成员绑定会话".to_string());
  }
  mgr.reset_group_thread_binding(group_id, member).await
}

#[tauri::command]
async fn get_site_adapters(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
) -> Result<SiteAdaptersSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以查看站点适配规则".to_string());
  }
  Ok(mgr.get_site_adapters().await)
}

#[tauri::command]
async fn save_site_adapter_overrides(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
  raw_json: String,
) -> Result<SiteAdaptersSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以保存站点适配规则".to_string());
  }
  mgr.save_site_adapter_overrides(raw_json).await
}

#[tauri::command]
async fn reset_site_adapter_overrides(
  mgr: State<'_, Arc<SessionManager>>,
  webview: tauri::Webview,
) -> Result<SiteAdaptersSnapshot, String> {
  if webview.label() != "manager" {
    return Err("仅账号管理页可以重置站点适配规则".to_string());
  }
  mgr.reset_site_adapter_overrides().await
}

#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<UpdateCheckResult, String> {
  updater::check_for_updates(&app).await
}

#[tauri::command]
async fn download_and_install_update(
  app: AppHandle,
  request: InstallUpdateRequest,
) -> Result<InstallUpdateOutcome, String> {
  updater::download_and_install_update(&app, request).await
}

#[tauri::command]
async fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
  updater::open_external_url(&app, &url).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default();

  #[cfg(desktop)]
  let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

  #[cfg(mobile)]
  let builder = builder.plugin(platform::mobile::init());

  builder
    .setup(|app| {
      let app_data_dir = app.path().app_data_dir()?;
      let session_mgr = Arc::new(SessionManager::new(app_data_dir));
      app.manage(Arc::clone(&session_mgr));

      #[cfg(desktop)]
      {
        use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
        use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

        let toggle_item = MenuItem::with_id(app, "tray_toggle", "显示 / 隐藏窗口", true, None::<&str>)?;
        let sep_item = PredefinedMenuItem::separator(app)?;
        let quit_item = MenuItem::with_id(app, "tray_quit", "退出叠号", true, None::<&str>)?;
        let tray_menu = Menu::with_items(app, &[&toggle_item, &sep_item, &quit_item])?;
        let tray_icon = match app.default_window_icon().cloned() {
          Some(icon) => icon,
          None => tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?,
        };
        let _tray = TrayIconBuilder::with_id("main-tray")
          .icon(tray_icon)
          .menu(&tray_menu)
          .tooltip("叠号 · AI 助手账号切换器")
          .show_menu_on_left_click(false)
          .on_menu_event(|app_handle, event| match event.id().as_ref() {
            "tray_toggle" => SessionManager::toggle_main_window(app_handle),
            "tray_quit" => {
              QUIT_REQUESTED.store(true, Ordering::SeqCst);
              app_handle.exit(0);
            }
            _ => {}
          })
          .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
              button: MouseButton::Left,
              button_state: MouseButtonState::Up,
              ..
            } = event
            {
              SessionManager::toggle_main_window(tray.app_handle());
            }
          })
          .build(app)?;

        let main_window = tauri::window::WindowBuilder::new(app, "main")
          .title(APP_TITLE)
          .inner_size(1240.0, 800.0)
          .min_inner_size(760.0, 520.0)
          .build()?;

        let manager_builder =
          tauri::webview::WebviewBuilder::new("manager", tauri::WebviewUrl::App("index.html".into()))
            .disable_drag_drop_handler()
            .on_navigation(|url| {
              url.scheme() == "tauri"
                || url.domain() == Some("tauri.localhost")
                || (cfg!(dev) && url.host_str() == Some("localhost"))
            });

        let _manager_wv = main_window.add_child(
          manager_builder,
          tauri::LogicalPosition::new(0.0, 0.0),
          tauri::LogicalSize::new(1240.0, 800.0),
        )?;
        #[cfg(windows)]
        platform::lock_windows_manager_webview_zoom(&_manager_wv);

        let app_for_resize = app.handle().clone();
        let mgr_for_resize = Arc::clone(&session_mgr);
        main_window.on_window_event(move |event| match event {
          tauri::WindowEvent::Resized(_)
          | tauri::WindowEvent::Moved(_)
          | tauri::WindowEvent::ScaleFactorChanged { .. } => {
            let app_inner = app_for_resize.clone();
            let mgr_inner = Arc::clone(&mgr_for_resize);
            tauri::async_runtime::spawn(async move {
              mgr_inner.recompute_desktop_bounds(&app_inner).await;
            });
          }
          tauri::WindowEvent::Focused(focused) if *focused => {
            let app_inner = app_for_resize.clone();
            let mgr_inner = Arc::clone(&mgr_for_resize);
            tauri::async_runtime::spawn(async move {
              mgr_inner.trigger_silent_sync(&app_inner, false).await;
            });
          }
          tauri::WindowEvent::CloseRequested { api, .. } => {
            if !QUIT_REQUESTED.load(Ordering::SeqCst) && mgr_for_resize.minimize_to_tray_enabled() {
              api.prevent_close();
              if let Some(win) = app_for_resize.get_window("main") {
                let _ = win.hide();
              }
            }
          }
          _ => {}
        });
      }

      #[cfg(mobile)]
      {
        let _ = APP_TITLE;
        tauri::WebviewWindowBuilder::new(
          app,
          "manager",
          tauri::WebviewUrl::App("index.html".into()),
        )
        .build()?;
      }

      let app_for_boot = app.handle().clone();
      let mgr_for_boot = Arc::clone(&session_mgr);
      tauri::async_runtime::spawn(async move {
        mgr_for_boot.run_startup_tasks(&app_for_boot).await;
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_manager_state,
      select_provider,
      add_account,
      rename_account,
      test_account_proxy,
      copy_account_cookie,
      import_account_cookies,
      remove_account,
      reorder_accounts,
      reload_account,
      clear_account_data,
      reveal_downloaded_file,
      switch_account,
      set_modal_active,
      open_official_site,
      resolve_uuid_conflict,
      dismiss_notice,
      trigger_quota_sync,
      update_app_settings,
      select_group_chat,
      create_group_chat,
      update_group_chat,
      delete_group_chat,
      send_group_message,
      retry_group_message,
      sync_group_message_reply,
      clear_group_messages,
      start_group_new_topic,
      update_group_knowledge,
      append_group_shared_context,
      upsert_external_agent,
      delete_external_agent,
      update_group_workflow,
      reset_group_thread_binding,
      get_site_adapters,
      save_site_adapter_overrides,
      reset_site_adapter_overrides,
      check_for_updates,
      download_and_install_update,
      open_external_url,
      export_sync_bundle,
      import_sync_bundle,
      start_lan_sync_share,
      start_relay_sync_share,
      stop_lan_sync_share,
      pull_lan_sync_share,
      scan_sync_qr_code,
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|app_handle, event| {
      #[cfg(target_os = "macos")]
      if let tauri::RunEvent::Reopen {
        has_visible_windows: false,
        ..
      } = event
      {
        if let Some(win) = app_handle.get_window("main") {
          let _ = win.unminimize();
          let _ = win.show();
          let _ = win.set_focus();
        }
      }
      #[cfg(not(target_os = "macos"))]
      let _ = (app_handle, event);
    });
}
