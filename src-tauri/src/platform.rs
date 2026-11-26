#[cfg(desktop)]
use std::path::Path;
#[cfg(desktop)]
use uuid::Uuid;

pub fn current_platform_name() -> &'static str {
  #[cfg(target_os = "macos")]
  {
    "macos"
  }
  #[cfg(windows)]
  {
    "windows"
  }
  #[cfg(target_os = "android")]
  {
    "android"
  }
  #[cfg(target_os = "ios")]
  {
    "ios"
  }
  #[cfg(all(
    not(target_os = "macos"),
    not(windows),
    not(target_os = "android"),
    not(target_os = "ios")
  ))]
  {
    "linux"
  }
}

#[cfg(desktop)]
pub fn open_external_browser(url: &str) -> Result<(), String> {
  open::that_detached(url).map_err(|e| format!("打开系统浏览器失败: {e}"))
}

pub const SUPPORTS_ACCOUNT_PROXY: bool = cfg!(any(target_os = "macos", windows, target_os = "ios"));

pub fn parse_account_timezone(raw: &str) -> Result<Option<String>, String> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Ok(None);
  }
  let invalid = || {
    "时区格式无效，请使用 IANA 时区名称（例如 America/New_York、Asia/Tokyo）".to_string()
  };
  if trimmed.len() > 64
    || !trimmed
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-' | b'/'))
  {
    return Err(invalid());
  }
  if trimmed.eq_ignore_ascii_case("UTC") || trimmed.eq_ignore_ascii_case("GMT") {
    return Ok(Some(trimmed.to_ascii_uppercase()));
  }
  if !trimmed.contains('/')
    || trimmed.starts_with('/')
    || trimmed.ends_with('/')
    || trimmed.contains("//")
    || trimmed.contains("..")
  {
    return Err(invalid());
  }
  Ok(Some(trimmed.to_string()))
}

#[cfg_attr(all(not(desktop), not(mobile)), allow(dead_code))]
pub fn build_timezone_override_init_js(timezone: &str) -> String {
  let tz_json = serde_json::to_string(timezone).unwrap_or_else(|_| "\"UTC\"".to_string());
  format!(
    r#"(() => {{
  const TZ = {tz_json};
  if (!TZ || window.__maamTimezoneOverride === TZ) return;
  const NativeDate = window.Date;
  const NativeIntl = window.Intl;
  if (!NativeDate || !NativeIntl || !NativeIntl.DateTimeFormat) return;
  const NativeDTF = NativeIntl.DateTimeFormat;
  try {{
    new NativeDTF('en-US', {{ timeZone: TZ }});
  }} catch (_) {{
    return;
  }}
  window.__maamTimezoneOverride = TZ;

  const nativeResolvedOptions = NativeDTF.prototype.resolvedOptions;
  const nativeFormat = NativeDTF.prototype.format;
  const nativeFormatToParts = NativeDTF.prototype.formatToParts;
  const nativeValueOf = NativeDate.prototype.valueOf;
  const nativeGetUTCFullYear = NativeDate.prototype.getUTCFullYear;
  const nativeGetUTCMonth = NativeDate.prototype.getUTCMonth;
  const nativeGetUTCDate = NativeDate.prototype.getUTCDate;
  const nativeGetUTCDay = NativeDate.prototype.getUTCDay;
  const nativeGetUTCHours = NativeDate.prototype.getUTCHours;
  const nativeGetUTCMinutes = NativeDate.prototype.getUTCMinutes;
  const nativeGetUTCSeconds = NativeDate.prototype.getUTCSeconds;
  const nativeGetUTCMilliseconds = NativeDate.prototype.getUTCMilliseconds;
  const nativeSetTime = NativeDate.prototype.setTime;
  const nativeToLocaleString = NativeDate.prototype.toLocaleString;
  const nativeToLocaleDateString = NativeDate.prototype.toLocaleDateString;
  const nativeToLocaleTimeString = NativeDate.prototype.toLocaleTimeString;

  const partsFmt = new NativeDTF('en-US', {{
    timeZone: TZ,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hourCycle: 'h23'
  }});

  let longNameFmt = null;
  try {{
    longNameFmt = new NativeDTF('en-US', {{ timeZone: TZ, timeZoneName: 'long' }});
  }} catch (_) {{}}

  const getOffsetMinutes = (ms) => {{
    if (!Number.isFinite(ms)) return 0;
    const parts = nativeFormatToParts.call(partsFmt, new NativeDate(ms));
    let y = 1970, mo = 1, d = 1, h = 0, mi = 0, s = 0;
    for (let i = 0; i < parts.length; i++) {{
      const p = parts[i];
      if (p.type === 'year') y = Number(p.value);
      else if (p.type === 'month') mo = Number(p.value);
      else if (p.type === 'day') d = Number(p.value);
      else if (p.type === 'hour') h = Number(p.value) % 24;
      else if (p.type === 'minute') mi = Number(p.value);
      else if (p.type === 'second') s = Number(p.value);
    }}
    const wallUtc = NativeDate.UTC(y, mo - 1, d, h, mi, s);
    const realSec = Math.floor(ms / 1000) * 1000;
    return Math.round((realSec - wallUtc) / 60000);
  }};

  const shiftedUtcDate = (dateObj) => {{
    const ms = nativeValueOf.call(dateObj);
    if (!Number.isFinite(ms)) return null;
    const offsetMin = getOffsetMinutes(ms);
    return new NativeDate(ms - offsetMin * 60000);
  }};

  const injectTzOpt = (options) => {{
    if (!options || typeof options !== 'object') return {{ timeZone: TZ }};
    if (options.timeZone === undefined) return Object.assign({{}}, options, {{ timeZone: TZ }});
    return options;
  }};

  const WrappedDTF = function DateTimeFormat(locales, options) {{
    const nextOpts = injectTzOpt(options);
    if (new.target) {{
      return new NativeDTF(locales, nextOpts);
    }}
    return NativeDTF(locales, nextOpts);
  }};
  WrappedDTF.prototype = NativeDTF.prototype;
  WrappedDTF.supportedLocalesOf = NativeDTF.supportedLocalesOf.bind(NativeDTF);
  try {{
    Object.defineProperty(NativeIntl, 'DateTimeFormat', {{
      configurable: true,
      writable: true,
      value: WrappedDTF
    }});
  }} catch (_) {{}}

  const pad2 = (n) => (n < 10 ? '0' + n : String(n));
  const pad4 = (n) => {{
    const sign = n < 0 ? '-' : '';
    const abs = String(Math.abs(n)).padStart(4, '0');
    return sign + abs;
  }};
  const WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  const formatGmtSuffix = (ms) => {{
    const offsetMin = getOffsetMinutes(ms);
    const west = offsetMin;
    const total = Math.abs(west);
    const hh = pad2(Math.floor(total / 60));
    const mm = pad2(total % 60);
    const sign = west <= 0 ? '+' : '-';
    let tzLabel = TZ;
    if (longNameFmt) {{
      try {{
        const ps = nativeFormatToParts.call(longNameFmt, new NativeDate(ms));
        const tn = ps.find((x) => x.type === 'timeZoneName');
        if (tn && tn.value) tzLabel = tn.value;
      }} catch (_) {{}}
    }}
    return 'GMT' + sign + hh + mm + ' (' + tzLabel + ')';
  }};

  const dp = NativeDate.prototype;
  dp.getTimezoneOffset = function () {{
    const ms = nativeValueOf.call(this);
    if (!Number.isFinite(ms)) return NaN;
    return getOffsetMinutes(ms);
  }};
  dp.getFullYear = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCFullYear.call(s) : NaN; }};
  dp.getMonth = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCMonth.call(s) : NaN; }};
  dp.getDate = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCDate.call(s) : NaN; }};
  dp.getDay = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCDay.call(s) : NaN; }};
  dp.getHours = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCHours.call(s) : NaN; }};
  dp.getMinutes = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCMinutes.call(s) : NaN; }};
  dp.getSeconds = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCSeconds.call(s) : NaN; }};
  dp.getMilliseconds = function () {{ const s = shiftedUtcDate(this); return s ? nativeGetUTCMilliseconds.call(s) : NaN; }};

  const applyWallUpdate = (targetDate, y, m, d, h, mi, sec, msPart) => {{
    const guess = NativeDate.UTC(y, m, d, h, mi, sec, msPart);
    if (!Number.isFinite(guess)) return nativeSetTime.call(targetDate, NaN);
    const off1 = getOffsetMinutes(guess);
    const utc1 = guess + off1 * 60000;
    const off2 = getOffsetMinutes(utc1);
    return nativeSetTime.call(targetDate, guess + off2 * 60000);
  }};
  dp.setFullYear = function (y, m, d) {{
    const s = shiftedUtcDate(this);
    if (!s) return NaN;
    return applyWallUpdate(
      this,
      y,
      m === undefined ? nativeGetUTCMonth.call(s) : m,
      d === undefined ? nativeGetUTCDate.call(s) : d,
      nativeGetUTCHours.call(s),
      nativeGetUTCMinutes.call(s),
      nativeGetUTCSeconds.call(s),
      nativeGetUTCMilliseconds.call(s)
    );
  }};
  dp.setMonth = function (m, d) {{
    const s = shiftedUtcDate(this);
    if (!s) return NaN;
    return applyWallUpdate(
      this,
      nativeGetUTCFullYear.call(s),
      m,
      d === undefined ? nativeGetUTCDate.call(s) : d,
      nativeGetUTCHours.call(s),
      nativeGetUTCMinutes.call(s),
      nativeGetUTCSeconds.call(s),
      nativeGetUTCMilliseconds.call(s)
    );
  }};
  dp.setDate = function (d) {{
    const s = shiftedUtcDate(this);
    if (!s) return NaN;
    return applyWallUpdate(
      this,
      nativeGetUTCFullYear.call(s),
      nativeGetUTCMonth.call(s),
      d,
      nativeGetUTCHours.call(s),
      nativeGetUTCMinutes.call(s),
      nativeGetUTCSeconds.call(s),
      nativeGetUTCMilliseconds.call(s)
    );
  }};
  dp.setHours = function (h, mi, sec, msPart) {{
    const s = shiftedUtcDate(this);
    if (!s) return NaN;
    return applyWallUpdate(
      this,
      nativeGetUTCFullYear.call(s),
      nativeGetUTCMonth.call(s),
      nativeGetUTCDate.call(s),
      h,
      mi === undefined ? nativeGetUTCMinutes.call(s) : mi,
      sec === undefined ? nativeGetUTCSeconds.call(s) : sec,
      msPart === undefined ? nativeGetUTCMilliseconds.call(s) : msPart
    );
  }};
  dp.setMinutes = function (mi, sec, msPart) {{
    const s = shiftedUtcDate(this);
    if (!s) return NaN;
    return applyWallUpdate(
      this,
      nativeGetUTCFullYear.call(s),
      nativeGetUTCMonth.call(s),
      nativeGetUTCDate.call(s),
      nativeGetUTCHours.call(s),
      mi,
      sec === undefined ? nativeGetUTCSeconds.call(s) : sec,
      msPart === undefined ? nativeGetUTCMilliseconds.call(s) : msPart
    );
  }};

  dp.toDateString = function () {{
    const s = shiftedUtcDate(this);
    if (!s) return 'Invalid Date';
    return WEEKDAYS[nativeGetUTCDay.call(s)] + ' ' + MONTHS[nativeGetUTCMonth.call(s)] + ' ' + pad2(nativeGetUTCDate.call(s)) + ' ' + pad4(nativeGetUTCFullYear.call(s));
  }};
  dp.toTimeString = function () {{
    const ms = nativeValueOf.call(this);
    const s = shiftedUtcDate(this);
    if (!s || !Number.isFinite(ms)) return 'Invalid Date';
    return pad2(nativeGetUTCHours.call(s)) + ':' + pad2(nativeGetUTCMinutes.call(s)) + ':' + pad2(nativeGetUTCSeconds.call(s)) + ' ' + formatGmtSuffix(ms);
  }};
  dp.toString = function () {{
    const s = shiftedUtcDate(this);
    if (!s) return 'Invalid Date';
    return dp.toDateString.call(this) + ' ' + dp.toTimeString.call(this);
  }};
  dp.toLocaleString = function (locales, options) {{
    return nativeToLocaleString.call(this, locales, injectTzOpt(options));
  }};
  dp.toLocaleDateString = function (locales, options) {{
    return nativeToLocaleDateString.call(this, locales, injectTzOpt(options));
  }};
  dp.toLocaleTimeString = function (locales, options) {{
    return nativeToLocaleTimeString.call(this, locales, injectTzOpt(options));
  }};

  const _origResolved = nativeResolvedOptions;
  if (typeof _origResolved === 'function') {{
    NativeDTF.prototype.resolvedOptions = function () {{
      return _origResolved.call(this);
    }};
  }}
  void nativeFormat;
}})();"#
  )
}

