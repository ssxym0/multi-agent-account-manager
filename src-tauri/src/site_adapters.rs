use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteAdapterConfig {
  pub id: String,
  pub name: String,
  #[serde(default)]
  pub host_patterns: Vec<String>,
  #[serde(default)]
  pub composer_selectors: Vec<String>,
  #[serde(default)]
  pub send_button_selectors: Vec<String>,
  #[serde(default)]
  pub stop_button_selectors: Vec<String>,
  #[serde(default)]
  pub message_list_selectors: Vec<String>,
  #[serde(default)]
  pub assistant_message_selectors: Vec<String>,
  #[serde(default)]
  pub new_chat_selectors: Vec<String>,
  #[serde(default)]
  pub new_chat_url: Option<String>,
  #[serde(default)]
  pub thread_url_patterns: Vec<String>,
  #[serde(default)]
  pub transient_status_words: Vec<String>,
  #[serde(default)]
  pub chat_api_patterns: Vec<String>,
  #[serde(default)]
  pub prefer_enter_submit: bool,
  #[serde(default)]
  pub async_agent: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteAdaptersSnapshot {
  pub adapters: Vec<SiteAdapterConfig>,
  pub override_path: String,
  pub has_overrides: bool,
  pub override_json: String,
}

fn svec(items: &[&str]) -> Vec<String> {
  items.iter().map(|s| (*s).to_string()).collect()
}

pub fn builtin_site_adapters() -> Vec<SiteAdapterConfig> {
  vec![
    SiteAdapterConfig {
      id: "muse".to_string(),
      name: "Muse · Meta".to_string(),
      host_patterns: svec(&["muse.ai", "*.muse.ai"]),
      composer_selectors: svec(&["textarea[placeholder*=\"消息\"]", "textarea"]),
      send_button_selectors: svec(&["button[aria-label*=\"发送\"]", "button[title*=\"发送\"]"]),
      stop_button_selectors: svec(&[
        "button[aria-label*=\"停止\"]",
        "button[data-testid*=\"stop\" i]",
      ]),
      message_list_selectors: svec(&["[data-testid=\"message-list\"]"]),
      assistant_message_selectors: svec(&[
        "[data-message-author-role=\"assistant\"]",
        "[data-testid*=\"assistant\" i]",
      ]),
      new_chat_selectors: Vec::new(),
      new_chat_url: None,
      thread_url_patterns: Vec::new(),
      transient_status_words: svec(&[
        "正在预热",
        "预热中",
        "空闲中",
        "已连接",
        "连接中",
        "正在连接",
        "仍在发送中",
        "发送中",
        "努力工作中",
        "工作中",
        "努力思考中",
        "检索中",
        "正在检索",
        "正在回复",
        "回复中",
        "正在分析",
        "分析中",
        "正在查询",
        "查询中",
        "正在处理",
        "处理中",
        "正在生成",
        "生成中",
        "正在准备",
        "准备中",
        "搜索中",
        "正在搜索",
        "正在搜索网页",
        "正在联网搜索",
        "Researching channels",
        "Researching",
        "Searching",
        "加载中",
        "待命中",
      ]),
      chat_api_patterns: svec(&["/api/", "/events", "/chat", "/view"]),
      prefer_enter_submit: false,
      async_agent: true,
    },
    SiteAdapterConfig {
      id: "hark".to_string(),
      name: "Hark".to_string(),
      host_patterns: svec(&["app.hark.com", "*.hark.com", "hark.com"]),
      composer_selectors: svec(&["textarea[placeholder*=\"消息\"]", "textarea"]),
      send_button_selectors: svec(&["button[type=\"submit\"]", "button[aria-label*=\"发送\"]"]),
      stop_button_selectors: svec(&[
        "button[aria-label*=\"停止\"]",
        "button[data-testid*=\"stop\" i]",
      ]),
      message_list_selectors: svec(&["[data-testid=\"message-list\"]"]),
      assistant_message_selectors: svec(&[
        "[data-message-author-role=\"assistant\"]",
        "[data-testid*=\"assistant\" i]",
      ]),
      new_chat_selectors: Vec::new(),
      new_chat_url: None,
      thread_url_patterns: Vec::new(),
      transient_status_words: svec(&[
        "已阅读",
        "给 Hark 发消息",
        "Hark",
        "正在搜索网页",
        "正在搜索",
        "搜索中",
        "正在思考",
        "思考中",
      ]),
      chat_api_patterns: svec(&["/api/messages/send", "/api/messages/", "/api/chat"]),
      prefer_enter_submit: false,
      async_agent: true,
    },
    SiteAdapterConfig {
      id: "generic".to_string(),
      name: "通用适配兜底".to_string(),
      host_patterns: svec(&["*"]),
      composer_selectors: svec(&[
        "textarea",
        "[contenteditable=\"true\"]",
        "[role=\"textbox\"]",
      ]),
      send_button_selectors: svec(&[
        "button[data-testid*=\"send\" i]",
        "button[aria-label*=\"Send\" i]",
        "button[aria-label*=\"发送\"]",
        "button[type=\"submit\"]",
      ]),
      stop_button_selectors: svec(&[
        "button[aria-label*=\"Stop\" i]",
        "button[aria-label*=\"停止\"]",
        "button[data-testid*=\"stop\" i]",
        "[aria-busy=\"true\"]",
      ]),
      message_list_selectors: svec(&[
        "[data-testid=\"message-list\"]",
        "[data-testid*=\"chat-messages\" i]",
        "[data-testid*=\"message-feed\" i]",
      ]),
      assistant_message_selectors: svec(&[
        "[data-message-author-role=\"assistant\"]",
        "[data-role=\"assistant\"]",
        "[data-testid*=\"assistant\" i]",
      ]),
      new_chat_selectors: svec(&[
        "button[aria-label*=\"New chat\" i]",
        "button[aria-label*=\"新对话\"]",
        "button[aria-label*=\"新建\"]",
      ]),
      new_chat_url: None,
      thread_url_patterns: svec(&["/c/", "/chat/", "/app/", "/s/", "cid="]),
      transient_status_words: Vec::new(),
      chat_api_patterns: svec(&[
        "/api/",
        "/chat",
        "/conversation",
        "/completion",
        "/generate",
        "/stream",
      ]),
      prefer_enter_submit: false,
      async_agent: false,
    },
  ]
}

pub fn override_file_path(app_data_dir: &Path) -> PathBuf {
  app_data_dir.join("site_adapters.json")
}

fn parse_overrides_str(raw: &str) -> Result<Vec<SiteAdapterConfig>, String> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Ok(Vec::new());
  }
  let val: Value =
    serde_json::from_str(trimmed).map_err(|e| format!("站点适配 JSON 语法错误: {e}"))?;
  let list_val = if val.is_array() {
    val
  } else if let Some(arr) = val.get("adapters").and_then(|v| v.as_array()) {
    Value::Array(arr.clone())
  } else {
    return Err("站点适配 JSON 必须是数组或包含 adapters 数组的对象".to_string());
  };
  let mut parsed: Vec<SiteAdapterConfig> = serde_json::from_value(list_val)
    .map_err(|e| format!("站点适配字段格式无效: {e}"))?;
  for item in &mut parsed {
    item.id = item.id.trim().to_string();
    item.name = item.name.trim().to_string();
    if item.id.is_empty() || item.name.is_empty() {
      return Err("每个站点适配规则都必须包含非空的 id 和 name".to_string());
    }
    if item.host_patterns.is_empty() {
      return Err(format!("站点规则「{}」至少需要一个 hostPatterns", item.id));
    }
  }
  Ok(parsed)
}

