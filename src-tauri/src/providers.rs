use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
  pub fn muse() -> Self {
    Self("muse".to_string())
  }

  pub fn hark() -> Self {
    Self("hark".to_string())
  }

  pub fn api() -> Self {
    Self("api".to_string())
  }

  pub fn cli() -> Self {
    Self("cli".to_string())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn is_muse(&self) -> bool {
    self.0 == "muse"
  }

  pub fn is_hark(&self) -> bool {
    self.0 == "hark"
  }

  pub fn is_api(&self) -> bool {
    self.0 == "api"
  }

  pub fn is_cli(&self) -> bool {
    self.0 == "cli"
  }

  #[cfg_attr(not(desktop), allow(dead_code))]
  pub fn is_external(&self) -> bool {
    self.is_api() || self.is_cli()
  }

  pub fn is_builtin(&self) -> bool {
    self.is_muse() || self.is_hark()
  }

  pub fn parse(value: &str) -> Option<Self> {
    let clean = value.trim().to_ascii_lowercase();
    if clean == "muse" || clean == "hark" || clean == "api" || clean == "cli" {
      Some(Self(clean))
    } else {
      None
    }
  }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
  pub id: ProviderId,
  pub name: String,
  pub short_name: String,
  pub enabled: bool,
  pub status_text: String,
  pub entry_url: String,
  pub homepage_url: String,
  pub callback_origins: Vec<String>,
  pub verified_methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderSpec {
  pub info: ProviderInfo,
  pub allowed_hosts: Vec<String>,
}

pub fn builtin_provider_specs() -> Vec<ProviderSpec> {
  vec![
    ProviderSpec {
      info: ProviderInfo {
        id: ProviderId::muse(),
        name: "Muse · Meta".to_string(),
        short_name: "Muse".to_string(),
        enabled: true,
        status_text: "已启用".to_string(),
        entry_url: "https://muse.ai/".to_string(),
        homepage_url: "https://muse.ai/".to_string(),
        callback_origins: vec![
          "https://muse.ai".to_string(),
          "https://www.muse.ai".to_string(),
        ],
        verified_methods: vec!["email".to_string(), "phone".to_string()],
      },
      allowed_hosts: vec![
        "muse.ai".to_string(),
        "meta.com".to_string(),
        "meta.ai".to_string(),
        "metaaiusercontent.com".to_string(),
        "ecto1usercontent.com".to_string(),
        "meta-agents-apps.workers.dev".to_string(),
        "facebook.com".to_string(),
        "fbsbx.com".to_string(),
        "fbcdn.net".to_string(),
        "instagram.com".to_string(),
        "cdninstagram.com".to_string(),
        "fb.com".to_string(),
        "appleid.apple.com".to_string(),
        "idmsa.apple.com".to_string(),
      ],
    },
    ProviderSpec {
      info: ProviderInfo {
        id: ProviderId::hark(),
        name: "Hark".to_string(),
        short_name: "Hark".to_string(),
        enabled: true,
        status_text: "已启用".to_string(),
        entry_url: "https://app.hark.com/".to_string(),
        homepage_url: "https://hark.com/".to_string(),
        callback_origins: vec![
          "https://app.hark.com".to_string(),
          "https://hark.com".to_string(),
        ],
        verified_methods: vec![
          "email".to_string(),
          "phone".to_string(),
          "apple".to_string(),
        ],
      },
      allowed_hosts: vec![
        "app.hark.com".to_string(),
        "hark.com".to_string(),
        "appleid.apple.com".to_string(),
        "idmsa.apple.com".to_string(),
      ],
    },
  ]
}

pub fn resolve_provider(id: &ProviderId) -> Option<ProviderSpec> {
  if !id.is_builtin() {
    return None;
  }
  builtin_provider_specs()
    .into_iter()
    .find(|p| &p.info.id == id)
}

pub fn all_provider_infos() -> Vec<ProviderInfo> {
  builtin_provider_specs()
    .into_iter()
    .map(|p| p.info)
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDecision {
  AllowBlank,
  AllowHttps,
  ExternalGoogleOAuth,
  Deny,
}

pub fn host_matches(host: &str, domain: &str) -> bool {
  if host.eq_ignore_ascii_case(domain) {
    return true;
  }
  let suffix = format!(".{}", domain.to_ascii_lowercase());
  host.to_ascii_lowercase().ends_with(&suffix)
}

const EMBEDDED_WIDGET_HOSTS: &[&str] = &[
  "challenges.cloudflare.com",
  "turnstile.cloudflare.com",
  "hcaptcha.com",
  "recaptcha.net",
  "recaptcha.google.com",
  "gstatic.com",
  "fbsbx.com",
  "arkoselabs.com",
  "funcaptcha.com",
  "friendlycaptcha.com",
  "geetest.com",
  "captcha-delivery.com",
  "firebaseapp.com",
  "stripe.com",
  "stripecdn.com",
];

fn is_embedded_widget_url(url: &Url, host: &str) -> bool {
  if EMBEDDED_WIDGET_HOSTS.iter().any(|allowed| host_matches(host, allowed)) {
    return true;
  }
  (host.eq_ignore_ascii_case("www.google.com") || host.eq_ignore_ascii_case("google.com"))
    && url.path().starts_with("/recaptcha")
}

pub fn classify_top_level_url(spec: &ProviderSpec, url: &Url, allow_blank: bool) -> NavigationDecision {
  let raw = url.as_str();
  if raw == "about:srcdoc"
    || raw.starts_with("blob:https://")
    || raw.starts_with("blob:null/")
  {
    return NavigationDecision::AllowBlank;
  }
  if allow_blank && raw == "about:blank" {
    return NavigationDecision::AllowBlank;
  }
  if url.scheme() != "https" {
    return NavigationDecision::Deny;
  }
  let Some(host) = url.host_str() else {
    return NavigationDecision::Deny;
  };
  if host_matches(host, "accounts.google.com") {
    return NavigationDecision::ExternalGoogleOAuth;
  }
  if spec.allowed_hosts.iter().any(|allowed| host_matches(host, allowed))
    || is_embedded_widget_url(url, host)
  {
    NavigationDecision::AllowHttps
  } else {
    NavigationDecision::Deny
  }
}

pub fn is_verified_callback_origin(spec: &ProviderSpec, url: &Url) -> bool {
  if url.scheme() != "https" {
    return false;
  }
  let origin = url.origin().ascii_serialization();
  spec
    .info
    .callback_origins
    .iter()
    .any(|allowed| allowed.eq_ignore_ascii_case(&origin))
}