pub fn parse_account_proxy(raw: &str) -> Result<Option<url::Url>, String> {
  let raw = raw.trim();
  if raw.is_empty() {
    return Ok(None);
  }
  if !SUPPORTS_ACCOUNT_PROXY {
    return Err("当前平台不支持账号代理".to_string());
  }
  let invalid = || "请输入无认证的 HTTP CONNECT 代理地址（http://主机:端口）".to_string();
  if !raw.get(..7).is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    || raw.bytes().any(|byte| byte.is_ascii_whitespace())
  {
    return Err(invalid());
  }
  let rest = &raw[7..];
  let authority = rest.strip_suffix('/').unwrap_or(rest);
  let explicit_port = if let Some(after_bracket) = authority.strip_prefix('[') {
    after_bracket.split_once("]:").map(|(_, port)| port)
  } else {
    authority.split_once(':').map(|(_, port)| port)
  };
  let has_explicit_port = explicit_port
    .is_some_and(|port| !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()));
  if !has_explicit_port {
    return Err(invalid());
  }
  let parsed = url::Url::parse(raw).map_err(|_| invalid())?;
  if parsed.scheme() != "http"
    || parsed.host().is_none()
    || !parsed.port_or_known_default().is_some_and(|port| port != 0)
    || !parsed.username().is_empty()
    || parsed.password().is_some()
    || parsed.path() != "/"
    || parsed.query().is_some()
    || parsed.fragment().is_some()
  {
    return Err(invalid());
  }
  if let Some(url::Host::Domain(host)) = parsed.host() {
    if !host
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
    {
      return Err(invalid());
    }
  }
  Ok(Some(parsed))
}

#[allow(dead_code)]
pub fn format_account_proxy(proxy: &url::Url) -> String {
  let host = proxy.host_str().expect("validated HTTP proxy host");
  let port = proxy.port_or_known_default().expect("validated HTTP proxy port");
  format!("http://{host}:{port}")
}

#[cfg(windows)]
pub fn windows_proxy_browser_args(proxy: &url::Url) -> String {
  format!(
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
     --autoplay-policy=no-user-gesture-required --proxy-server={}",
    format_account_proxy(proxy)
  )
}

#[cfg(windows)]
pub fn with_windows_account_proxy(
  builder: tauri::webview::WebviewBuilder<tauri::Wry>,
  proxy: Option<&url::Url>,
  app_id: &str,
) -> Result<tauri::webview::WebviewBuilder<tauri::Wry>, String> {
  let Some(proxy) = proxy else {
    return Ok(builder);
  };
  check_windows_proxy_overrides(app_id)?;
  Ok(if proxy.port().is_some() {
    builder.proxy_url(proxy.clone())
  } else {
    builder.additional_browser_args(&windows_proxy_browser_args(proxy))
  })
}

#[cfg(windows)]
fn windows_browser_switch(arg: &str) -> Option<(&str, &str)> {
  let arg = arg.trim();
  let switch = arg.strip_prefix("--").or_else(|| arg.strip_prefix(['-', '/']))?;
  Some(switch.split_once('=').unwrap_or((switch, "")))
}

#[cfg(windows)]
fn check_windows_proxy_arguments(args: &str, source: &str) -> Result<(), String> {
  use windows::core::HSTRING;
  use windows::Win32::Foundation::{LocalFree, HLOCAL};
  use windows::Win32::UI::Shell::CommandLineToArgvW;

  if args.trim().is_empty() {
    return Ok(());
  }
  let command = HSTRING::from(format!("webview2 {args}"));
  let mut count = 0;
  let argv = unsafe { CommandLineToArgvW(&command, &mut count) };
  if argv.is_null() {
    return Err(format!("无法核对 Windows WebView2 启动参数（{source}），已阻止打开账号"));
  }
  let conflicts = unsafe { std::slice::from_raw_parts(argv, count as usize) }
    .iter()
    .skip(1)
    .any(|arg| {
      let arg = String::from_utf16_lossy(unsafe { arg.as_wide() });
      windows_browser_switch(&arg).is_some_and(|(name, _)| {
        let name = name.to_ascii_lowercase();
        name.starts_with("proxy-")
          || matches!(
            name.as_str(),
            "no-proxy-server" | "winhttp-proxy-resolver" | "edge-webview-switches"
          )
      })
    });
  unsafe { LocalFree(Some(HLOCAL(argv.cast()))) };
  if conflicts {
    return Err(format!(
      "Windows WebView2 启动参数（{source}）可能覆盖账号代理，已阻止打开账号"
    ));
  }
  Ok(())
}

#[cfg(windows)]
fn check_windows_proxy_overrides(app_id: &str) -> Result<(), String> {
  use windows::core::HSTRING;
  use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};
  use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ,
  };
  use windows::Win32::UI::Shell::GetCurrentProcessExplicitAppUserModelID;

  const ENV_ARGS: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";
  const POLICY_KEY: &str = r"SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments";

  if let Some(args) = std::env::var_os(ENV_ARGS) {
    check_windows_proxy_arguments(&args.to_string_lossy(), ENV_ARGS)?;
  }
  for arg in std::env::args_os().skip(1) {
    if let Some((name, value)) = windows_browser_switch(&arg.to_string_lossy()) {
      if name.eq_ignore_ascii_case("edge-webview-switches") {
        check_windows_proxy_arguments(value, "--edge-webview-switches")?;
      }
    }
  }

  let exe = std::env::current_exe().map_err(|err| format!("无法核对 Windows 应用名称: {err}"))?;
  let exe_name = exe.file_name().ok_or_else(|| "无法核对 Windows 应用名称".to_string())?;
  let explicit_id = unsafe { GetCurrentProcessExplicitAppUserModelID() }
    .ok()
    .map(|id| HSTRING::from(webview2_com::take_pwstr(id)));
  let names = [HSTRING::from(app_id), HSTRING::from(exe_name), HSTRING::from("*")];
  let policy_key = HSTRING::from(POLICY_KEY);
  let mut buffer = vec![0u16; 32_768];
  for (root, hive) in [(HKEY_LOCAL_MACHINE, "HKLM"), (HKEY_CURRENT_USER, "HKCU")] {
    for name in explicit_id.iter().chain(names.iter()).filter(|name| !name.is_empty()) {
      let mut size = (buffer.len() * size_of::<u16>()) as u32;
      let result = unsafe {
        RegGetValueW(
          root,
          &policy_key,
          name,
          RRF_RT_REG_SZ,
          None,
          Some(buffer.as_mut_ptr().cast()),
          Some(&mut size),
        )
      };
      if matches!(result, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) {
        continue;
      }
      let source = format!("{hive}\\{POLICY_KEY}\\{name}");
      result
        .ok()
        .map_err(|err| format!("无法核对 Windows WebView2 启动参数（{source}）: {err}"))?;
      let args = String::from_utf16_lossy(&buffer[..size as usize / size_of::<u16>()]);
      check_windows_proxy_arguments(args.trim_end_matches('\0'), &source)?;
    }
  }
  Ok(())
}

#[cfg(windows)]
pub fn windows_webview_options_in_use(error: &tauri::Error) -> bool {
  use windows::Win32::Foundation::ERROR_INVALID_STATE;

  let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(inner)) = error else {
    return false;
  };
  matches!(
    inner.downcast_ref::<wry::Error>(),
    Some(wry::Error::WebView2Error(webview2_com::Error::WindowsError(inner)))
      if inner.code() == ERROR_INVALID_STATE.to_hresult()
  )
}

#[cfg(windows)]
pub async fn on_windows_navigation_completed(
  webview: &tauri::Webview,
  on_completed: impl Fn(Result<url::Url, ()>) + Send + 'static,
) -> Result<(), String> {
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2NavigationCompletedEventArgs2, COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED,
    COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN,
    COREWEBVIEW2_WEB_ERROR_STATUS_VALID_PROXY_AUTHENTICATION_REQUIRED,
  };
  use webview2_com::{take_pwstr, NavigationCompletedEventHandler};
  use windows::core::{Interface, BOOL, PWSTR};

  let (sender, receiver) = tokio::sync::oneshot::channel();
  webview
    .with_webview(move |platform_view| unsafe {
      let result = platform_view.controller().CoreWebView2().and_then(|core| {
        let handler = NavigationCompletedEventHandler::create(Box::new(move |view, args| {
          let (Some(view), Some(args)) = (view, args) else {
            return Ok(());
          };
          let mut success = BOOL::default();
          args.IsSuccess(&mut success)?;
          let mut status = Default::default();
          args.WebErrorStatus(&mut status)?;
          let mut http_status = 0;
          if !success.as_bool() {
            let _ = args
              .cast::<ICoreWebView2NavigationCompletedEventArgs2>()
              .and_then(|args| args.HttpStatusCode(&mut http_status));
          }
          if http_status == 407
            || status == COREWEBVIEW2_WEB_ERROR_STATUS_VALID_PROXY_AUTHENTICATION_REQUIRED
          {
            on_completed(Err(()));
          } else if success.as_bool() || http_status != 0 {
            let mut source = PWSTR::null();
            view.Source(&mut source)?;
            if let Ok(url) = url::Url::parse(&take_pwstr(source)) {
              if url.as_str() != "about:blank" {
                on_completed(Ok(url));
              }
            }
          } else if !matches!(
            status,
            COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN | COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED
          ) {
            on_completed(Err(()));
          }
          Ok(())
        }));
        core.add_NavigationCompleted(&handler, &mut 0)
      });
      let _ = sender.send(result);
    })
    .map_err(|err| format!("访问 Windows WebView2 失败: {err}"))?;
  receiver
    .await
    .map_err(|_| "注册 Windows WebView2 导航事件中断".to_string())?
    .map_err(|err| format!("注册 Windows WebView2 导航事件失败: {err}"))
}