pub fn merge_adapters(overrides: Vec<SiteAdapterConfig>) -> Vec<SiteAdapterConfig> {
  let mut result = builtin_site_adapters();
  for ov in overrides {
    if let Some(pos) = result.iter().position(|b| b.id == ov.id) {
      result[pos] = ov;
    } else if let Some(generic_pos) = result.iter().position(|b| b.id == "generic") {
      result.insert(generic_pos, ov);
    } else {
      result.push(ov);
    }
  }
  result
}

pub fn load_effective_adapters(app_data_dir: &Path) -> Vec<SiteAdapterConfig> {
  let path = override_file_path(app_data_dir);
  let Ok(raw) = fs::read_to_string(path) else {
    return builtin_site_adapters();
  };
  match parse_overrides_str(&raw) {
    Ok(overrides) => merge_adapters(overrides),
    Err(_) => builtin_site_adapters(),
  }
}

fn host_matches_pattern(host: &str, pattern: &str) -> bool {
  let h = host.trim().to_ascii_lowercase();
  let p = pattern.trim().to_ascii_lowercase();
  if p.is_empty() {
    return false;
  }
  if p == "*" {
    return true;
  }
  if let Some(suffix) = p.strip_prefix("*.") {
    return h == suffix || h.ends_with(&format!(".{suffix}"));
  }
  h == p
}

