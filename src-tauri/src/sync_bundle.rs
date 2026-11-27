use crate::platform::ImportedCookieSpec;
use crate::storage::{
  now_ms, AccountRecord, ExternalAgentRecord, GroupChatRecord,
};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::{STANDARD as B64, URL_SAFE_NO_PAD};
use base64::Engine;
use hmac::Hmac;
use pbkdf2::pbkdf2_hmac;
use qrcode::render::svg as qr_svg;
use qrcode::QrCode;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use url::Url;

pub const SYNC_BUNDLE_MAGIC: &str = "MAAM_SYNC";
pub const SYNC_BUNDLE_VERSION: u32 = 1;
pub const PBKDF2_ITERATIONS: u32 = 120_000;
pub const MAX_SYNC_BODY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_RELAY_BODY_BYTES: usize = 2 * 1024 * 1024;
pub const OFFICIAL_RELAY_URL: &str = match option_env!("MAAM_RELAY_URL") {
  Some(v) => v,
  None => "https://sync.ldjx7.qzz.io",
};
type _HmacCheck = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncBundlePayload {
  pub schema_version: u32,
  pub exported_at_ms: u64,
  pub source_platform: String,
  pub accounts: Vec<AccountRecord>,
  #[serde(default)]
  pub account_cookies: BTreeMap<String, Vec<ImportedCookieSpec>>,
  #[serde(default)]
  pub group_chats: Vec<GroupChatRecord>,
  #[serde(default)]
  pub external_agents: Vec<ExternalAgentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedSyncEnvelope {
  pub magic: String,
  pub version: u32,
  pub kdf: String,
  pub iterations: u32,
  pub cipher: String,
  pub salt_b64: String,
  pub nonce_b64: String,
  pub ciphertext_b64: String,
  pub created_at_ms: u64,
  pub source_platform: String,
  pub account_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSyncBundleResponse {
  pub filename: String,
  pub saved_path: Option<String>,
  pub download_id: Option<u64>,
  pub bundle_json: String,
  pub account_count: usize,
  pub cookie_account_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncImportSummary {
  pub added_accounts: usize,
  pub updated_accounts: usize,
  pub imported_cookie_accounts: usize,
  pub skipped_cookie_accounts: usize,
  pub merged_groups: usize,
  pub merged_external_agents: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanSyncShareInfo {
  pub mode: String,
  pub relay_url: Option<String>,
  pub address: String,
  pub passphrase: String,
  pub sync_uri: String,
  pub qr_svg: String,
  pub expires_at_ms: u64,
  pub max_downloads: usize,
  pub account_count: usize,
  pub cookie_account_count: usize,
}

#[derive(Debug, Clone)]
pub struct ActiveRelayDrop {
  pub relay_url: String,
  pub channel_id: String,
  pub delete_token: String,
}

pub struct PreparedRelayDrop {
  pub channel_id: String,
  pub delete_token: String,
  pub upload_json: String,
  pub sync_uri: String,
  pub qr_svg: String,
  pub expires_at_ms: u64,
  pub max_downloads: usize,
}

#[derive(Debug)]
pub struct ParsedRelayTarget {
  pub relay_base_url: String,
  pub channel_id: String,
  pub key: [u8; 32],
  pub claim_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelayDropUploadBody {
  id: String,
  nonce: String,
  ciphertext: String,
  token_hash: String,
  delete_token_hash: String,
  ttl_secs: u64,
  max_downloads: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelayDropDownloadBody {
  nonce: String,
  ciphertext: String,
}

pub struct LanServerStarted {
  pub address: String,
  pub sync_uri: String,
  pub qr_svg: String,
  pub expires_at_ms: u64,
  pub max_downloads: usize,
  pub cancel_tx: oneshot::Sender<()>,
}

pub fn validate_sync_passphrase(raw: &str) -> Result<String, String> {
  let trimmed = raw.trim();
  if trimmed.chars().count() < 4 {
    return Err("同步口令至少需要 4 个字符".to_string());
  }
  if trimmed.len() > 256 {
    return Err("同步口令长度不能超过 256 字节".to_string());
  }
  Ok(trimmed.to_string())
}

pub fn encrypt_sync_bundle(payload: &SyncBundlePayload, passphrase: &str) -> Result<String, String> {
  let pass = validate_sync_passphrase(passphrase)?;
  let plaintext = serde_json::to_vec(payload).map_err(|e| format!("序列化同步数据失败: {e}"))?;

  let mut salt = [0u8; 16];
  let mut nonce_bytes = [0u8; 12];
  OsRng.fill_bytes(&mut salt);
  OsRng.fill_bytes(&mut nonce_bytes);

  let mut key = [0u8; 32];
  pbkdf2_hmac::<Sha256>(pass.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

  let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "初始化加密器失败".to_string())?;
  let nonce = Nonce::from_slice(&nonce_bytes);
  let ciphertext = cipher
    .encrypt(nonce, plaintext.as_ref())
    .map_err(|_| "加密同步数据失败".to_string())?;

  let env = EncryptedSyncEnvelope {
    magic: SYNC_BUNDLE_MAGIC.to_string(),
    version: SYNC_BUNDLE_VERSION,
    kdf: "pbkdf2-sha256".to_string(),
    iterations: PBKDF2_ITERATIONS,
    cipher: "aes-256-gcm".to_string(),
    salt_b64: B64.encode(salt),
    nonce_b64: B64.encode(nonce_bytes),
    ciphertext_b64: B64.encode(ciphertext),
    created_at_ms: payload.exported_at_ms,
    source_platform: payload.source_platform.clone(),
    account_count: payload.accounts.len(),
  };

  serde_json::to_string(&env).map_err(|e| format!("生成加密包失败: {e}"))
}

pub fn decrypt_sync_bundle(bundle_json: &str, passphrase: &str) -> Result<SyncBundlePayload, String> {
  let pass = validate_sync_passphrase(passphrase)?;
  let trimmed = bundle_json.trim();
  if trimmed.is_empty() {
    return Err("同步备份内容不能为空".to_string());
  }
  if trimmed.len() > MAX_SYNC_BODY_BYTES {
    return Err("同步备份文件过大".to_string());
  }
  let env: EncryptedSyncEnvelope =
    serde_json::from_str(trimmed).map_err(|_| "同步备份格式无效，请确认使用 .maam 加密文件".to_string())?;
  if env.magic != SYNC_BUNDLE_MAGIC || env.version != SYNC_BUNDLE_VERSION {
    return Err("不支持的同步备份版本".to_string());
  }
  if env.cipher != "aes-256-gcm" || env.kdf != "pbkdf2-sha256" {
    return Err("不支持的加密算法".to_string());
  }
  if !(10_000..=1_000_000).contains(&env.iterations) {
    return Err("同步备份加密参数无效".to_string());
  }

  let salt = B64
    .decode(env.salt_b64.trim())
    .map_err(|_| "同步备份数据损坏（salt）".to_string())?;
  let nonce_bytes = B64
    .decode(env.nonce_b64.trim())
    .map_err(|_| "同步备份数据损坏（nonce）".to_string())?;
  let ciphertext = B64
    .decode(env.ciphertext_b64.trim())
    .map_err(|_| "同步备份数据损坏（ciphertext）".to_string())?;
  if nonce_bytes.len() != 12 {
    return Err("同步备份 nonce 长度无效".to_string());
  }

  let mut key = [0u8; 32];
  pbkdf2_hmac::<Sha256>(pass.as_bytes(), &salt, env.iterations, &mut key);
  let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "初始化解密器失败".to_string())?;
  let nonce = Nonce::from_slice(&nonce_bytes);
  let plaintext = cipher
    .decrypt(nonce, ciphertext.as_ref())
    .map_err(|_| "同步口令错误或备份文件已损坏".to_string())?;

  serde_json::from_slice::<SyncBundlePayload>(&plaintext)
    .map_err(|_| "同步备份内容解析失败".to_string())
}

pub fn generate_lan_pin() -> String {
  let mut buf = [0u8; 4];
  OsRng.fill_bytes(&mut buf);
  let val = 100_000 + (u32::from_le_bytes(buf) % 900_000);
  format!("{val:06}")
}

pub fn derive_lan_token(port: u16, passphrase: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(format!("maam-lan-v1:{port}:{}", passphrase.trim()).as_bytes());
  let digest = hasher.finalize();
  digest[..16]
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect::<String>()
}

pub fn render_qr_svg(content: &str) -> Result<String, String> {
  let code =
    QrCode::new(content.as_bytes()).map_err(|e| format!("生成二维码失败: {e}"))?;
  Ok(
    code
      .render::<qr_svg::Color>()
      .min_dimensions(200, 200)
      .quiet_zone(true)
      .dark_color(qr_svg::Color("#0f172a"))
      .light_color(qr_svg::Color("#ffffff"))
      .build(),
  )
}

fn is_usable_lan_ipv4(ip: Ipv4Addr) -> bool {
  ip.is_private()
    && !ip.is_loopback()
    && !ip.is_link_local()
    && !ip.is_unspecified()
    && !ip.is_broadcast()
    && !ip.is_multicast()
}

fn is_virtual_iface_name(name: &str) -> bool {
  let lower = name.trim().to_ascii_lowercase();
  if lower == "lo"
    || (lower.starts_with("lo")
      && lower
        .as_bytes()
        .get(2)
        .is_some_and(|b| b.is_ascii_digit()))
    || (lower.starts_with("ap")
      && lower
        .as_bytes()
        .get(2)
        .is_some_and(|b| b.is_ascii_digit()))
  {
    return true;
  }

  const VIRTUAL_PREFIXES: &[&str] = &[
    "utun", "tun", "tap", "wg", "zt", "br-", "veth", "bridge", "awdl", "llw", "anpi", "pdp_ip",
    "vmnet", "vbox", "ppp", "gif", "stf", "ipsec", "virbr", "feth", "rmnet", "ccmni",
  ];
  if VIRTUAL_PREFIXES.iter().any(|p| lower.starts_with(p)) {
    return true;
  }

  const VIRTUAL_SUBSTRINGS: &[&str] = &[
    "tailscale",
    "zerotier",
    "docker",
    "vethernet",
    "hyper-v",
    "vmware",
    "virtualbox",
    "wsl",
    "clash",
    "mihomo",
    "surge",
    "sing-box",
    "shadowrocket",
    "wintun",
    "wireguard",
    "openvpn",
    "vpn",
    "loopback",
    "pseudo",
    "teredo",
    "isatap",
    "bluetooth",
  ];
  VIRTUAL_SUBSTRINGS.iter().any(|s| lower.contains(s))
}

fn score_lan_candidate(iface_name: &str, ip: Ipv4Addr) -> i32 {
  let lower = iface_name.trim().to_ascii_lowercase();
  let mut score = if matches!(
    lower.as_str(),
    "en0" | "wlan0" | "wi-fi" | "wifi" | "wlan"
  ) {
    110
  } else if (lower.starts_with("en")
    && lower
      .as_bytes()
      .get(2)
      .is_some_and(|b| b.is_ascii_digit()))
    || lower.starts_with("wlan")
    || lower.starts_with("eth")
    || lower.contains("wi-fi")
    || lower.contains("wifi")
    || lower.contains("ethernet")
    || iface_name.contains("以太网")
    || iface_name.contains("本地连接")
  {
    100
  } else {
    20
  };

  let octets = ip.octets();
  score += if octets[0] == 192 && octets[1] == 168 {
    30
  } else if octets[0] == 10 {
    20
  } else if octets[0] == 172 && (16..=31).contains(&octets[1]) {
    10
  } else {
    0
  };
  score
}

fn select_best_lan_ipv4(candidates: &[(&str, Ipv4Addr)]) -> Option<Ipv4Addr> {
  candidates
    .iter()
    .copied()
    .filter(|(name, ip)| !is_virtual_iface_name(name) && is_usable_lan_ipv4(*ip))
    .max_by_key(|(name, ip)| score_lan_candidate(name, *ip))
    .map(|(_, ip)| ip)
}

pub fn detect_lan_ipv4() -> Option<Ipv4Addr> {
  if let Ok(ifaces) = if_addrs::get_if_addrs() {
    let candidates: Vec<(&str, Ipv4Addr)> = ifaces
      .iter()
      .filter(|iface| !iface.is_loopback())
      .filter_map(|iface| match iface.ip() {
        IpAddr::V4(ip) => Some((iface.name.as_str(), ip)),
        IpAddr::V6(_) => None,
      })
      .collect();
    if let Some(ip) = select_best_lan_ipv4(&candidates) {
      return Some(ip);
    }
  }

  for probe in [
    "192.168.1.1:80",
    "192.168.0.1:80",
    "10.0.0.1:80",
    "172.16.0.1:80",
  ] {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
      if sock.connect(probe).is_ok() {
        if let Ok(SocketAddr::V4(addr)) = sock.local_addr() {
          let ip = *addr.ip();
          if is_usable_lan_ipv4(ip) {
            return Some(ip);
          }
        }
      }
    }
  }
  None
}

fn percent_encode_component(raw: &str) -> String {
  let mut out = String::with_capacity(raw.len());
  for &b in raw.as_bytes() {
    if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
      out.push(b as char);
    } else {
      out.push_str(&format!("%{b:02X}"));
    }
  }
  out
}

fn percent_decode_component(raw: &str) -> String {
  let bytes = raw.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut i = 0usize;
  while i < bytes.len() {
    if bytes[i] == b'%' && i + 2 < bytes.len() {
      if let (Some(h), Some(l)) = (
        (bytes[i + 1] as char).to_digit(16),
        (bytes[i + 2] as char).to_digit(16),
      ) {
        out.push(((h << 4) | l) as u8);
        i += 3;
        continue;
      }
    }
    out.push(bytes[i]);
    i += 1;
  }
  String::from_utf8_lossy(&out).to_string()
}

pub fn parse_lan_target(
  raw_address: &str,
  raw_passphrase: &str,
) -> Result<(Ipv4Addr, u16, String), String> {
  let mut addr_str = raw_address.trim().to_string();
  let mut pass_str = raw_passphrase.trim().to_string();

  if let Some((before_hash, frag)) = addr_str.split_once('#') {
    if pass_str.is_empty() && !frag.trim().is_empty() {
      pass_str = percent_decode_component(frag.trim());
    }
    addr_str = before_hash.trim().to_string();
  }

  for prefix in ["maam-sync://", "http://", "https://"] {
    if let Some(rest) = addr_str.strip_prefix(prefix) {
      addr_str = rest.to_string();
      break;
    }
  }
  if let Some((host_port, _)) = addr_str.split_once('/') {
    addr_str = host_port.to_string();
  }
  let (host_str, port_str) = addr_str
    .trim()
    .rsplit_once(':')
    .ok_or_else(|| "请输入有效的局域网同步地址（如 192.168.1.10:52345）".to_string())?;

  let ip: Ipv4Addr = host_str
    .trim()
    .parse()
    .map_err(|_| "仅支持局域网 IPv4 地址".to_string())?;
  if !(ip.is_private() || ip.is_loopback() || ip.is_link_local()) {
    return Err("出于安全考虑，局域网同步仅允许内网或本机 IPv4 地址".to_string());
  }
  let port: u16 = port_str
    .trim()
    .parse()
    .map_err(|_| "同步端口号无效".to_string())?;
  if port == 0 {
    return Err("同步端口号无效".to_string());
  }

  let pass = validate_sync_passphrase(&pass_str)?;
  Ok((ip, port, pass))
}

pub async fn spawn_lan_sync_server(
  encrypted_json: String,
  passphrase: String,
  max_downloads: usize,
  ttl_secs: u64,
) -> Result<LanServerStarted, String> {
  let listener = TcpListener::bind("0.0.0.0:0")
    .await
    .map_err(|e| format!("启动局域网同步服务失败: {e}"))?;
  let port = listener
    .local_addr()
    .map_err(|e| format!("获取同步端口失败: {e}"))?
    .port();
  let ip = detect_lan_ipv4()
    .ok_or_else(|| "未检测到可用的局域网 IPv4 地址，请确认设备已连接 Wi-Fi 或局域网".to_string())?;
  let address = format!("{ip}:{port}");
  let sync_uri = format!("maam-sync://{address}#{}", percent_encode_component(&passphrase));
  let qr_svg = render_qr_svg(&sync_uri)?;
  let expected_token = derive_lan_token(port, &passphrase);
  let expires_at_ms = now_ms() + ttl_secs.saturating_mul(1000);
  let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();

  let body_bytes = encrypted_json.into_bytes();
  tokio::spawn(async move {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(ttl_secs);
    let mut remaining = max_downloads.max(1);
    let mut bad_attempts = 0usize;

    loop {
      if remaining == 0 || bad_attempts >= 5 {
        break;
      }
      tokio::select! {
        _ = &mut cancel_rx => {
          break;
        }
        _ = tokio::time::sleep_until(deadline) => {
          break;
        }
        accepted = listener.accept() => {
          let Ok((mut stream, _)) = accepted else {
            break;
          };
          let mut buf = [0u8; 4096];
          let read_res = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await;
          let Ok(Ok(n)) = read_res else {
            continue;
          };
          if n == 0 {
            continue;
          }
          let req_text = String::from_utf8_lossy(&buf[..n]);
          let mut lines = req_text.lines();
          let first_line = lines.next().unwrap_or("");
          let is_get_sync = first_line.starts_with("GET /sync ") || first_line.starts_with("GET /sync?");
          if !is_get_sync {
            let _ = stream
              .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
              .await;
            continue;
          }
          let mut got_token = String::new();
          for line in lines {
            if let Some((k, v)) = line.split_once(':') {
              if k.trim().eq_ignore_ascii_case("x-sync-token") {
                got_token = v.trim().to_string();
              }
            }
          }
          if got_token != expected_token {
            bad_attempts += 1;
            let _ = stream
              .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
              .await;
            continue;
          }

          let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            body_bytes.len()
          );
          if stream.write_all(header.as_bytes()).await.is_ok()
            && stream.write_all(&body_bytes).await.is_ok()
          {
            let _ = stream.flush().await;
            remaining = remaining.saturating_sub(1);
          }
        }
      }
    }
  });

  Ok(LanServerStarted {
    address,
    sync_uri,
    qr_svg,
    expires_at_ms,
    max_downloads: max_downloads.max(1),
    cancel_tx,
  })
}

pub async fn fetch_lan_sync_bundle(
  ip: Ipv4Addr,
  port: u16,
  passphrase: &str,
) -> Result<String, String> {
  let token = derive_lan_token(port, passphrase);
  let mut stream = tokio::time::timeout(
    Duration::from_secs(10),
    tokio::net::TcpStream::connect((ip, port)),
  )
  .await
  .map_err(|_| "连接发送端超时，请确认两台设备在同一局域网且未关闭共享弹窗".to_string())?
  .map_err(|e| format!("无法连接到 {ip}:{port}：{e}"))?;

  let req = format!(
    "GET /sync HTTP/1.1\r\nHost: {ip}:{port}\r\nX-Sync-Token: {token}\r\nConnection: close\r\n\r\n"
  );
  stream
    .write_all(req.as_bytes())
    .await
    .map_err(|e| format!("发送同步请求失败: {e}"))?;

  let mut raw = Vec::new();
  let mut chunk = [0u8; 8192];
  loop {
    let n = tokio::time::timeout(Duration::from_secs(20), stream.read(&mut chunk))
      .await
      .map_err(|_| "接收同步数据超时".to_string())?
      .map_err(|e| format!("读取同步数据失败: {e}"))?;
    if n == 0 {
      break;
    }
    if raw.len() + n > MAX_SYNC_BODY_BYTES + 16384 {
      return Err("同步数据体积超出限制".to_string());
    }
    raw.extend_from_slice(&chunk[..n]);
  }

  let sep = raw
    .windows(4)
    .position(|w| w == b"\r\n\r\n")
    .ok_or_else(|| "收到无效的局域网同步响应".to_string())?;
  let head = String::from_utf8_lossy(&raw[..sep]);
  let status_line = head.lines().next().unwrap_or("");
  if status_line.contains(" 403 ") {
    return Err("同步口令不正确，请核对后重试".to_string());
  }
  if !status_line.contains(" 200 ") {
    return Err(format!("发送端拒绝或已停止共享（{status_line}）"));
  }
  let body = std::str::from_utf8(&raw[sep + 4..])
    .map_err(|_| "同步响应编码无效".to_string())?
    .trim()
    .to_string();
  if body.is_empty() {
    return Err("同步响应为空".to_string());
  }
  Ok(body)
}

pub fn official_relay_url() -> Option<String> {
  let trimmed = OFFICIAL_RELAY_URL.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(trimmed.trim_end_matches('/').to_string())
  }
}

pub fn normalize_relay_base_url(raw: &str) -> Result<String, String> {
  let candidate = raw.trim();
  let effective = if candidate.is_empty() {
    OFFICIAL_RELAY_URL.trim()
  } else {
    candidate
  };
  if effective.is_empty() {
    return Err("请先填写 Cloudflare Worker 中转地址（https://...）".to_string());
  }
  let parsed = Url::parse(effective).map_err(|_| "中转服务地址格式无效".to_string())?;
  if parsed.scheme() != "https" {
    return Err("出于安全考虑，加密中转通道仅支持 HTTPS 地址".to_string());
  }
  if !parsed.username().is_empty() || parsed.password().is_some() {
    return Err("中转服务地址不能包含认证凭据".to_string());
  }
  if parsed.query().is_some() || parsed.fragment().is_some() {
    return Err("中转服务地址不应包含查询参数或片段".to_string());
  }
  let host = parsed
    .host_str()
    .map(str::trim)
    .filter(|h| !h.is_empty())
    .ok_or_else(|| "中转服务地址缺少域名".to_string())?;
  let lower_host = host.to_ascii_lowercase();
  if lower_host == "localhost" || lower_host.ends_with(".local") {
    return Err("中转服务地址必须是公网 HTTPS 域名".to_string());
  }
  if let Ok(ip) = host.parse::<IpAddr>() {
    match ip {
      IpAddr::V4(v4) => {
        if v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified() {
          return Err("中转服务地址必须是公网 HTTPS 域名".to_string());
        }
      }
      IpAddr::V6(v6) => {
        if v6.is_loopback() || v6.is_unspecified() {
          return Err("中转服务地址必须是公网 HTTPS 域名".to_string());
        }
      }
    }
  }
  Ok(parsed.as_str().trim_end_matches('/').to_string())
}

fn sha256_hex(input: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(input);
  let digest = hasher.finalize();
  digest.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

fn random_hex(len_bytes: usize) -> String {
  let mut buf = vec![0u8; len_bytes];
  OsRng.fill_bytes(&mut buf);
  buf.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

fn random_b64url(len_bytes: usize) -> String {
  let mut buf = vec![0u8; len_bytes];
  OsRng.fill_bytes(&mut buf);
  URL_SAFE_NO_PAD.encode(&buf)
}

pub fn prepare_relay_drop(
  payload: &SyncBundlePayload,
  relay_base_url: &str,
  max_downloads: usize,
  ttl_secs: u64,
) -> Result<PreparedRelayDrop, String> {
  let base_url = normalize_relay_base_url(relay_base_url)?;
  let plaintext = serde_json::to_vec(payload).map_err(|e| format!("序列化同步数据失败: {e}"))?;

  let mut key = [0u8; 32];
  let mut nonce_bytes = [0u8; 12];
  OsRng.fill_bytes(&mut key);
  OsRng.fill_bytes(&mut nonce_bytes);

  let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "初始化加密器失败".to_string())?;
  let nonce = Nonce::from_slice(&nonce_bytes);
  let ciphertext = cipher
    .encrypt(nonce, plaintext.as_ref())
    .map_err(|_| "端到端加密同步数据失败".to_string())?;

  let channel_id = random_hex(16);
  let claim_token = random_b64url(16);
  let delete_token = random_b64url(16);
  let ttl = ttl_secs.clamp(30, 300);
  let max_dl = max_downloads.clamp(1, 3);

  let upload = RelayDropUploadBody {
    id: channel_id.clone(),
    nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
    ciphertext: B64.encode(ciphertext),
    token_hash: sha256_hex(claim_token.as_bytes()),
    delete_token_hash: sha256_hex(delete_token.as_bytes()),
    ttl_secs: ttl,
    max_downloads: max_dl,
  };
  let upload_json =
    serde_json::to_string(&upload).map_err(|e| format!("生成加密中转负载失败: {e}"))?;
  if upload_json.len() > MAX_RELAY_BODY_BYTES {
    return Err("同步数据加密后超过 2 MB 中转限制，请改用「加密备份文件」导出同步".to_string());
  }

  let key_b64 = URL_SAFE_NO_PAD.encode(key);
  let sync_uri = format!(
    "maam-sync://relay/{channel_id}?u={}#{key_b64}.{claim_token}",
    percent_encode_component(&base_url)
  );
  let qr_svg = render_qr_svg(&sync_uri)?;
  let expires_at_ms = now_ms() + ttl.saturating_mul(1000);

  Ok(PreparedRelayDrop {
    channel_id,
    delete_token,
    upload_json,
    sync_uri,
    qr_svg,
    expires_at_ms,
    max_downloads: max_dl,
  })
}

pub fn is_relay_sync_uri(raw: &str) -> bool {
  raw.trim().starts_with("maam-sync://relay/")
}

pub fn parse_relay_target(
  raw_address: &str,
  raw_fragment: &str,
) -> Result<ParsedRelayTarget, String> {
  let mut addr_str = raw_address.trim().to_string();
  let mut frag_str = raw_fragment.trim().to_string();

  if let Some((before_hash, hash_part)) = addr_str.split_once('#') {
    if frag_str.is_empty() && !hash_part.trim().is_empty() {
      frag_str = percent_decode_component(hash_part.trim());
    }
    addr_str = before_hash.trim().to_string();
  }

  let rest = addr_str
    .strip_prefix("maam-sync://relay/")
    .ok_or_else(|| "中转同步串格式无效".to_string())?;
  let (id_part, query_part) = match rest.split_once('?') {
    Some((id, q)) => (id.trim(), Some(q.trim())),
    None => (rest.trim(), None),
  };

  if !(24..=64).contains(&id_part.len())
    || !id_part.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
  {
    return Err("中转通道 ID 无效".to_string());
  }

  let mut raw_u = String::new();
  if let Some(q) = query_part {
    for pair in q.split('&') {
      if let Some((k, v)) = pair.split_once('=') {
        if k.trim() == "u" {
          raw_u = percent_decode_component(v.trim());
          break;
        }
      }
    }
  }
  let relay_base_url = normalize_relay_base_url(&raw_u)?;

  let (key_b64, claim_token) = frag_str
    .trim()
    .split_once('.')
    .ok_or_else(|| "中转密钥片段无效或缺失，无法端到端解密".to_string())?;
  let key_vec = URL_SAFE_NO_PAD
    .decode(key_b64.trim())
    .map_err(|_| "端到端加密密钥损坏".to_string())?;
  let key: [u8; 32] = key_vec
    .try_into()
    .map_err(|_| "端到端加密密钥长度无效".to_string())?;
  let token = claim_token.trim();
  if token.is_empty() || token.len() > 256 {
    return Err("中转取件令牌无效".to_string());
  }

  Ok(ParsedRelayTarget {
    relay_base_url,
    channel_id: id_part.to_string(),
    key,
    claim_token: token.to_string(),
  })
}

pub fn decrypt_relay_drop(response_json: &str, key: &[u8; 32]) -> Result<SyncBundlePayload, String> {
  let drop: RelayDropDownloadBody =
    serde_json::from_str(response_json.trim()).map_err(|_| "中转响应格式无效".to_string())?;
  let nonce_vec = URL_SAFE_NO_PAD
    .decode(drop.nonce.trim())
    .map_err(|_| "中转密文 nonce 损坏".to_string())?;
  if nonce_vec.len() != 12 {
    return Err("中转密文 nonce 长度无效".to_string());
  }
  let ciphertext = B64
    .decode(drop.ciphertext.trim())
    .map_err(|_| "中转密文数据损坏".to_string())?;

  let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "初始化解密器失败".to_string())?;
  let nonce = Nonce::from_slice(&nonce_vec);
  let plaintext = cipher
    .decrypt(nonce, ciphertext.as_ref())
    .map_err(|_| "端到端解密失败：密钥不匹配或密文已被篡改".to_string())?;

  serde_json::from_slice::<SyncBundlePayload>(&plaintext)
    .map_err(|_| "解密后同步数据解析失败".to_string())
}

async fn relay_http_request(
  app: &tauri::AppHandle,
  method: &str,
  url: &str,
  headers: HashMap<String, String>,
  body: Option<String>,
) -> Result<(u16, String), String> {
  #[cfg(desktop)]
  {
    let _ = app;
    let client = reqwest::Client::builder()
      .user_agent("MultiAgentAccountManager/0.2")
      .https_only(true)
      .redirect(reqwest::redirect::Policy::none())
      .connect_timeout(Duration::from_secs(15))
      .timeout(Duration::from_secs(25))
      .build()
      .map_err(|e| format!("初始化中转请求失败: {e}"))?;
    let mut req = match method {
      "POST" => client.post(url),
      "DELETE" => client.delete(url),
      _ => client.get(url),
    };
    for (k, v) in headers {
      req = req.header(&k, v);
    }
    if let Some(payload) = body {
      req = req.body(payload);
    }
    let resp = req
      .send()
      .await
      .map_err(|e| format!("连接加密中转通道失败: {e}"))?;
    let status = resp.status().as_u16();
    let text = resp
      .text()
      .await
      .map_err(|e| format!("读取中转响应失败: {e}"))?;
    Ok((status, text))
  }
  #[cfg(mobile)]
  {
    let resp = crate::platform::mobile::http_request(app, method, url, headers, body).await?;
    Ok((resp.status, resp.body))
  }
}

pub async fn upload_relay_drop(
  app: &tauri::AppHandle,
  relay_base_url: &str,
  upload_json: String,
) -> Result<(), String> {
  let url = format!("{relay_base_url}/v1/drop");
  let mut headers = HashMap::new();
  headers.insert("Content-Type".to_string(), "application/json".to_string());
  let (status, body) = relay_http_request(app, "POST", &url, headers, Some(upload_json)).await?;
  if !(200..300).contains(&status) {
    return Err(format!("上传加密中转通道失败（HTTP {status}）：{body}"));
  }
  Ok(())
}

pub async fn fetch_relay_drop(
  app: &tauri::AppHandle,
  target: &ParsedRelayTarget,
) -> Result<String, String> {
  let url = format!("{}/v1/drop/{}", target.relay_base_url, target.channel_id);
  let mut headers = HashMap::new();
  headers.insert("X-Relay-Token".to_string(), target.claim_token.clone());
  let (status, body) = relay_http_request(app, "GET", &url, headers, None).await?;
  match status {
    200..=299 => Ok(body),
    404 => Err("该中转同步码已过期、已被拉取销毁或不存在".to_string()),
    403 => Err("中转取件凭证无效或已超过尝试次数上限".to_string()),
    _ => Err(format!("中转服务异常（HTTP {status}）")),
  }
}

pub async fn delete_relay_drop(
  app: &tauri::AppHandle,
  active: &ActiveRelayDrop,
) -> Result<(), String> {
  let url = format!("{}/v1/drop/{}", active.relay_url, active.channel_id);
  let mut headers = HashMap::new();
  headers.insert("X-Relay-Token".to_string(), active.delete_token.clone());
  let (status, _) = relay_http_request(app, "DELETE", &url, headers, None).await?;
  if status == 404 || (200..300).contains(&status) {
    Ok(())
  } else {
    Err(format!("销毁中转暂存失败（HTTP {status}）"))
  }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavTestResult {
  pub ok: bool,
  pub remote_url: String,
  pub file_exists: bool,
  pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavPushResult {
  pub remote_url: String,
  pub account_count: usize,
  pub cookie_account_count: usize,
  pub size_bytes: usize,
}

fn is_local_or_private_host(host: &str) -> bool {
  let clean = host.trim().trim_start_matches('[').trim_end_matches(']');
  let lower = clean.to_ascii_lowercase();
  if lower == "localhost" || lower.ends_with(".local") {
    return true;
  }
  if let Ok(ip) = lower.parse::<IpAddr>() {
    return match ip {
      IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
      IpAddr::V6(v6) => v6.is_loopback(),
    };
  }
  false
}

pub fn normalize_webdav_target(
  raw_url: &str,
  raw_filename: Option<&str>,
) -> Result<(String, String, String), String> {
  let trimmed_url = raw_url.trim();
  if trimmed_url.is_empty() {
    return Err("请填写 WebDAV 服务地址（如 https://dav.jianguoyun.com/dav/）".to_string());
  }
  let mut parsed = Url::parse(trimmed_url).map_err(|_| "WebDAV 服务地址格式无效".to_string())?;
  let scheme = parsed.scheme().to_ascii_lowercase();
  let host = parsed
    .host_str()
    .map(str::trim)
    .filter(|h| !h.is_empty())
    .ok_or_else(|| "WebDAV 服务地址缺少域名或主机".to_string())?
    .to_string();
  let allow_http = scheme == "http" && is_local_or_private_host(&host);
  if scheme != "https" && !allow_http {
    return Err("公网 WebDAV 仅允许使用 HTTPS 地址".to_string());
  }
  if !parsed.username().is_empty() || parsed.password().is_some() {
    return Err("请将 WebDAV 用户名和密码填写到下方专用输入框，不要写在 URL 中".to_string());
  }
  if parsed.query().is_some() || parsed.fragment().is_some() {
    return Err("WebDAV 地址不应包含查询参数或片段".to_string());
  }

  let raw_file = raw_filename.unwrap_or("").trim();
  let mut clean_filename = if raw_file.is_empty() {
    "maam-sync.maam".to_string()
  } else {
    raw_file.trim_matches('/').to_string()
  };
  let parts: Vec<&str> = clean_filename.split('/').collect();
  if parts.is_empty() || parts.iter().any(|p| p.is_empty() || *p == "." || *p == "..") {
    return Err("WebDAV 同步文件名格式无效".to_string());
  }
  clean_filename = parts.join("/");

  let path_str = parsed.path().trim_end_matches('/').to_string();
  if path_str.to_ascii_lowercase().ends_with(".maam")
    && (raw_file.is_empty() || raw_file.eq_ignore_ascii_case("maam-sync.maam"))
  {
    if let Some((parent, leaf)) = path_str.rsplit_once('/') {
      parsed.set_path(&format!("{}/", parent));
      clean_filename = leaf.to_string();
    }
  } else {
    let dir_path = if path_str.is_empty() {
      "/".to_string()
    } else {
      format!("{path_str}/")
    };
    parsed.set_path(&dir_path);
  }

  let base_dir_url = parsed.as_str().to_string();
  let full_url = parsed
    .join(&clean_filename)
    .map_err(|_| "拼接 WebDAV 文件地址失败".to_string())?
    .to_string();
  Ok((base_dir_url, clean_filename, full_url))
}

fn build_webdav_headers(
  username: &str,
  password: &str,
  content_type: Option<&str>,
) -> HashMap<String, String> {
  let mut headers = HashMap::new();
  if !username.trim().is_empty() || !password.is_empty() {
    let creds = format!("{}:{}", username.trim(), password);
    headers.insert("Authorization".to_string(), format!("Basic {}", B64.encode(creds)));
  }
  if let Some(ct) = content_type {
    headers.insert("Content-Type".to_string(), ct.to_string());
  }
  headers
}

async fn webdav_http_request(
  app: &tauri::AppHandle,
  method: &str,
  url: &str,
  headers: HashMap<String, String>,
  body: Option<String>,
) -> Result<(u16, String), String> {
  #[cfg(desktop)]
  {
    let _ = app;
    let client = reqwest::Client::builder()
      .user_agent("MultiAgentAccountManager/0.2")
      .redirect(reqwest::redirect::Policy::none())
      .connect_timeout(Duration::from_secs(15))
      .timeout(Duration::from_secs(30))
      .build()
      .map_err(|e| format!("初始化 WebDAV 请求失败: {e}"))?;
    let http_method = reqwest::Method::from_bytes(method.as_bytes())
      .map_err(|_| "不支持的 HTTP 方法".to_string())?;
    let mut req = client.request(http_method, url);
    for (k, v) in headers {
      req = req.header(&k, v);
    }
    if let Some(payload) = body {
      req = req.body(payload);
    }
    let resp = req
      .send()
      .await
      .map_err(|e| format!("连接 WebDAV 服务失败: {e}"))?;
    let status = resp.status().as_u16();
    let text = resp
      .text()
      .await
      .map_err(|e| format!("读取 WebDAV 响应失败: {e}"))?;
    Ok((status, text))
  }
  #[cfg(mobile)]
  {
    let resp = crate::platform::mobile::http_request(app, method, url, headers, body).await?;
    Ok((resp.status, resp.body))
  }
}

pub async fn test_webdav_connection(
  app: &tauri::AppHandle,
  raw_url: &str,
  username: &str,
  password: &str,
  raw_filename: Option<&str>,
) -> Result<WebDavTestResult, String> {
  let (base_dir_url, _, full_url) = normalize_webdav_target(raw_url, raw_filename)?;
  let headers = build_webdav_headers(username, password, None);
  #[cfg(not(target_os = "android"))]
  let status = {
    let mut propfind_headers = headers.clone();
    propfind_headers.insert("Depth".to_string(), "0".to_string());
    let (st, _) =
      webdav_http_request(app, "PROPFIND", &base_dir_url, propfind_headers, None).await?;
    if st == 405 || st == 501 {
      let (opt_st, _) =
        webdav_http_request(app, "OPTIONS", &base_dir_url, headers.clone(), None).await?;
      if opt_st == 405 {
        webdav_http_request(app, "HEAD", &base_dir_url, headers.clone(), None).await?.0
      } else {
        opt_st
      }
    } else {
      st
    }
  };
  #[cfg(target_os = "android")]
  let status = {
    let (opt_st, _) =
      webdav_http_request(app, "OPTIONS", &base_dir_url, headers.clone(), None).await?;
    if opt_st == 405 {
      webdav_http_request(app, "HEAD", &base_dir_url, headers.clone(), None).await?.0
    } else {
      opt_st
    }
  };
  if status == 401 || status == 403 {
    return Err("WebDAV 认证失败，请检查账号或应用密码".to_string());
  }
  if status == 404 {
    return Err("WebDAV 目录不存在（HTTP 404），请确认路径已在网盘中创建".to_string());
  }
  if !(200..300).contains(&status) {
    return Err(format!("WebDAV 服务异常（HTTP {status}）"));
  }
  let (head_status, _) = webdav_http_request(app, "HEAD", &full_url, headers, None).await?;
  if head_status == 401 {
    return Err("WebDAV 认证失败，请检查账号或应用密码".to_string());
  }
  if head_status == 403 {
    return Err("WebDAV 目标目录无访问权限（HTTP 403），如使用坚果云请指定子目录（例如 /dav/diehao/）".to_string());
  }
  let file_exists = (200..300).contains(&head_status);
  let message = if file_exists {
    "WebDAV 连接成功，已检测到远端同步备份文件".to_string()
  } else {
    "WebDAV 连接成功（远端尚无备份文件，可点击上传创建）".to_string()
  };
  Ok(WebDavTestResult {
    ok: true,
    remote_url: full_url,
    file_exists,
    message,
  })
}

pub async fn upload_webdav_bundle(
  app: &tauri::AppHandle,
  raw_url: &str,
  username: &str,
  password: &str,
  raw_filename: Option<&str>,
  encrypted_json: String,
) -> Result<String, String> {
  let (base_dir_url, clean_filename, full_url) = normalize_webdav_target(raw_url, raw_filename)?;
  let headers = build_webdav_headers(username, password, Some("application/json; charset=utf-8"));
  #[cfg_attr(target_os = "android", allow(unused_mut))]
  let (mut status, _) =
    webdav_http_request(app, "PUT", &full_url, headers.clone(), Some(encrypted_json.clone())).await?;
  #[cfg(not(target_os = "android"))]
  if status == 409 || status == 404 {
    let mkcol_headers = build_webdav_headers(username, password, None);
    let _ = webdav_http_request(app, "MKCOL", &base_dir_url, mkcol_headers.clone(), None).await;
    if let Some((sub_parent, _)) = clean_filename.rsplit_once('/') {
      if let Ok(base_parsed) = Url::parse(&base_dir_url) {
        if let Ok(sub_url) = base_parsed.join(&format!("{sub_parent}/")) {
          let _ = webdav_http_request(app, "MKCOL", sub_url.as_str(), mkcol_headers, None).await;
        }
      }
    }
    (status, _) = webdav_http_request(app, "PUT", &full_url, headers, Some(encrypted_json)).await?;
  }
  #[cfg(target_os = "android")]
  let _ = (base_dir_url, clean_filename);
  match status {
    200..=299 => Ok(full_url),
    401 | 403 => Err("WebDAV 认证失败，请检查账号或应用密码".to_string()),
    404 | 409 => Err(format!(
      "远端目录不存在（HTTP {status}），请先在 WebDAV 网盘中创建对应目录"
    )),
    _ => Err(format!("上传到 WebDAV 失败（HTTP {status}）")),
  }
}

pub async fn download_webdav_bundle(
  app: &tauri::AppHandle,
  raw_url: &str,
  username: &str,
  password: &str,
  raw_filename: Option<&str>,
) -> Result<String, String> {
  let (_, _, full_url) = normalize_webdav_target(raw_url, raw_filename)?;
  let headers = build_webdav_headers(username, password, None);
  let (status, body) = webdav_http_request(app, "GET", &full_url, headers, None).await?;
  match status {
    200..=299 => {
      let trimmed = body.trim().to_string();
      if trimmed.is_empty() {
        return Err("远端 WebDAV 备份文件为空".to_string());
      }
      if trimmed.len() > MAX_SYNC_BODY_BYTES {
        return Err("远端 WebDAV 备份文件体积超出限制".to_string());
      }
      Ok(trimmed)
    }
    401 | 403 => Err("WebDAV 认证失败，请检查账号或应用密码".to_string()),
    404 => Err("远端尚无备份文件，请先在源设备点击「加密上传到 WebDAV」".to_string()),
    _ => Err(format!("从 WebDAV 拉取失败（HTTP {status}）")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::providers::ProviderId;
  use crate::storage::{AccountRecord, AuthState};

  #[test]
  fn encrypt_decrypt_roundtrip_and_wrong_passphrase() {
    let mut cookies = BTreeMap::new();
    cookies.insert(
      "muse:11111111-1111-4111-8111-111111111111".to_string(),
      vec![ImportedCookieSpec {
        name: "sid".to_string(),
        value: "secret-val".to_string(),
        domain: ".muse.ai".to_string(),
        path: "/".to_string(),
        secure: true,
        http_only: true,
        same_site: Some("lax".to_string()),
        max_age_secs: Some(3600),
      }],
    );
    let payload = SyncBundlePayload {
      schema_version: 1,
      exported_at_ms: 1700000000000,
      source_platform: "macos".to_string(),
      accounts: vec![AccountRecord {
        provider_id: ProviderId::muse(),
        uuid: "11111111-1111-4111-8111-111111111111".to_string(),
        alias: "work@example.com".to_string(),
        created_at_ms: 1700000000000,
        auto_alias: false,
        proxy_url: None,
        timezone: Some("America/New_York".to_string()),
        avatar_url: None,
        quota_text: None,
        quota_percent: None,
        quota_detail: None,
        auth_state: AuthState::Authenticated,
        cookie_expires_at_ms: None,
        cookie_session_only: false,
      }],
      account_cookies: cookies,
      group_chats: Vec::new(),
      external_agents: Vec::new(),
    };

    let encrypted = encrypt_sync_bundle(&payload, "pass1234").expect("encrypt ok");
    let err = decrypt_sync_bundle(&encrypted, "wrong999").unwrap_err();
    assert!(err.contains("口令错误"));
    let dec = decrypt_sync_bundle(&encrypted, "pass1234").expect("decrypt ok");
    assert_eq!(dec.accounts.len(), 1);
    assert_eq!(dec.accounts[0].alias, "work@example.com");
    assert_eq!(
      dec.account_cookies["muse:11111111-1111-4111-8111-111111111111"][0].value,
      "secret-val"
    );
  }

  #[test]
  fn parse_lan_target_supports_sync_uri_and_rejects_public_ip() {
    let (ip, port, pass) =
      parse_lan_target("maam-sync://192.168.1.25:54321#123456", "").expect("parse uri");
    assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 25));
    assert_eq!(port, 54321);
    assert_eq!(pass, "123456");

    let err = parse_lan_target("8.8.8.8:8080", "123456").unwrap_err();
    assert!(err.contains("内网"));
  }
}