#[cfg(target_os = "macos")]
unsafe fn apply_macos_proxy_to_wkwebview(
  view: &objc2_web_kit::WKWebView,
  proxy_url: Option<&url::Url>,
) -> Result<(), String> {
  use objc2_foundation::NSArray;

  let store = view.configuration().websiteDataStore();
  let configurations = proxy_url
    .map(create_http_connect_proxy)
    .transpose()?
    .map(|proxy| NSArray::arrayWithObject(&*proxy));
  let _: () = objc2::msg_send![&*store, setProxyConfigurations: configurations.as_deref()];
  Ok(())
}

#[cfg(target_os = "macos")]
pub async fn configure_desktop_proxy(
  webview: &tauri::Webview,
  proxy_url: Option<url::Url>,
) -> Result<(), String> {
  use objc2_web_kit::WKWebView;

  let (sender, receiver) = tokio::sync::oneshot::channel();
  webview
    .with_webview(move |platform_view| unsafe {
      let view: &WKWebView = &*platform_view.inner().cast();
      let result = apply_macos_proxy_to_wkwebview(view, proxy_url.as_ref());
      let _ = sender.send(result);
    })
    .map_err(|err| format!("访问 macOS WebView 失败: {err}"))?;
  receiver
    .await
    .map_err(|_| "macOS 账号代理配置中断".to_string())?
}

#[cfg(target_os = "macos")]
fn should_fail_macos_navigation(domain: &str, code: isize) -> bool {
  !((domain == "NSURLErrorDomain" && code == -999)
    || (domain == "WebKitErrorDomain" && code == 102))
}

#[cfg(target_os = "macos")]
mod macos_nav_delegate {
  use super::should_fail_macos_navigation;
  use objc2::rc::{Retained, Weak};
  use objc2::runtime::{AnyObject, Bool, NSObject, NSObjectProtocol, Sel};
  use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
  use objc2_foundation::NSError;
  use objc2_web_kit::WKWebView;

  struct MaamNavigationDelegateIvars {
    inner: Option<Weak<AnyObject>>,
    on_fail: Box<dyn Fn()>,
  }

  define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MaamNavigationDelegateIvars]
    struct MaamNavigationDelegateProxy;

    unsafe impl NSObjectProtocol for MaamNavigationDelegateProxy {}

    impl MaamNavigationDelegateProxy {
      #[unsafe(method(respondsToSelector:))]
      fn responds_to_selector(&self, sel: Sel) -> Bool {
        let base: Bool = unsafe { msg_send![super(self), respondsToSelector: sel] };
        if base.as_bool() {
          return Bool::YES;
        }
        if let Some(inner) = self.ivars().inner.as_ref().and_then(Weak::load) {
          let inner_responds: Bool = unsafe { msg_send![&*inner, respondsToSelector: sel] };
          return inner_responds;
        }
        Bool::NO
      }

      #[unsafe(method(forwardingTargetForSelector:))]
      fn forwarding_target_for_selector(&self, sel: Sel) -> *mut AnyObject {
        if let Some(inner) = self.ivars().inner.as_ref().and_then(Weak::load) {
          let inner_responds: Bool = unsafe { msg_send![&*inner, respondsToSelector: sel] };
          if inner_responds.as_bool() {
            return Retained::autorelease_return(inner);
          }
        }
        std::ptr::null_mut()
      }

      #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
      fn did_fail_provisional_navigation(
        &self,
        webview: *mut AnyObject,
        navigation: *mut AnyObject,
        error: *mut NSError,
      ) {
        if !error.is_null() {
          let err = unsafe { &*error };
          if should_fail_macos_navigation(&err.domain().to_string(), err.code()) {
            (self.ivars().on_fail)();
          }
        }
        if let Some(inner) = self.ivars().inner.as_ref().and_then(Weak::load) {
          let sel = objc2::sel!(webView:didFailProvisionalNavigation:withError:);
          let inner_responds: Bool = unsafe { msg_send![&*inner, respondsToSelector: sel] };
          if inner_responds.as_bool() {
            let _: () = unsafe {
              msg_send![&*inner, webView: webview, didFailProvisionalNavigation: navigation, withError: error]
            };
          }
        }
      }

      #[unsafe(method(webView:didFailNavigation:withError:))]
      fn did_fail_navigation(
        &self,
        webview: *mut AnyObject,
        navigation: *mut AnyObject,
        error: *mut NSError,
      ) {
        if !error.is_null() {
          let err = unsafe { &*error };
          if should_fail_macos_navigation(&err.domain().to_string(), err.code()) {
            (self.ivars().on_fail)();
          }
        }
        if let Some(inner) = self.ivars().inner.as_ref().and_then(Weak::load) {
          let sel = objc2::sel!(webView:didFailNavigation:withError:);
          let inner_responds: Bool = unsafe { msg_send![&*inner, respondsToSelector: sel] };
          if inner_responds.as_bool() {
            let _: () = unsafe {
              msg_send![&*inner, webView: webview, didFailNavigation: navigation, withError: error]
            };
          }
        }
      }

      #[unsafe(method(webView:didReceiveAuthenticationChallenge:completionHandler:))]
      fn did_receive_authentication_challenge(
        &self,
        _webview: *mut AnyObject,
        challenge: *mut AnyObject,
        completion_handler: &block2::Block<dyn Fn(isize, *mut AnyObject)>,
      ) {
        let is_proxy = if !challenge.is_null() {
          let space: Option<Retained<AnyObject>> =
            unsafe { msg_send![challenge, protectionSpace] };
          if let Some(space) = space {
            let proxy_flag: Bool = unsafe { msg_send![&*space, isProxy] };
            proxy_flag.as_bool()
          } else {
            false
          }
        } else {
          false
        };
        if is_proxy {
          completion_handler.call((2, std::ptr::null_mut()));
          (self.ivars().on_fail)();
        } else {
          completion_handler.call((1, std::ptr::null_mut()));
        }
      }
    }
  );

  static PROXY_DELEGATE_ASSOC_KEY: u8 = 0;

  pub(super) unsafe fn attach_macos_nav_failed_delegate(
    view: &WKWebView,
    on_failed: impl Fn() + Send + 'static,
  ) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| "当前不在 macOS 主线程".to_string())?;
    let prev: Option<Retained<AnyObject>> = msg_send![view, navigationDelegate];
    let inner = prev.as_deref().map(Weak::new);
    let allocated = mtm.alloc::<MaamNavigationDelegateProxy>();
    let set = allocated.set_ivars(MaamNavigationDelegateIvars {
      inner,
      on_fail: Box::new(on_failed),
    });
    let proxy: Retained<MaamNavigationDelegateProxy> = msg_send![super(set), init];
    objc2::ffi::objc_setAssociatedObject(
      (view as *const WKWebView) as *mut AnyObject,
      (&raw const PROXY_DELEGATE_ASSOC_KEY).cast::<std::ffi::c_void>(),
      Retained::as_ptr(&proxy) as *mut AnyObject,
      objc2::ffi::OBJC_ASSOCIATION_RETAIN_NONATOMIC,
    );
    let _: () = msg_send![view, setNavigationDelegate: &*proxy];
    Ok(())
  }
}

#[cfg(target_os = "macos")]
pub fn raise_macos_webview_to_front(webview: &tauri::Webview) {
  let _ = webview.with_webview(|platform_wv| unsafe {
    let ns_view: &objc2_app_kit::NSView = &*platform_wv.inner().cast();
    if let Some(superview) = ns_view.superview() {
      superview.addSubview(ns_view);
    }
  });
}

#[cfg(target_os = "macos")]
pub async fn is_macos_webview_loading(webview: &tauri::Webview) -> bool {
  let (tx, rx) = tokio::sync::oneshot::channel();
  if webview
    .with_webview(move |platform_wv| unsafe {
      let wk: &objc2_web_kit::WKWebView = &*platform_wv.inner().cast();
      let _ = tx.send(wk.isLoading());
    })
    .is_err()
  {
    return false;
  }
  match tokio::time::timeout(std::time::Duration::from_millis(300), rx).await {
    Ok(Ok(loading)) => loading,
    _ => false,
  }
}

#[cfg(target_os = "macos")]
pub async fn on_macos_navigation_failed(
  webview: &tauri::Webview,
  on_failed: impl Fn() + Send + 'static,
) -> Result<(), String> {
  use objc2_web_kit::WKWebView;

  let (sender, receiver) = tokio::sync::oneshot::channel();
  webview
    .with_webview(move |platform_view| unsafe {
      let view: &WKWebView = &*platform_view.inner().cast();
      let result = macos_nav_delegate::attach_macos_nav_failed_delegate(view, on_failed);
      let _ = sender.send(result);
    })
    .map_err(|err| format!("访问 macOS WebView 失败: {err}"))?;
  receiver
    .await
    .map_err(|_| "注册 macOS WebView 导航失败回调中断".to_string())?
}

#[cfg(target_os = "macos")]
fn create_http_connect_proxy(
  url: &url::Url,
) -> Result<objc2::rc::Retained<objc2_foundation::NSObject>, String> {
  use objc2::rc::Retained;
  use objc2_foundation::NSObject;
  use std::ffi::{c_char, CString};

  #[link(name = "Network", kind = "framework")]
  unsafe extern "C" {
    fn nw_endpoint_create_host(host: *const c_char, port: *const c_char) -> *mut NSObject;
    fn nw_proxy_config_create_http_connect(
      endpoint: *const NSObject,
      tls_options: *const NSObject,
    ) -> *mut NSObject;
    fn nw_proxy_config_set_failover_allowed(proxy: *const NSObject, allowed: bool);
  }

  let host = url.host_str().ok_or_else(|| "缺少代理主机".to_string())?;
  let port = url
    .port_or_known_default()
    .ok_or_else(|| "缺少代理端口".to_string())?;
  let host = CString::new(host.trim_matches(['[', ']'])).map_err(|_| "代理主机无效".to_string())?;
  let port = CString::new(port.to_string()).map_err(|_| "代理端口无效".to_string())?;
  unsafe {
    let endpoint = Retained::from_raw(nw_endpoint_create_host(host.as_ptr(), port.as_ptr()))
      .ok_or_else(|| "无法创建代理端点".to_string())?;
    let proxy = Retained::from_raw(nw_proxy_config_create_http_connect(&*endpoint, std::ptr::null()))
      .ok_or_else(|| "无法创建 HTTP CONNECT 代理配置".to_string())?;
    nw_proxy_config_set_failover_allowed(&*proxy, false);
    Ok(proxy)
  }
}