pub fn resolve_adapter_for_host(app_data_dir: &Path, host: &str) -> SiteAdapterConfig {
  let adapters = load_effective_adapters(app_data_dir);
  for adapter in &adapters {
    if adapter.id == "generic" {
      continue;
    }
    if adapter
      .host_patterns
      .iter()
      .any(|pat| host_matches_pattern(host, pat))
    {
      return adapter.clone();
    }
  }
  adapters
    .into_iter()
    .find(|a| a.id == "generic")
    .unwrap_or_else(|| builtin_site_adapters().pop().expect("generic adapter"))
}

pub fn get_adapters_snapshot(app_data_dir: &Path) -> SiteAdaptersSnapshot {
  let path = override_file_path(app_data_dir);
  let override_raw = fs::read_to_string(&path).ok();
  let has_overrides = override_raw.as_ref().is_some_and(|s| !s.trim().is_empty());
  let override_json = override_raw.unwrap_or_default();
  SiteAdaptersSnapshot {
    adapters: load_effective_adapters(app_data_dir),
    override_path: path.to_string_lossy().to_string(),
    has_overrides,
    override_json,
  }
}

pub fn validate_and_save_overrides(
  app_data_dir: &Path,
  raw_json: &str,
) -> Result<SiteAdaptersSnapshot, String> {
  let trimmed = raw_json.trim();
  if trimmed.is_empty() {
    return reset_overrides(app_data_dir);
  }
  let parsed = parse_overrides_str(trimmed)?;
  let path = override_file_path(app_data_dir);
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
  }
  let pretty =
    serde_json::to_vec_pretty(&parsed).map_err(|e| format!("序列化站点规则失败: {e}"))?;
  let tmp_path = path.with_file_name(format!(".site_adapters.tmp.{}", Uuid::new_v4().simple()));
  {
    let mut f = File::create(&tmp_path).map_err(|e| format!("创建临时规则文件失败: {e}"))?;
    f.write_all(&pretty)
      .map_err(|e| format!("写入规则文件失败: {e}"))?;
    f.sync_all()
      .map_err(|e| format!("同步规则文件失败: {e}"))?;
  }
  fs::rename(&tmp_path, &path).map_err(|e| {
    let _ = fs::remove_file(&tmp_path);
    format!("替换 site_adapters.json 失败: {e}")
  })?;
  Ok(get_adapters_snapshot(app_data_dir))
}

pub fn reset_overrides(app_data_dir: &Path) -> Result<SiteAdaptersSnapshot, String> {
  let path = override_file_path(app_data_dir);
  if path.exists() {
    fs::remove_file(&path).map_err(|e| format!("删除自定义站点规则失败: {e}"))?;
  }
  Ok(get_adapters_snapshot(app_data_dir))
}