#[cfg(desktop)]
pub fn verify_desktop_profile_binding(
  webview: &tauri::Webview,
  expected_uuid: Uuid,
  expected_dir: &Path,
) -> impl std::future::Future<Output = Result<(), String>> + Send {
  #[cfg(target_os = "macos")]
  {
    let _ = expected_dir;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let res = webview
      .with_webview(move |platform_wv| unsafe {
        let wk: &objc2_web_kit::WKWebView = &*platform_wv.inner().cast();
        let store = wk.configuration().websiteDataStore();
        let check = match store.identifier() {
          Some(ns_uuid) => {
            let actual = ns_uuid.UUIDString().to_string();
            let expected = expected_uuid.hyphenated().to_string();
            if actual.eq_ignore_ascii_case(&expected) {
              Ok(())
            } else {
              Err(format!(
                "WKWebsiteDataStore 标识不匹配（期望 {expected}，实际 {actual}）"
              ))
            }
          }
          None => Err("WKWebsiteDataStore 未绑定独立 UUID（落入默认资料），已阻止加载".to_string()),
        };
        let _ = tx.send(check);
      });

    async move {
      res.map_err(|e| format!("访问 macOS WebView 失败: {e}"))?;
      rx
        .await
        .map_err(|_| "核对 macOS WKWebsiteDataStore 超时或中断".to_string())?
    }
  }

  #[cfg(windows)]
  {
    let _ = expected_uuid;
    let expected_owned = expected_dir.to_path_buf();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let res = webview
      .with_webview(move |platform_wv| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
          ICoreWebView2Environment7, ICoreWebView2_2,
        };
        use windows::core::Interface;

        let check = (|| -> Result<(), String> {
          let core = platform_wv
            .controller()
            .CoreWebView2()
            .map_err(|e| format!("获取 CoreWebView2 失败: {e}"))?;
          let core2: ICoreWebView2_2 = core
            .cast()
            .map_err(|e| format!("转换 ICoreWebView2_2 失败: {e}"))?;
          let env = core2
            .Environment()
            .map_err(|e| format!("获取 WebView2 Environment 失败: {e}"))?;
          let env7: ICoreWebView2Environment7 = env
            .cast()
            .map_err(|e| format!("转换 ICoreWebView2Environment7 失败: {e}"))?;
          let mut raw_folder = windows::core::PWSTR::null();
          env7
            .UserDataFolder(&mut raw_folder)
            .map_err(|e| format!("读取 WebView2 UserDataFolder 失败: {e}"))?;
          let actual_str = raw_folder
            .to_string()
            .map_err(|e| format!("解析 UserDataFolder 失败: {e}"))?;
          windows::Win32::System::Com::CoTaskMemFree(Some(raw_folder.0 as *const _));

          let actual_norm = normalize_win_path(Path::new(&actual_str));
          let expected_norm = normalize_win_path(&expected_owned);
          if actual_norm == expected_norm {
            Ok(())
          } else {
            Err(format!(
              "Windows WebView2 实际数据目录被系统配置覆盖（实际: {actual_str}），已阻止打开以防串号"
            ))
          }
        })();
        let _ = tx.send(check);
      });

    async move {
      res.map_err(|e| format!("访问 Windows WebView2 失败: {e}"))?;
      rx
        .await
        .map_err(|_| "核对 Windows WebView2 数据目录中断".to_string())?
    }
  }

  #[cfg(all(not(target_os = "macos"), not(windows)))]
  {
    let _ = (webview, expected_uuid, expected_dir);
    async move { Ok(()) }
  }
}

#[cfg(windows)]
pub async fn configure_windows_timezone(
  webview: &tauri::Webview,
  timezone: Option<&str>,
) -> Result<(), String> {
  let Some(tz) = timezone.map(str::trim).filter(|s| !s.is_empty()) else {
    return Ok(());
  };
  let params_json = serde_json::json!({ "timezoneId": tz }).to_string();
  let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
  let tx_slot = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

  webview
    .with_webview(move |platform_wv| unsafe {
      use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
      use windows::core::{HSTRING, PCWSTR};

      let core = match platform_wv.controller().CoreWebView2() {
        Ok(core) => core,
        Err(e) => {
          if let Some(sender) = tx_slot.lock().unwrap().take() {
            let _ = sender.send(Err(format!("获取 CoreWebView2 失败: {e}")));
          }
          return;
        }
      };

      let method_w = HSTRING::from("Emulation.setTimezoneOverride");
      let params_w = HSTRING::from(params_json);
      let tx_cb = std::sync::Arc::clone(&tx_slot);
      let handler =
        CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |err_code, _res_json| {
          let outcome = if err_code.is_ok() {
            Ok(())
          } else {
            Err(format!("设置 WebView2 时区失败: {err_code:?}"))
          };
          if let Some(sender) = tx_cb.lock().unwrap().take() {
            let _ = sender.send(outcome);
          }
          Ok(())
        }));

      let method_ptr = PCWSTR(method_w.as_ptr());
      let params_ptr = PCWSTR(params_w.as_ptr());
      if let Err(e) = core.CallDevToolsProtocolMethod(method_ptr, params_ptr, &handler) {
        if let Some(sender) = tx_slot.lock().unwrap().take() {
          let _ = sender.send(Err(format!("调用 WebView2 时区接口失败: {e}")));
        }
      }
    })
    .map_err(|e| format!("访问 Windows WebView2 失败: {e}"))?;

  match tokio::time::timeout(std::time::Duration::from_secs(3), rx).await {
    Ok(Ok(res)) => res,
    Ok(Err(_)) => Err("Windows WebView2 时区设置回调中断".to_string()),
    Err(_) => Err("Windows WebView2 时区设置超时".to_string()),
  }
}

#[cfg(windows)]
fn normalize_win_path(path: &Path) -> String {
  let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
  resolved
    .to_string_lossy()
    .trim_start_matches(r"\\?\")
    .trim_end_matches(['\\', '/'])
    .to_ascii_lowercase()
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieMeta {
  pub name: String,
  pub http_only: bool,
  pub session_only: bool,
  pub expires_at_ms: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CookieExpirySummary {
  pub expires_at_ms: Option<u64>,
  pub session_only: bool,
}

#[allow(dead_code)]
pub fn unix_secs_f64_to_ms(secs: f64) -> Option<u64> {
  if !secs.is_finite() || secs <= 0.0 {
    return None;
  }
  let ms = secs * 1000.0;
  if ms >= (u64::MAX as f64) {
    Some(u64::MAX)
  } else {
    Some(ms as u64)
  }
}

#[allow(dead_code)]
pub fn unix_secs_i64_to_ms(secs: i64) -> Option<u64> {
  if secs <= 0 {
    return None;
  }
  Some((secs as u64).saturating_mul(1000))
}

#[allow(dead_code)]
fn is_noise_cookie_name(name: &str) -> bool {
  let lower = name.trim().to_ascii_lowercase();
  if lower.is_empty() {
    return true;
  }
  const EXACT: &[&str] = &[
    "_ga",
    "_gid",
    "_gat",
    "_fbp",
    "_fbc",
    "_ttclid",
    "_uetsid",
    "_uetvid",
    "_clck",
    "_clsk",
    "__cf_bm",
    "cf_clearance",
    "__cfduid",
    "_cfuvid",
    "__cflb",
    "ak_bmsc",
    "bm_sz",
    "bm_sv",
    "_abck",
    "aws-waf-token",
    "hubspotutk",
    "_rdt_uuid",
    "_pin_unauth",
    "muc_ads",
    "locale",
    "lang",
    "theme",
    "color_scheme",
    "tz",
    "timezone",
    "dpr",
    "wd",
    "viewport",
    "datr",
    "sb",
    "fr",
    "oai-did",
    "device_id",
    "did",
    "client_id",
    "guest_id",
    "anon_id",
  ];
  if EXACT.contains(&lower.as_str()) {
    return true;
  }
  const PREFIXES: &[&str] = &[
    "_ga_",
    "_gid_",
    "_gat_",
    "_gcl_",
    "_hj",
    "_pk_",
    "_ym_",
    "_dd_s",
    "ajs_",
    "amplitude_",
    "mp_",
    "ph_",
    "intercom-",
    "__hs",
    "__cf",
    "cf_",
    "incap_ses_",
    "visid_incap_",
  ];
  if PREFIXES.iter().any(|prefix| lower.starts_with(prefix)) {
    return true;
  }
  const SUBSTRINGS: &[&str] = &[
    "csrf",
    "xsrf",
    "analytics",
    "telemetry",
    "sentry",
    "datadog",
    "intercom",
  ];
  SUBSTRINGS.iter().any(|part| lower.contains(part))
}

#[allow(dead_code)]
fn is_auth_cookie_name(name: &str) -> bool {
  let lower = name.trim().to_ascii_lowercase();
  const KEYWORDS: &[&str] = &[
    "session",
    "sess",
    "token",
    "auth",
    "jwt",
    "refresh",
    "oauth",
    "oidc",
    "sso",
    "login",
    "credential",
    "bearer",
  ];
  if KEYWORDS.iter().any(|kw| lower.contains(kw)) {
    return true;
  }
  matches!(lower.as_str(), "sid" | "c_user" | "xs")
    || lower.ends_with(".sid")
    || lower.ends_with("_sid")
    || lower.ends_with("-sid")
    || lower.starts_with("__secure-")
    || lower.starts_with("__host-")
}

#[allow(dead_code)]
pub fn summarize_cookie_expiry(cookies: &[CookieMeta], now_ms: u64) -> CookieExpirySummary {
  let valid: Vec<(bool, bool, Option<u64>)> = cookies
    .iter()
    .filter_map(|cookie| {
      let expires_at_ms = if cookie.session_only {
        None
      } else {
        cookie.expires_at_ms
      };
      if expires_at_ms.is_some_and(|exp| exp <= now_ms) {
        return None;
      }
      if is_noise_cookie_name(&cookie.name) {
        return None;
      }
      Some((cookie.http_only, is_auth_cookie_name(&cookie.name), expires_at_ms))
    })
    .collect();

  if valid.is_empty() {
    return CookieExpirySummary::default();
  }

  let pick_tier = |predicate: &dyn Fn(bool, bool) -> bool| -> Option<CookieExpirySummary> {
    let mut matched = false;
    let mut max_exp: Option<u64> = None;
    for &(http_only, auth, expires_at_ms) in &valid {
      if predicate(http_only, auth) {
        matched = true;
        if let Some(exp) = expires_at_ms {
          max_exp = Some(max_exp.map_or(exp, |prev| prev.max(exp)));
        }
      }
    }
    if !matched {
      return None;
    }
    Some(CookieExpirySummary {
      expires_at_ms: max_exp,
      session_only: max_exp.is_none(),
    })
  };

  pick_tier(&|http_only, auth| http_only && auth)
    .or_else(|| pick_tier(&|_, auth| auth))
    .or_else(|| pick_tier(&|http_only, _| http_only))
    .or_else(|| pick_tier(&|_, _| true))
    .unwrap_or_default()
}

#[cfg(target_os = "macos")]
pub async fn read_macos_cookies_for_url(
  app: &tauri::AppHandle,
  uuid: Uuid,
  url: url::Url,
) -> Result<(String, CookieExpirySummary), String> {
  use objc2::{AnyThread, MainThreadMarker};
  use objc2_foundation::{NSArray, NSHTTPCookie, NSString, NSUUID};
  use objc2_web_kit::WKWebsiteDataStore;
  use std::cell::RefCell;
  use std::ptr::NonNull;

  let host = url.host_str().ok_or_else(|| "服务域名无效".to_string())?.to_owned();
  let path = url.path().to_owned();
  let (sender, receiver) = tokio::sync::oneshot::channel();
  app
    .run_on_main_thread(move || {
      let identifier = NSString::from_str(&uuid.to_string());
      let Some(identifier) = NSUUID::initWithUUIDString(NSUUID::alloc(), &identifier) else {
        let _ = sender.send(Err("账号资料标识无效".to_string()));
        return;
      };
      let Some(main_thread) = MainThreadMarker::new() else {
        let _ = sender.send(Err("读取账号 Cookie 失败".to_string()));
        return;
      };
      let store = unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&identifier, main_thread) };
      let sender = RefCell::new(Some(sender));
      let callback = block2::RcBlock::new(move |cookies: NonNull<NSArray<NSHTTPCookie>>| {
        let now_ms = crate::storage::now_ms();
        let mut pairs = Vec::new();
        let mut metas = Vec::new();
        for cookie in unsafe { cookies.as_ref() }.to_vec() {
          let domain = cookie.domain().to_string().to_ascii_lowercase();
          let domain_matches = match domain.strip_prefix('.') {
            Some(domain) => host == domain
              || host.strip_suffix(domain).is_some_and(|prefix| prefix.ends_with('.')),
            None => host == domain,
          };
          let cookie_path = cookie.path().to_string();
          let path_matches = path == cookie_path
            || path.strip_prefix(&cookie_path).is_some_and(|suffix| {
              cookie_path.ends_with('/') || suffix.starts_with('/')
            });
          let expires_at_ms = cookie
            .expiresDate()
            .and_then(|date| unix_secs_f64_to_ms(date.timeIntervalSince1970()));
          let expired = expires_at_ms.is_some_and(|exp| exp <= now_ms)
            || cookie.expiresDate().is_some_and(|date| date.timeIntervalSinceNow() <= 0.0);
          if domain_matches && path_matches && !expired {
            let name = cookie.name().to_string();
            let value = cookie.value().to_string();
            metas.push(CookieMeta {
              name: name.clone(),
              http_only: cookie.isHTTPOnly(),
              session_only: cookie.isSessionOnly(),
              expires_at_ms,
            });
            pairs.push((cookie_path.len(), format!("{name}={value}")));
          }
        }
        pairs.sort_by_key(|(path_len, _)| std::cmp::Reverse(*path_len));
        let header = pairs.into_iter().map(|(_, pair)| pair).collect::<Vec<_>>().join("; ");
        let summary = summarize_cookie_expiry(&metas, now_ms);
        if let Some(sender) = sender.borrow_mut().take() {
          let _ = sender.send(Ok((header, summary)));
        }
      });
      unsafe { store.httpCookieStore().getAllCookies(&callback) };
    })
    .map_err(|_| "读取账号 Cookie 失败".to_string())?;
  receiver.await.map_err(|_| "读取账号 Cookie 中断".to_string())?
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub async fn read_macos_cookie_header(
  app: &tauri::AppHandle,
  uuid: Uuid,
  url: url::Url,
) -> Result<String, String> {
  read_macos_cookies_for_url(app, uuid, url)
    .await
    .map(|(header, _)| header)
}

#[cfg(target_os = "macos")]
pub async fn read_macos_cookie_specs_for_hosts(
  app: &tauri::AppHandle,
  uuid: Uuid,
  allowed_hosts: Vec<String>,
) -> Result<Vec<ImportedCookieSpec>, String> {
  use objc2::{AnyThread, MainThreadMarker};
  use objc2_foundation::{NSArray, NSHTTPCookie, NSString, NSUUID};
  use objc2_web_kit::WKWebsiteDataStore;
  use std::cell::RefCell;
  use std::ptr::NonNull;

  let (sender, receiver) = tokio::sync::oneshot::channel();
  app
    .run_on_main_thread(move || {
      let identifier = NSString::from_str(&uuid.to_string());
      let Some(identifier) = NSUUID::initWithUUIDString(NSUUID::alloc(), &identifier) else {
        let _ = sender.send(Err("账号资料标识无效".to_string()));
        return;
      };
      let Some(main_thread) = MainThreadMarker::new() else {
        let _ = sender.send(Err("读取账号 Cookie 失败".to_string()));
        return;
      };
      let store = unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&identifier, main_thread) };
      let sender = RefCell::new(Some(sender));
      let callback = block2::RcBlock::new(move |cookies: NonNull<NSArray<NSHTTPCookie>>| {
        let now_ms = crate::storage::now_ms();
        let mut out = Vec::new();
        for cookie in unsafe { cookies.as_ref() }.to_vec() {
          let raw_domain = cookie.domain().to_string();
          let domain_norm = raw_domain.trim().trim_start_matches('.').to_ascii_lowercase();
          if domain_norm.is_empty()
            || (!allowed_hosts.is_empty()
              && !allowed_hosts
                .iter()
                .any(|allowed| crate::providers::host_matches(&domain_norm, allowed)))
          {
            continue;
          }
          let expires_at_ms = cookie
            .expiresDate()
            .and_then(|date| unix_secs_f64_to_ms(date.timeIntervalSince1970()));
          let expired = expires_at_ms.is_some_and(|exp| exp <= now_ms)
            || cookie.expiresDate().is_some_and(|date| date.timeIntervalSinceNow() <= 0.0);
          if expired {
            continue;
          }
          let name = cookie.name().to_string();
          if name.is_empty() {
            continue;
          }
          let value = cookie.value().to_string();
          let path = cookie.path().to_string();
          let max_age_secs = expires_at_ms
            .map(|exp| ((exp.saturating_sub(now_ms)) / 1000).clamp(1, 365 * 86_400) as i64);
          out.push(ImportedCookieSpec {
            name,
            value,
            domain: if raw_domain.starts_with('.') {
              raw_domain.to_ascii_lowercase()
            } else {
              domain_norm
            },
            path: if path.is_empty() { "/".to_string() } else { path },
            secure: cookie.isSecure(),
            http_only: cookie.isHTTPOnly(),
            same_site: None,
            max_age_secs,
          });
        }
        if let Some(sender) = sender.borrow_mut().take() {
          let _ = sender.send(Ok(out));
        }
      });
      unsafe { store.httpCookieStore().getAllCookies(&callback) };
    })
    .map_err(|_| "读取账号 Cookie 失败".to_string())?;
  receiver.await.map_err(|_| "读取账号 Cookie 中断".to_string())?
}

#[cfg(any(target_os = "macos", windows))]
pub async fn write_cookie_clipboard(app: &tauri::AppHandle, header: String) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  use arboard::SetExtApple;
  #[cfg(windows)]
  use arboard::SetExtWindows;

  if header.is_empty() {
    return Err("该账号暂无可复制的 Cookie，请先登录".to_string());
  }
  let (sender, receiver) = tokio::sync::oneshot::channel();
  app
    .run_on_main_thread(move || {
      let result = arboard::Clipboard::new().and_then(|mut clipboard| {
        let text = clipboard.set().exclude_from_history();
        #[cfg(windows)]
        let text = text.exclude_from_cloud().exclude_from_monitoring();
        text.text(header)
      });
      let _ = sender.send(result.map_err(|_| "无法写入系统剪贴板，请重试".to_string()));
    })
    .map_err(|_| "无法写入系统剪贴板，请重试".to_string())?;
  receiver.await.map_err(|_| "写入系统剪贴板中断".to_string())?
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedCookieSpec {
  pub name: String,
  pub value: String,
  pub domain: String,
  pub path: String,
  pub secure: bool,
  pub http_only: bool,
  pub same_site: Option<String>,
  pub max_age_secs: Option<i64>,
}

fn normalize_same_site(raw: Option<&str>) -> Option<String> {
  let s = raw?.trim().to_ascii_lowercase();
  match s.as_str() {
    "lax" => Some("lax".to_string()),
    "strict" => Some("strict".to_string()),
    "none" | "no_restriction" => Some("none".to_string()),
    _ => None,
  }
}

fn push_validated_imported_cookie(
  out: &mut Vec<ImportedCookieSpec>,
  seen: &mut std::collections::HashMap<(String, String, String), usize>,
  default_host: &str,
  allowed_hosts: &[String],
  raw_name: &str,
  raw_value: &str,
  raw_domain: Option<&str>,
  raw_path: Option<&str>,
  mut secure: bool,
  http_only: bool,
  same_site: Option<String>,
  max_age_secs: Option<i64>,
) {
  let name = raw_name.trim();
  if name.is_empty()
    || !name
      .bytes()
      .all(|b| b.is_ascii_graphic() && !matches!(b, b';' | b'='))
  {
    return;
  }
  if raw_value.bytes().any(|b| matches!(b, b'\r' | b'\n' | 0)) {
    return;
  }
  let value = raw_value.to_string();

  let default_clean = default_host.trim().to_ascii_lowercase();
  let mut domain = raw_domain
    .map(|d| d.trim().to_ascii_lowercase())
    .filter(|d| !d.is_empty())
    .unwrap_or_else(|| default_clean.clone());
  let check_host = domain.trim_start_matches('.');
  if check_host.is_empty()
    || (!crate::providers::host_matches(check_host, &default_clean)
      && !allowed_hosts
        .iter()
        .any(|allowed| crate::providers::host_matches(check_host, allowed)))
  {
    return;
  }

  let raw_p = raw_path.map(|p| p.trim()).unwrap_or("/");
  let mut path = if raw_p.is_empty() {
    "/".to_string()
  } else if raw_p.starts_with('/') {
    raw_p.to_string()
  } else {
    format!("/{raw_p}")
  };

  if name.starts_with("__Host-") {
    domain = check_host.to_string();
    path = "/".to_string();
    secure = true;
  } else if name.starts_with("__Secure-") {
    secure = true;
  }

  let spec = ImportedCookieSpec {
    name: name.to_string(),
    value,
    domain: domain.clone(),
    path: path.clone(),
    secure,
    http_only,
    same_site,
    max_age_secs,
  };
  let key = (name.to_string(), domain, path);
  if let Some(&idx) = seen.get(&key) {
    out[idx] = spec;
  } else {
    let idx = out.len();
    seen.insert(key, idx);
    out.push(spec);
  }
}