#[cfg(desktop)]
pub const NET_HOOK_INIT_JS: &str = r#"(() => {
  try {
    if (window.__maamNet && window.__maamNet.installed) return;
    const state = {
      installed: true,
      seq: 0,
      entries: [],
      wsLastAt: 0,
      wsText: '',
      wsEvents: [],
      esLastAt: 0,
      esText: ''
    };
    window.__maamNet = state;

    const TELEMETRY_RE = /google-analytics|googletagmanager|\/_mp\/|\/cdn-cgi\/rum|\/otlp\/|\/pixel\/|\/billing\/summary|\/unread-count|\/consent\/status|\/client-ip|\.(?:js|css|png|jpg|jpeg|svg|woff2?|ico)(?:\?|$)/i;
    const pushEntry = (entry) => {
      state.entries.push(entry);
      if (state.entries.length > 40) {
        state.entries.splice(0, state.entries.length - 40);
      }
    };

    const pickTextFromJson = (obj) => {
      if (!obj || typeof obj !== 'object') return '';
      if (Array.isArray(obj.choices) && obj.choices[0]) {
        const c = obj.choices[0];
        if (c.delta && typeof c.delta.content === 'string') return c.delta.content;
        if (c.message && typeof c.message.content === 'string') return c.message.content;
        if (typeof c.text === 'string') return c.text;
      }
      if (obj.delta && typeof obj.delta.text === 'string') return obj.delta.text;
      if (obj.type === 'content_block_delta' && obj.delta && typeof obj.delta.text === 'string') return obj.delta.text;
      if (obj.o === 'append' && typeof obj.v === 'string') return obj.v;
      if (obj.message && obj.message.author && obj.message.author.role === 'assistant' && obj.message.content && Array.isArray(obj.message.content.parts)) {
        const joined = obj.message.content.parts.filter((p) => typeof p === 'string').join('');
        if (joined) return '\x00REPLACE:' + joined;
      }
      if (typeof obj.completion === 'string') return obj.completion;
      if (typeof obj.delta === 'string') return obj.delta;
      if (typeof obj.text === 'string' && !obj.role) return obj.text;
      if (typeof obj.answer === 'string') return obj.answer;
      if (typeof obj.reply === 'string') return obj.reply;
      return '';
    };

    const consumeStream = async (resp, entry) => {
      let reader = null;
      let hitLimit = false;
      try {
        reader = resp.body && resp.body.getReader ? resp.body.getReader() : null;
        if (!reader) {
          entry.done = true;
          return;
        }
        const decoder = new TextDecoder('utf-8', { fatal: false });
        let buf = '';
        let totalBytes = 0;
        let isSseMode = entry.contentType.includes('event-stream');
        while (true) {
          if (totalBytes >= 262144) {
            hitLimit = true;
            break;
          }
          const { done, value } = await reader.read();
          if (done) break;
          if (!value || !value.byteLength) continue;
          totalBytes += value.byteLength;
          entry.updatedAt = Date.now();
          const chunk = decoder.decode(value, { stream: true });
          buf += chunk;
          if (!isSseMode && chunk.includes('data:')) {
            isSseMode = true;
            entry.isStream = true;
          }
          if (isSseMode) {
            const lines = buf.split(/\r?\n/);
            buf = lines.pop() || '';
            for (const rawLine of lines) {
              const line = rawLine.trim();
              if (!line.startsWith('data:')) continue;
              const payload = line.slice(5).trim();
              if (!payload) continue;
              if (payload === '[DONE]') {
                entry.done = true;
                continue;
              }
              try {
                const parsed = JSON.parse(payload);
                const piece = pickTextFromJson(parsed);
                if (piece.startsWith('\x00REPLACE:')) {
                  entry.text = piece.slice(9);
                } else if (piece) {
                  entry.text += piece;
                }
              } catch (_) {}
            }
          }
        }
        if (!entry.text && buf.trim()) {
          try {
            const parsed = JSON.parse(buf.trim());
            const piece = pickTextFromJson(parsed);
            if (piece.startsWith('\x00REPLACE:')) {
              entry.text = piece.slice(9);
            } else if (piece) {
              entry.text = piece;
            }
          } catch (_) {}
        }
        entry.done = true;
        entry.updatedAt = Date.now();
      } catch (_) {
        entry.done = true;
        entry.updatedAt = Date.now();
      } finally {
        if (reader) {
          if (hitLimit) {
            try { await reader.cancel(); } catch (_) {}
          }
          try { reader.releaseLock(); } catch (_) {}
        }
      }
    };

    if (typeof window.fetch === 'function') {
      const origFetch = window.fetch.bind(window);
      window.fetch = function(input, init) {
        let url = '';
        let method = 'GET';
        try {
          url = typeof input === 'string' ? input : (input && input.url ? String(input.url) : '');
          method = ((init && init.method) || (input && input.method) || 'GET').toUpperCase();
        } catch (_) {}
        const p = origFetch(input, init);
        try {
          if (!url || TELEMETRY_RE.test(url)) return p;
          const startedAt = Date.now();
          const entry = {
            id: ++state.seq,
            url: url.slice(0, 220),
            method,
            startedAt,
            updatedAt: startedAt,
            elapsedMs: 0,
            status: 0,
            contentType: '',
            isStream: false,
            done: false,
            text: '',
            errorSnippet: ''
          };
          pushEntry(entry);
          p.then((resp) => {
            try {
              if (!resp) { entry.done = true; return; }
              const nowTs = Date.now();
              entry.status = resp.status || 0;
              entry.updatedAt = nowTs;
              entry.elapsedMs = nowTs - startedAt;
              const ct = (resp.headers && resp.headers.get ? (resp.headers.get('content-type') || '') : '').toLowerCase();
              entry.contentType = ct;
              if (entry.status >= 400 && resp.clone) {
                resp.clone().text().then((t) => {
                  entry.errorSnippet = (t || '').replace(/\s+/g, ' ').trim().slice(0, 240);
                  entry.done = true;
                  entry.updatedAt = Date.now();
                }).catch(() => { entry.done = true; });
                return;
              }
              if (resp.clone && url.includes('/api/') && ct.includes('json')) {
                resp.clone().json().then((d) => {
                  if (d && typeof d === 'object') {
                    const hasErr = Boolean(d.error || d.errors || d.code || d.ok === false || d.authenticated === false);
                    if (hasErr || url.includes('/api/session') || url.includes('/api/auth/')) {
                      entry.errorSnippet = JSON.stringify({
                        keys: Object.keys(d).slice(0, 12),
                        error: d.error,
                        errors: d.errors,
                        code: d.code,
                        message: d.message,
                        status: d.status,
                        vm_resolution_issue: d.vm_resolution_issue,
                        ok: d.ok,
                        authenticated: d.authenticated
                      }).slice(0, 240);
                    }
                  }
                }).catch(() => {});
              }
              const looksStream = ct.includes('event-stream') || ct.includes('ndjson') || ct.includes('stream');
              const looksChatPost = method === 'POST' && (ct.includes('json') || ct.includes('text') || looksStream);
              if (resp.ok && resp.clone && method === 'POST' && (looksStream || looksChatPost)) {
                entry.isStream = looksStream;
                void consumeStream(resp.clone(), entry);
              } else {
                entry.done = true;
              }
            } catch (_) {
              entry.done = true;
            }
          }).catch((err) => {
            const nowTs = Date.now();
            entry.done = true;
            entry.updatedAt = nowTs;
            entry.elapsedMs = nowTs - startedAt;
            entry.errorSnippet = String(err || '').slice(0, 200);
          });
        } catch (_) {}
        return p;
      };
    }

    if (typeof window.EventSource === 'function') {
      const OrigES = window.EventSource;
      window.EventSource = new Proxy(OrigES, {
        construct(target, args, newTarget) {
          const es = Reflect.construct(target, args, newTarget);
          try {
            const u = String(args && args[0] ? args[0] : '');
            if (!TELEMETRY_RE.test(u)) {
              es.addEventListener('message', (ev) => {
                state.esLastAt = Date.now();
                const raw = String(ev && ev.data ? ev.data : '').trim();
                if (!raw || raw === '[DONE]') return;
                try {
                  const parsed = JSON.parse(raw);
                  const piece = pickTextFromJson(parsed);
                  if (piece.startsWith('\x00REPLACE:')) state.esText = piece.slice(9);
                  else if (piece) state.esText += piece;
                } catch (_) {}
              });
            }
          } catch (_) {}
          return es;
        }
      });
    }

    if (typeof window.WebSocket === 'function') {
      const OrigWS = window.WebSocket;
      window.WebSocket = new Proxy(OrigWS, {
        construct(target, args, newTarget) {
          const ws = Reflect.construct(target, args, newTarget);
          try {
            const u = String(args && args[0] ? args[0] : '');
            if (!TELEMETRY_RE.test(u)) {
              state.wsEvents.push({ t: 'open', u: u.slice(0, 120) });
              if (state.wsEvents.length > 10) state.wsEvents.shift();
              ws.addEventListener('error', () => {
                state.wsEvents.push({ t: 'err', u: u.slice(0, 120) });
                if (state.wsEvents.length > 10) state.wsEvents.shift();
              });
              ws.addEventListener('close', (ev) => {
                state.wsEvents.push({ t: 'close', code: ev && ev.code, reason: ev && ev.reason });
                if (state.wsEvents.length > 10) state.wsEvents.shift();
              });
              ws.addEventListener('message', (ev) => {
                state.wsLastAt = Date.now();
                if (typeof ev.data === 'string' && ev.data.startsWith('{')) {
                  try {
                    const parsed = JSON.parse(ev.data);
                    const piece = pickTextFromJson(parsed);
                    if (piece.startsWith('\x00REPLACE:')) state.wsText = piece.slice(9);
                    else if (piece && piece.length <= 32000) state.wsText += piece;
                  } catch (_) {}
                }
              });
            }
          } catch (_) {}
          return ws;
        }
      });
    }
  } catch (_) {}
})()"#;