pub fn parse_imported_cookies(
  raw: &str,
  default_host: &str,
  allowed_hosts: &[String],
  now_secs: i64,
) -> Result<Vec<ImportedCookieSpec>, String> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Err("请先粘贴要导入的 Cookie 内容".to_string());
  }

  let mut out = Vec::new();
  let mut seen = std::collections::HashMap::new();

  if trimmed.starts_with('[') || trimmed.starts_with('{') {
    let val: serde_json::Value = serde_json::from_str(trimmed)
      .map_err(|_| "JSON 格式 Cookie 无法解析，请检查是否复制完整".to_string())?;
    let arr = if let Some(list) = val.as_array() {
      list.as_slice()
    } else if let Some(list) = val.get("cookies").and_then(|v| v.as_array()) {
      list.as_slice()
    } else {
      return Err("JSON Cookie 必须是数组或包含 cookies 数组的对象".to_string());
    };
    for item in arr {
      let Some(obj) = item.as_object() else {
        continue;
      };
      let Some(name) = obj.get("name").and_then(|v| v.as_str()) else {
        continue;
      };
      let Some(value) = obj.get("value").and_then(|v| v.as_str()) else {
        continue;
      };
      let exp_num = obj
        .get("expirationDate")
        .or_else(|| obj.get("expires"))
        .and_then(|v| v.as_f64());
      let max_age_secs = if let Some(raw_exp) = exp_num.filter(|n| n.is_finite() && *n > 0.0) {
        let exp_secs = if raw_exp > 1_000_000_000_000.0 {
          (raw_exp / 1000.0) as i64
        } else {
          raw_exp as i64
        };
        if exp_secs <= now_secs {
          continue;
        }
        Some((exp_secs - now_secs).clamp(60, 365 * 86_400))
      } else {
        None
      };
      let domain = obj.get("domain").and_then(|v| v.as_str());
      let path = obj.get("path").and_then(|v| v.as_str());
      let secure = obj.get("secure").and_then(|v| v.as_bool()).unwrap_or(true);
      let http_only = obj
        .get("httpOnly")
        .or_else(|| obj.get("http_only"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      let same_site = normalize_same_site(
        obj
          .get("sameSite")
          .or_else(|| obj.get("same_site"))
          .and_then(|v| v.as_str()),
      );
      push_validated_imported_cookie(
        &mut out,
        &mut seen,
        default_host,
        allowed_hosts,
        name,
        value,
        domain,
        path,
        secure,
        http_only,
        same_site,
        max_age_secs,
      );
    }
  } else {
    let has_netscape_tabs = trimmed.lines().any(|line| {
      let l = line.trim();
      if l.is_empty() {
        return false;
      }
      let effective = l.strip_prefix("#HttpOnly_").unwrap_or(l);
      if effective.starts_with('#') {
        return false;
      }
      effective.matches('\t').count() >= 6
    });

    if has_netscape_tabs {
      for line in trimmed.lines() {
        let l = line.trim();
        if l.is_empty() {
          continue;
        }
        let (http_only, effective) = if let Some(rest) = l.strip_prefix("#HttpOnly_") {
          (true, rest)
        } else {
          if l.starts_with('#') {
            continue;
          }
          (false, l)
        };
        let parts: Vec<&str> = effective.splitn(7, '\t').collect();
        if parts.len() != 7 {
          continue;
        }
        let domain = parts[0].trim();
        let path = parts[2].trim();
        let secure = parts[3].trim().eq_ignore_ascii_case("true") || parts[3].trim() == "1";
        let expiry = parts[4].trim().parse::<i64>().unwrap_or(0);
        if expiry > 0 && expiry <= now_secs {
          continue;
        }
        let max_age_secs = if expiry > now_secs {
          Some((expiry - now_secs).clamp(60, 365 * 86_400))
        } else {
          None
        };
        let name = parts[5];
        let value = parts[6];
        push_validated_imported_cookie(
          &mut out,
          &mut seen,
          default_host,
          allowed_hosts,
          name,
          value,
          Some(domain),
          Some(path),
          secure,
          http_only,
          None,
          max_age_secs,
        );
      }
    } else {
      for segment in trimmed.split([';', '\r', '\n']) {
        let mut s = segment.trim();
        if s.is_empty() {
          continue;
        }
        if s.len() >= 7 && s[..7].eq_ignore_ascii_case("cookie:") {
          s = s[7..].trim();
        }
        let Some((raw_name, raw_val)) = s.split_once('=') else {
          continue;
        };
        push_validated_imported_cookie(
          &mut out,
          &mut seen,
          default_host,
          allowed_hosts,
          raw_name,
          raw_val.trim(),
          None,
          Some("/"),
          true,
          false,
          Some("lax".to_string()),
          Some(30 * 86_400),
        );
      }
    }
  }

  if out.is_empty() {
    return Err("未解析到适用于当前服务的有效 Cookie，请检查粘贴格式或域名".to_string());
  }
  Ok(out)
}

fn is_valid_identity_email_str(s: &str) -> bool {
  let Some((local, domain)) = s.split_once('@') else {
    return false;
  };
  if local.is_empty() || domain.len() < 4 || !domain.contains('.') {
    return false;
  }
  if !local
    .bytes()
    .all(|b| b.is_ascii_alphanumeric() || b"._%+-".contains(&b))
  {
    return false;
  }
  let mut parts = domain.split('.');
  let Some(last) = parts.clone().last() else {
    return false;
  };
  if last.len() < 2 || !last.bytes().all(|b| b.is_ascii_alphabetic()) {
    return false;
  }
  parts.all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}

fn is_bad_identity_email(email: &str) -> bool {
  let lower = email.to_ascii_lowercase();
  let prefix_bad = [
    "support@",
    "privacy@",
    "help@",
    "legal@",
    "press@",
    "noreply@",
    "no-reply@",
    "info@",
    "security@",
    "contact@",
    "abuse@",
    "feedback@",
    "sales@",
    "terms@",
    "hello@",
    "team@",
    "admin@",
    "postmaster@",
    "notifications@",
  ];
  if prefix_bad.iter().any(|p| lower.starts_with(p)) {
    return true;
  }
  lower.ends_with("@example.com")
    || lower.ends_with("@sentry.io")
    || lower.ends_with("@wixpress.com")
}

fn is_provider_identity_email(email: &str) -> bool {
  let lower = email.to_ascii_lowercase();
  lower.ends_with("@muse.ai")
    || lower.ends_with("@mail.muse.ai")
    || lower.ends_with("@hark.com")
    || lower.ends_with("@meta.com")
    || lower.ends_with("@facebook.com")
}

fn push_email_candidates_from_text(text: &str, out: &mut Vec<String>) {
  let bytes = text.as_bytes();
  for (idx, &b) in bytes.iter().enumerate() {
    if b != b'@' {
      continue;
    }
    let mut start = idx;
    while start > 0 {
      let c = bytes[start - 1];
      if c.is_ascii_alphanumeric() || b"._%+-".contains(&c) {
        start -= 1;
      } else {
        break;
      }
    }
    let mut end = idx + 1;
    while end < bytes.len() {
      let c = bytes[end];
      if c.is_ascii_alphanumeric() || b".-".contains(&c) {
        end += 1;
      } else {
        break;
      }
    }
    let candidate = text[start..end].trim_matches('.').to_ascii_lowercase();
    if is_valid_identity_email_str(&candidate) && !is_bad_identity_email(&candidate) {
      out.push(candidate);
    }
  }
}

fn percent_decode_lossy(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'%' && i + 2 < bytes.len() {
      let h1 = (bytes[i + 1] as char).to_digit(16);
      let h2 = (bytes[i + 2] as char).to_digit(16);
      if let (Some(a), Some(b)) = (h1, h2) {
        out.push(((a << 4) | b) as u8);
        i += 3;
        continue;
      }
    }
    out.push(bytes[i]);
    i += 1;
  }
  String::from_utf8_lossy(&out).into_owned()
}

fn decode_base64_url_loose(input: &str) -> Option<Vec<u8>> {
  let clean = input.trim().trim_end_matches('=');
  if clean.len() < 4 {
    return None;
  }
  let mut out = Vec::with_capacity((clean.len() * 3) / 4);
  let mut buf: u32 = 0;
  let mut bits: u8 = 0;
  for b in clean.bytes() {
    let val = match b {
      b'A'..=b'Z' => b - b'A',
      b'a'..=b'z' => b - b'a' + 26,
      b'0'..=b'9' => b - b'0' + 52,
      b'-' | b'+' => 62,
      b'_' | b'/' => 63,
      _ => return None,
    };
    buf = (buf << 6) | (val as u32);
    bits += 6;
    if bits >= 8 {
      bits -= 8;
      out.push(((buf >> bits) & 0xff) as u8);
    }
  }
  Some(out)
}

fn extract_emails_from_json_value(val: &serde_json::Value, depth: u8, out: &mut Vec<String>) {
  if depth > 6 {
    return;
  }
  const EMAIL_KEYS: &[&str] = &[
    "email",
    "user_email",
    "userEmail",
    "contact_email",
    "contactEmail",
    "primary_email",
    "primaryEmail",
    "email_address",
    "emailAddress",
    "billing_email",
    "billingEmail",
    "customer_email",
    "customerEmail",
    "login_email",
    "loginEmail",
    "account_email",
    "accountEmail",
    "verified_email",
    "verifiedEmail",
    "display_label",
    "displayLabel",
    "contact_point",
    "contactPoint",
    "sub",
    "preferred_username",
    "upn",
  ];
  match val {
    serde_json::Value::Object(map) => {
      for k in EMAIL_KEYS {
        if let Some(s) = map.get(*k).and_then(|v| v.as_str()) {
          push_email_candidates_from_text(s, out);
        }
      }
      for v in map.values() {
        if v.is_object() || v.is_array() {
          extract_emails_from_json_value(v, depth + 1, out);
        }
      }
    }
    serde_json::Value::Array(arr) => {
      for item in arr.iter().take(20) {
        extract_emails_from_json_value(item, depth + 1, out);
      }
    }
    _ => {}
  }
}

pub fn extract_identity_hint_from_cookies(specs: &[ImportedCookieSpec]) -> Option<String> {
  let mut candidates = Vec::new();
  for spec in specs {
    let decoded = percent_decode_lossy(&spec.value);
    push_email_candidates_from_text(&spec.value, &mut candidates);
    if decoded != spec.value {
      push_email_candidates_from_text(&decoded, &mut candidates);
    }

    let mut b64_segments: Vec<&str> = Vec::new();
    let trimmed = decoded.trim();
    if trimmed.matches('.').count() >= 2 {
      for seg in trimmed.split('.').skip(1).take(2) {
        b64_segments.push(seg);
      }
    } else if !trimmed.contains('.') {
      b64_segments.push(trimmed);
    }

    for seg in b64_segments {
      if let Some(bytes) = decode_base64_url_loose(seg) {
        if let Ok(text) = std::str::from_utf8(&bytes) {
          if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            extract_emails_from_json_value(&json, 0, &mut candidates);
          } else if text.contains('@') {
            push_email_candidates_from_text(text, &mut candidates);
          }
        }
      }
    }
  }

  candidates
    .iter()
    .find(|e| !is_provider_identity_email(e))
    .cloned()
    .or_else(|| candidates.into_iter().next())
}

#[cfg(desktop)]
pub fn imported_cookie_to_tauri(
  spec: &ImportedCookieSpec,
) -> Result<tauri::webview::Cookie<'static>, String> {
  let mut template = format!("k=v; Domain={}; Path={}", spec.domain, spec.path);
  if let Some(max_age) = spec.max_age_secs {
    template.push_str(&format!("; Max-Age={max_age}"));
  }
  if spec.secure {
    template.push_str("; Secure");
  }
  if spec.http_only {
    template.push_str("; HttpOnly");
  }
  if let Some(ss) = spec.same_site.as_deref() {
    match ss {
      "lax" => template.push_str("; SameSite=Lax"),
      "strict" => template.push_str("; SameSite=Strict"),
      "none" => template.push_str("; SameSite=None"),
      _ => {}
    }
  }
  let mut cookie =
    tauri::webview::Cookie::parse(template).map_err(|_| "Cookie 属性无效".to_string())?;
  cookie.set_name(spec.name.clone());
  cookie.set_value(spec.value.clone());
  Ok(cookie)
}

#[cfg_attr(not(mobile), allow(dead_code))]
pub fn imported_cookie_to_set_cookie_header(spec: &ImportedCookieSpec) -> String {
  let mut header = format!("{}={}", spec.name, spec.value);
  if !spec.name.starts_with("__Host-") {
    header.push_str(&format!("; Domain={}", spec.domain));
  }
  header.push_str(&format!("; Path={}", spec.path));
  if let Some(max_age) = spec.max_age_secs {
    header.push_str(&format!("; Max-Age={max_age}"));
  }
  if spec.secure {
    header.push_str("; Secure");
  }
  if spec.http_only {
    header.push_str("; HttpOnly");
  }
  if let Some(ss) = spec.same_site.as_deref() {
    match ss {
      "lax" => header.push_str("; SameSite=Lax"),
      "strict" => header.push_str("; SameSite=Strict"),
      "none" => header.push_str("; SameSite=None"),
      _ => {}
    }
  }
  header
}

#[cfg(desktop)]
pub async fn delete_desktop_profile(
  app: &tauri::AppHandle,
  app_data_dir: &Path,
  uuid_str: &str,
) -> Result<(), String> {
  let parsed = Uuid::parse_str(uuid_str).map_err(|e| format!("非法 UUID: {e}"))?;

  #[cfg(target_os = "macos")]
  {
    let _ = app_data_dir;
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
    let uuid_formatted = parsed.hyphenated().to_string();
    let tx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    app
      .run_on_main_thread(move || {
        use objc2::AnyThread;
        use objc2_foundation::{NSError, NSString, NSUUID};
        use objc2_web_kit::WKWebsiteDataStore;

        let ns_str = NSString::from_str(&uuid_formatted);
        let Some(ns_uuid) = NSUUID::initWithUUIDString(NSUUID::alloc(), &ns_str) else {
          if let Some(sender) = tx_cell.lock().unwrap().take() {
            let _ = sender.send(Err("构造 NSUUID 失败".to_string()));
          }
          return;
        };

        let Some(mtm) = objc2::MainThreadMarker::new() else {
          if let Some(sender) = tx_cell.lock().unwrap().take() {
            let _ = sender.send(Err("当前不在 macOS 主线程".to_string()));
          }
          return;
        };

        let tx_inner = tx_cell.clone();
        let block = block2::RcBlock::new(move |err: *mut NSError| {
          let outcome = if err.is_null() {
            Ok(())
          } else {
            let msg = unsafe { (&*err).localizedDescription().to_string() };
            if msg.to_ascii_lowercase().contains("not exist")
              || msg.to_ascii_lowercase().contains("not found")
            {
              Ok(())
            } else {
              Err(format!("移除 WKWebsiteDataStore 失败: {msg}"))
            }
          };
          if let Some(sender) = tx_inner.lock().unwrap().take() {
            let _ = sender.send(outcome);
          }
        });

        unsafe {
          WKWebsiteDataStore::removeDataStoreForIdentifier_completionHandler(&ns_uuid, &block, mtm);
        }
      })
      .map_err(|e| format!("调度主线程清理 WKWebsiteDataStore 失败: {e}"))?;

    rx
      .await
      .map_err(|_| "等待 WKWebsiteDataStore 清理完成中断".to_string())?
  }

  #[cfg(not(target_os = "macos"))]
  {
    let _ = (app, parsed);
    let profile_dir = app_data_dir.join("profiles").join(uuid_str);
    if profile_dir.exists() {
      std::fs::remove_dir_all(&profile_dir)
        .map_err(|e| format!("清理资料目录被占用或失败: {e}"))?;
    }
    Ok(())
  }
}

#[cfg(windows)]
pub fn lock_windows_manager_webview_zoom(webview: &tauri::Webview) {
  let _ = webview.with_webview(|pwv| unsafe {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings5;
    use windows::core::Interface;

    let controller = pwv.controller();
    let _ = controller.SetZoomFactor(1.0);
    if let Ok(core) = controller.CoreWebView2() {
      if let Ok(settings) = core.Settings() {
        let _ = settings.SetIsZoomControlEnabled(false);
        if let Ok(settings5) = settings.cast::<ICoreWebView2Settings5>() {
          let _ = settings5.SetIsPinchZoomEnabled(false);
        }
      }
    }
  });
}

#[cfg(desktop)]
pub fn reveal_path_in_file_manager(path: &Path) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  {
    if path.exists() {
      std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("打开访达失败: {e}"))
    } else if let Some(parent) = path.parent().filter(|p| p.exists()) {
      open::that_detached(parent).map_err(|e| format!("打开文件夹失败: {e}"))
    } else {
      Err("文件已被移动或删除".to_string())
    }
  }
  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;

    if path.exists() {
      let raw = path.to_string_lossy();
      let clean = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
      std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{clean}\""))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("打开资源管理器失败: {e}"))
    } else if let Some(parent) = path.parent().filter(|p| p.exists()) {
      open::that_detached(parent).map_err(|e| format!("打开文件夹失败: {e}"))
    } else {
      Err("文件已被移动或删除".to_string())
    }
  }
  #[cfg(all(not(target_os = "macos"), not(windows)))]
  {
    if let Some(target) = path.parent().filter(|p| p.exists()) {
      open::that_detached(target).map_err(|e| format!("打开文件夹失败: {e}"))
    } else if path.exists() {
      open::that_detached(path).map_err(|e| format!("打开文件失败: {e}"))
    } else {
      Err("文件已被移动或删除".to_string())
    }
  }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
  pub ok: bool,
  pub latency_ms: u64,
  pub status_code: Option<u16>,
  pub target_host: String,
  pub used_proxy: bool,
  pub message: String,
}

#[cfg(desktop)]
pub async fn test_account_proxy(target_url: &str, raw_proxy: &str) -> Result<ProxyTestResult, String> {
  let proxy_opt = parse_account_proxy(raw_proxy)?;
  let parsed_target = url::Url::parse(target_url.trim())
    .map_err(|_| "目标服务入口地址无效".to_string())?;
  let target_host = parsed_target
    .host_str()
    .filter(|h| !h.is_empty())
    .ok_or_else(|| "目标服务缺少主机名".to_string())?
    .to_string();
  let used_proxy = proxy_opt.is_some();

  let mut builder = reqwest::Client::builder()
    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15")
    .connect_timeout(std::time::Duration::from_secs(6))
    .timeout(std::time::Duration::from_secs(10))
    .redirect(reqwest::redirect::Policy::none());

  if let Some(ref proxy_url) = proxy_opt {
    let proxy = reqwest::Proxy::all(proxy_url.as_str())
      .map_err(|e| format!("构造代理客户端失败: {e}"))?;
    builder = builder.proxy(proxy);
  }

  let client = builder
    .build()
    .map_err(|e| format!("初始化测速客户端失败: {e}"))?;

  let start = std::time::Instant::now();
  let mut res = client.head(parsed_target.clone()).send().await;
  if let Ok(ref resp) = res {
    if resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
      res = client.get(parsed_target).send().await;
    }
  }
  let latency_ms = start.elapsed().as_millis() as u64;

  match res {
    Ok(resp) => {
      let code = resp.status().as_u16();
      let prefix = if used_proxy { "代理连通" } else { "系统网络连通" };
      Ok(ProxyTestResult {
        ok: true,
        latency_ms,
        status_code: Some(code),
        target_host,
        used_proxy,
        message: format!("{prefix} · {latency_ms} ms (HTTP {code})"),
      })
    }
    Err(err) => {
      let reason = if err.is_timeout() {
        "连接超时".to_string()
      } else if err.is_connect() {
        if used_proxy {
          "无法连接到代理或代理握手失败".to_string()
        } else {
          "无法连接到目标服务器".to_string()
        }
      } else {
        format!("请求失败: {err}")
      };
      let prefix = if used_proxy { "代理不可用" } else { "连接失败" };
      Ok(ProxyTestResult {
        ok: false,
        latency_ms,
        status_code: None,
        target_host,
        used_proxy,
        message: format!("{prefix}（{reason}，耗时 {latency_ms} ms）"),
      })
    }
  }
}

#[cfg(not(desktop))]
#[allow(dead_code)]
pub async fn test_account_proxy(_target_url: &str, _raw_proxy: &str) -> Result<ProxyTestResult, String> {
  Err("当前平台不支持代理测试".to_string())
}