#[cfg(test)]
mod tests {
  use super::{
    get_adapters_snapshot, reset_overrides, resolve_adapter_for_host, validate_and_save_overrides,
  };
  use std::fs;
  use uuid::Uuid;

  fn temp_dir(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("maam-{tag}-{}", Uuid::new_v4().simple()));
    fs::create_dir_all(&p).expect("create temp dir");
    p
  }

  #[test]
  fn host_matching_resolves_builtin_and_generic_fallback() {
    let dir = temp_dir("adapters-builtin");
    assert_eq!(resolve_adapter_for_host(&dir, "muse.ai").id, "muse");
    assert_eq!(resolve_adapter_for_host(&dir, "sub.muse.ai").id, "muse");
    assert_eq!(resolve_adapter_for_host(&dir, "app.hark.com").id, "hark");
    assert_eq!(resolve_adapter_for_host(&dir, "unknown.example.org").id, "generic");
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn overrides_hot_reload_validate_and_reset() {
    let dir = temp_dir("adapters-override");
    let bad = validate_and_save_overrides(&dir, "{ invalid json");
    assert!(bad.is_err());

    let custom_json = r##"
      [
        {
          "id": "custom_web",
          "name": "Custom Web",
          "hostPatterns": ["example.com", "*.example.com"],
          "composerSelectors": ["[contenteditable=\"true\"]"],
          "preferEnterSubmit": true
        },
        {
          "id": "muse",
          "name": "Muse Custom",
          "hostPatterns": ["muse.ai"],
          "composerSelectors": ["#custom-muse-box"],
          "preferEnterSubmit": true
        }
      ]
    "##;

    let snap = validate_and_save_overrides(&dir, custom_json).expect("save valid overrides");
    assert!(snap.has_overrides);
    assert_eq!(resolve_adapter_for_host(&dir, "www.example.com").id, "custom_web");
    let muse = resolve_adapter_for_host(&dir, "muse.ai");
    assert_eq!(muse.name, "Muse Custom");
    assert_eq!(muse.composer_selectors, vec!["#custom-muse-box".to_string()]);
    assert!(muse.prefer_enter_submit);
    assert_eq!(resolve_adapter_for_host(&dir, "other.example.net").id, "generic");

    let reset_snap = reset_overrides(&dir).expect("reset overrides");
    assert!(!reset_snap.has_overrides);
    assert_eq!(resolve_adapter_for_host(&dir, "www.example.com").id, "generic");
    assert_eq!(resolve_adapter_for_host(&dir, "muse.ai").name, "Muse · Meta");
    assert!(!get_adapters_snapshot(&dir).has_overrides);
    let _ = fs::remove_dir_all(&dir);
  }
}