#[cfg(mobile)]
pub use tauri_plugin_account_session as mobile;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_and_format_account_proxy_rules() {
    assert_eq!(parse_account_proxy("   "), Ok(None));

    if !SUPPORTS_ACCOUNT_PROXY {
      assert!(parse_account_proxy("http://127.0.0.1:7890").is_err());
      return;
    }

    let ipv4 = parse_account_proxy("  HTTP://127.0.0.1:7890/ ").unwrap().unwrap();
    assert_eq!(format_account_proxy(&ipv4), "http://127.0.0.1:7890");

    let ipv6 = parse_account_proxy("http://[::1]:10808").unwrap().unwrap();
    assert_eq!(format_account_proxy(&ipv6), "http://[::1]:10808");

    let domain = parse_account_proxy("http://proxy_1.example-host.local:80").unwrap().unwrap();
    assert_eq!(format_account_proxy(&domain), "http://proxy_1.example-host.local:80");

    for invalid in [
      "http://127.0.0.1",
      "http://127.0.0.1/",
      "http://127.0.0.1:",
      "http://[::1]",
      "http://[::1]/",
      "http://127.0.0.1:0",
      "socks5://127.0.0.1:7890",
      "https://127.0.0.1:7890",
      "http://user:pass@127.0.0.1:7890",
      "http://127.0.0.1:7890/path",
      "http://127.0.0.1:7890?a=1",
      "http://127.0.0.1:7890#frag",
      "http://127.0.0.1: 7890",
      "http://proxy!bad.local:7890",
    ] {
      assert!(parse_account_proxy(invalid).is_err(), "expected error for {invalid}");
    }
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn macos_http_connect_proxy_and_nav_error_filter() {
    for raw in [
      "http://127.0.0.1:7890",
      "http://[::1]:10808",
      "http://proxy.internal:80",
    ] {
      let url = parse_account_proxy(raw).unwrap().unwrap();
      assert!(create_http_connect_proxy(&url).is_ok(), "failed for {raw}");
    }

    assert!(!should_fail_macos_navigation("NSURLErrorDomain", -999));
    assert!(!should_fail_macos_navigation("WebKitErrorDomain", 102));
    assert!(should_fail_macos_navigation("NSURLErrorDomain", -1004));
    assert!(should_fail_macos_navigation("NSURLErrorDomain", -1012));
    assert!(should_fail_macos_navigation("NSPOSIXErrorDomain", 61));
    assert!(should_fail_macos_navigation("WebKitErrorDomain", 101));
  }

  #[cfg(windows)]
  #[test]
  fn windows_proxy_args_and_error_matching() {
    let url80 = parse_account_proxy("http://proxy.local:80").unwrap().unwrap();
    let args80 = windows_proxy_browser_args(&url80);
    assert!(args80.contains("--proxy-server=http://proxy.local:80"));
    assert!(args80.contains("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"));
    assert!(args80.contains("--autoplay-policy=no-user-gesture-required"));

    let ipv6 = parse_account_proxy("http://[::1]:7890").unwrap().unwrap();
    assert!(windows_proxy_browser_args(&ipv6).contains("--proxy-server=http://[::1]:7890"));

    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_STATE};
    let in_use_err = tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(Box::new(
      wry::Error::WebView2Error(webview2_com::Error::from(ERROR_INVALID_STATE.to_hresult())),
    )));
    assert!(windows_webview_options_in_use(&in_use_err));

    let other_err = tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(Box::new(
      wry::Error::WebView2Error(webview2_com::Error::from(ERROR_ACCESS_DENIED.to_hresult())),
    )));
    assert!(!windows_webview_options_in_use(&other_err));
  }

  #[test]
  fn summarize_cookie_expiry_prioritizes_auth_cookies_and_ignores_noise() {
    let now = 1_700_000_000_000;

    // 1. Filters expired and noise cookies; picks max among httpOnly+auth tier.
    let cookies = vec![
      CookieMeta {
        name: "_ga".to_string(),
        http_only: false,
        session_only: false,
        expires_at_ms: Some(now + 100_000_000),
      },
      CookieMeta {
        name: "cf_clearance".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now + 90_000_000),
      },
      CookieMeta {
        name: "expired_session".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now - 1_000),
      },
      CookieMeta {
        name: "access_token".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now + 3_600_000),
      },
      CookieMeta {
        name: "refresh_token".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now + 30 * 86_400_000),
      },
      CookieMeta {
        name: "ui_state".to_string(),
        http_only: false,
        session_only: false,
        expires_at_ms: Some(now + 365 * 86_400_000),
      },
    ];
    assert_eq!(
      summarize_cookie_expiry(&cookies, now),
      CookieExpirySummary {
        expires_at_ms: Some(now + 30 * 86_400_000),
        session_only: false,
      }
    );

    // 2. Falls back to auth without httpOnly before non-auth httpOnly.
    let auth_non_httponly = vec![
      CookieMeta {
        name: "opaque_pref".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now + 10 * 86_400_000),
      },
      CookieMeta {
        name: "c_user".to_string(),
        http_only: false,
        session_only: false,
        expires_at_ms: Some(now + 90 * 86_400_000),
      },
    ];
    assert_eq!(
      summarize_cookie_expiry(&auth_non_httponly, now),
      CookieExpirySummary {
        expires_at_ms: Some(now + 90 * 86_400_000),
        session_only: false,
      }
    );

    // 3. Session-only auth cookies report session_only = true.
    let session_only = vec![
      CookieMeta {
        name: "__Host-next-auth.csrf-token".to_string(),
        http_only: true,
        session_only: false,
        expires_at_ms: Some(now + 86_400_000),
      },
      CookieMeta {
        name: "JSESSIONID".to_string(),
        http_only: true,
        session_only: true,
        expires_at_ms: None,
      },
    ];
    assert_eq!(
      summarize_cookie_expiry(&session_only, now),
      CookieExpirySummary {
        expires_at_ms: None,
        session_only: true,
      }
    );

    // 4. Noise-only returns empty summary.
    let noise_only = vec![CookieMeta {
      name: "__cf_bm".to_string(),
      http_only: true,
      session_only: false,
      expires_at_ms: Some(now + 1_800_000),
    }];
    assert_eq!(
      summarize_cookie_expiry(&noise_only, now),
      CookieExpirySummary::default()
    );
  }

  #[test]
  fn parse_imported_cookies_supports_json_header_netscape_and_filters_domains() {
    let allowed = vec!["muse.ai".to_string(), "meta.com".to_string()];
    let now = 1_700_000_000i64;

    assert!(parse_imported_cookies("   ", "muse.ai", &allowed, now).is_err());

    // 1. JSON array with __Host- normalization, expired skip, and unrelated domain skip
    let json_raw = r#"[
      {"name":"__Host-sess","value":"tok1","domain":".muse.ai","path":"/app","secure":false,"httpOnly":true,"sameSite":"lax","expirationDate":1700086400},
      {"name":"expired_ck","value":"dead","domain":"muse.ai","expirationDate":1699999999},
      {"name":"other_site","value":"bad","domain":"google.com","expirationDate":1700086400}
    ]"#;
    let parsed_json = parse_imported_cookies(json_raw, "muse.ai", &allowed, now).unwrap();
    assert_eq!(parsed_json.len(), 1);
    assert_eq!(parsed_json[0].name, "__Host-sess");
    assert_eq!(parsed_json[0].domain, "muse.ai");
    assert_eq!(parsed_json[0].path, "/");
    assert!(parsed_json[0].secure);
    assert!(parsed_json[0].http_only);
    assert_eq!(parsed_json[0].same_site.as_deref(), Some("lax"));
    assert_eq!(parsed_json[0].max_age_secs, Some(86_400));
    let header_line = imported_cookie_to_set_cookie_header(&parsed_json[0]);
    assert!(header_line.starts_with("__Host-sess=tok1; Path=/"));
    assert!(!header_line.contains("Domain="));

    // 2. HTTP Cookie header string
    let hdr = "Cookie: hatch_sess=abc==; pref=dark";
    let parsed_hdr = parse_imported_cookies(hdr, "muse.ai", &allowed, now).unwrap();
    assert_eq!(parsed_hdr.len(), 2);
    assert_eq!(parsed_hdr[0].name, "hatch_sess");
    assert_eq!(parsed_hdr[0].value, "abc==");
    assert_eq!(parsed_hdr[1].name, "pref");
    assert_eq!(parsed_hdr[1].value, "dark");

    // 3. Netscape cookies.txt (including #HttpOnly_ prefix)
    let netscape = format!(
      "# Netscape HTTP Cookie File\n#HttpOnly_.muse.ai\tTRUE\t/\tTRUE\t{}\thatch_sess\tval123\n.google.com\tTRUE\t/\tTRUE\t{}\tNID\tignored\n",
      now + 3600,
      now + 3600
    );
    let parsed_net = parse_imported_cookies(&netscape, "muse.ai", &allowed, now).unwrap();
    assert_eq!(parsed_net.len(), 1);
    assert_eq!(parsed_net[0].name, "hatch_sess");
    assert_eq!(parsed_net[0].value, "val123");
    assert!(parsed_net[0].http_only);

    // 4. All filtered out -> explicit error
    assert!(parse_imported_cookies(
      r#"[{"name":"SID","value":"x","domain":"google.com"}]"#,
      "muse.ai",
      &allowed,
      now
    )
    .is_err());

    #[cfg(desktop)]
    {
      let tauri_ck = imported_cookie_to_tauri(&parsed_json[0]).unwrap();
      assert_eq!(tauri_ck.name(), "__Host-sess");
      assert_eq!(tauri_ck.value(), "tok1");
    }
  }

  #[test]
  fn extract_identity_hint_from_cookies_handles_urlencoded_jwt_and_filters_noise() {
    let mk = |name: &str, value: &str| ImportedCookieSpec {
      name: name.to_string(),
      value: value.to_string(),
      domain: "muse.ai".to_string(),
      path: "/".to_string(),
      secure: true,
      http_only: true,
      same_site: Some("lax".to_string()),
      max_age_secs: Some(3600),
    };

    assert_eq!(
      extract_identity_hint_from_cookies(&[
        mk("sys", "noreply%40muse.ai"),
        mk("login_hint", "User.Name%40Gmail.com"),
      ]),
      Some("user.name@gmail.com".to_string())
    );

    // Base64URL JWT payload {"email":"alice@example.org","sub":"123"} -> eyJlbWFpbCI6ImFsaWNlQGV4YW1wbGUub3JnIiwic3ViIjoiMTIzIn0
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJlbWFpbCI6ImFsaWNlQGV4YW1wbGUub3JnIiwic3ViIjoiMTIzIn0.sig";
    assert_eq!(
      extract_identity_hint_from_cookies(&[mk("id_token", jwt)]),
      Some("alice@example.org".to_string())
    );

    // System / placeholder emails only -> None
    assert_eq!(
      extract_identity_hint_from_cookies(&[
        mk("a", "support@muse.ai"),
        mk("b", "test@example.com"),
      ]),
      None
    );
  }
}
