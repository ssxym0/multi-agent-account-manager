import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";
import type {
  AccountKey,
  AccountRecord,
  DownloadNoticeEvent,
  ExternalAgentKind,
  ExternalAgentRecord,
  GroupChatMessage,
  GroupChatRecord,
  GroupMemberRole,
  GroupSharedSkill,
  GroupWorkflowMode,
  InstallUpdateOutcome,
  ManagerSnapshot,
  ProviderId,
  ProviderInfo,
  ProxyTestResult,
  QuotaDetail,
  SiteAdaptersSnapshot,
  ThemeMode,
  UpdateCheckResult,
  UpdateProgressEvent,
} from "./types";

type AccountDraft = {
  draft: string;
  proxyDraft: string;
  timezoneDraft: string;
  cookieDraft?: string;
  cookieImporting?: boolean;
  error?: string;
  saving?: boolean;
  proxyTesting?: boolean;
  proxyTestResult?: ProxyTestResult | null;
  confirmClear?: boolean;
};

const TIMEZONE_PRESETS: Array<{ label: string; value: string }> = [
  { label: "跟随系统", value: "" },
  { label: "纽约", value: "America/New_York" },
  { label: "洛杉矶", value: "America/Los_Angeles" },
  { label: "伦敦", value: "Europe/London" },
  { label: "东京", value: "Asia/Tokyo" },
  { label: "新加坡", value: "Asia/Singapore" },
  { label: "上海", value: "Asia/Shanghai" },
  { label: "UTC", value: "UTC" },
];

type ModalMode =
  | { kind: "none" }
  | ({ kind: "add"; providerId: ProviderId } & AccountDraft)
  | ({ kind: "rename"; account: AccountRecord } & AccountDraft)
  | { kind: "remove"; account: AccountRecord }
  | {
      kind: "group";
      group?: GroupChatRecord;
      nameDraft: string;
      selectedMembers: AccountKey[];
      error?: string;
      saving?: boolean;
    }
  | {
      kind: "group-knowledge";
      group: GroupChatRecord;
      tab: "context" | "skills" | "roles" | "adapters";
      contextDraft: string;
      skillsDraft: GroupSharedSkill[];
      rolesDraft: GroupMemberRole[];
      skillNameDraft: string;
      skillDescDraft: string;
      skillContentDraft: string;
      editingSkillId: string | null;
      adaptersSnap?: SiteAdaptersSnapshot | null;
      adaptersJsonDraft: string;
      adaptersLoaded?: boolean;
      error?: string;
      saving?: boolean;
    }
  | {
      kind: "external-agents";
      editingId: string | null;
      formKind: ExternalAgentKind;
      nameDraft: string;
      baseUrlDraft: string;
      apiKeyDraft: string;
      modelDraft: string;
      commandDraft: string;
      argsDraft: string;
      workdirDraft: string;
      error?: string;
      saving?: boolean;
    }
  | { kind: "delete-group"; group: GroupChatRecord }
  | {
      kind: "settings";
      minimizeToTray: boolean;
      globalShortcut: string;
      autoApprove: boolean;
      error?: string;
      saving?: boolean;
    }
  | { kind: "update" };

type ProviderLogoSpec = { src: string; fullBleed: boolean; monoDark?: boolean };

const PROVIDER_LOGOS: Record<string, ProviderLogoSpec> = {
  muse: { src: "/logos/muse.png", fullBleed: true },
  hark: { src: "/logos/hark.svg", fullBleed: false, monoDark: true },
};

const GROUP_SKILL_PRESETS: Array<Omit<GroupSharedSkill, "id" | "enabled">> = [
  {
    name: "极致简洁中文",
    description: "约束各成员直接给结论和关键点，禁止寒暄和铺垫。",
    content: "统一使用简体中文回复；直接给结论、关键依据和可执行方案，禁止客套和冗余铺垫。",
  },
  {
    name: "架构与风险评审",
    description: "从稳定性、边界情况、安全性和性能四个维度输出意见。",
    content: "分析技术方案时优先检查边界情况、并发竞争、权限安全与资源占用，明确指出取舍与潜在风险。",
  },
  {
    name: "交叉事实核查",
    description: "要求先指出前序发言或前提中的漏洞，再补充新结论。",
    content: "若共享上下文或前序成员发言存在矛盾、假设漏洞或遗漏，请先指出修正，再给出补充判断。",
  },
];

const EXTERNAL_AGENT_PRESETS: Array<{
  label: string;
  kind: ExternalAgentKind;
  name: string;
  baseUrl: string;
  model: string;
  command: string;
  args: string;
}> = [
  {
    label: "OpenAI",
    kind: "openai_api",
    name: "GPT-4o",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o",
    command: "",
    args: "",
  },
  {
    label: "DeepSeek",
    kind: "openai_api",
    name: "DeepSeek",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    command: "",
    args: "",
  },
  {
    label: "Ollama 本地",
    kind: "openai_api",
    name: "Ollama-Qwen",
    baseUrl: "http://127.0.0.1:11434/v1",
    model: "qwen2.5:14b",
    command: "",
    args: "",
  },
  {
    label: "Claude CLI",
    kind: "local_cli",
    name: "Claude-Code",
    baseUrl: "",
    model: "",
    command: "claude",
    args: "-p",
  },
  {
    label: "Codex CLI",
    kind: "local_cli",
    name: "Codex-CLI",
    baseUrl: "",
    model: "",
    command: "codex",
    args: "exec {prompt}",
  },
];

const ICON_EXTERNAL = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>`;
const ICON_REFRESH = `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>`;
const ICON_EDIT = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>`;
const ICON_TRASH = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>`;
const ICON_USER_PLUS = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="19" y1="8" x2="19" y2="14"/><line x1="22" y1="11" x2="16" y2="11"/></svg>`;
const ICON_UPDATE = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`;
const ICON_SETTINGS = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`;
const ICON_THEME_DARK = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>`;
const ICON_THEME_LIGHT = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><line x1="12" y1="2" x2="12" y2="4"/><line x1="12" y1="20" x2="12" y2="22"/><line x1="4.93" y1="4.93" x2="6.34" y2="6.34"/><line x1="17.66" y1="17.66" x2="19.07" y2="19.07"/><line x1="2" y1="12" x2="4" y2="12"/><line x1="20" y1="12" x2="22" y2="12"/><line x1="6.34" y1="17.66" x2="4.93" y2="19.07"/><line x1="19.07" y1="4.93" x2="17.66" y2="6.34"/></svg>`;
const ICON_THEME_SYSTEM = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`;
const ICON_BOLT = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`;
const ICON_SORT = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M6 12h12"/><path d="M10 18h4"/></svg>`;
const ICON_GROUP = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M8 10h.01"/><path d="M12 10h.01"/><path d="M16 10h.01"/></svg>`;
const ICON_PIN = `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/><line x1="12" x2="12" y1="7" y2="13"/><line x1="15" x2="9" y1="10" y2="10"/></svg>`;
const syncingMsgIds = new Set<string>();

const THEME_STORAGE_KEY = "maam_theme_mode";
const systemDarkQuery = window.matchMedia("(prefers-color-scheme: dark)");
let themeMode: ThemeMode = loadSavedThemeMode();

function loadSavedThemeMode(): ThemeMode {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    if (raw === "light" || raw === "dark" || raw === "system") {
      return raw;
    }
  } catch {
    // ignore storage errors
  }
  return "system";
}

function effectiveTheme(): "light" | "dark" {
  if (themeMode === "system") {
    return systemDarkQuery.matches ? "dark" : "light";
  }
  return themeMode;
}

function applyTheme(): void {
  document.documentElement.dataset.theme = effectiveTheme();
}

function themeLabel(mode: ThemeMode): string {
  if (mode === "dark") return "深色";
  if (mode === "light") return "浅色";
  return "跟随";
}

function themeIcon(mode: ThemeMode): string {
  if (mode === "dark") return ICON_THEME_DARK;
  if (mode === "light") return ICON_THEME_LIGHT;
  return ICON_THEME_SYSTEM;
}

function cycleThemeMode(): void {
  themeMode = themeMode === "system" ? "light" : themeMode === "light" ? "dark" : "system";
  try {
    localStorage.setItem(THEME_STORAGE_KEY, themeMode);
  } catch {
    // ignore storage errors
  }
  applyTheme();
  rerenderPreservingScroll();
}

applyTheme();
systemDarkQuery.addEventListener("change", () => {
  if (themeMode === "system") {
    applyTheme();
    if (snapshot) {
      rerenderPreservingScroll();
    }
  }
});

let snapshot: ManagerSnapshot | null = null;
let modal: ModalMode = { kind: "none" };
let optimisticSwitching: AccountKey | null = null;
let busy = false;
let viewMode: "provider" | "groups" = "provider";
let lastGroupListScroll = 0;
let shouldAnimateModal = false;
let modalOverlayScrollTop = 0;
let groupMemberPickerScrollTop = 0;
let modalCardScrollTop = 0;

function modalOverlayClass(): string {
  return shouldAnimateModal ? "inline-modal animate-in" : "inline-modal";
}
const groupDrafts: Record<string, string> = {};
const groupTargets: Record<string, AccountKey[]> = {};
const groupAutoChannels: Record<string, string[]> = {};
let newTopicConfirmGroupId: string | null = null;
const groupScrollOffsets: Record<string, number> = {};
let groupScrolledToBottom = true;
let forceGroupScrollBottom = false;
let groupSending = false;
let lastRenderedUpdateStage: string | null = null;
const copyingCookies = new Set<string>();
let downloadToast: DownloadNoticeEvent | null = null;
let downloadToastTimer: number | undefined;
let draggingAccountUuid: string | null = null;

function clearDownloadToastTimer(): void {
  if (downloadToastTimer !== undefined) {
    window.clearTimeout(downloadToastTimer);
    downloadToastTimer = undefined;
  }
}

function scheduleDownloadToastDismiss(): void {
  clearDownloadToastTimer();
  if (!downloadToast || downloadToast.status === "started") return;
  downloadToastTimer = window.setTimeout(() => {
    downloadToast = null;
    rerenderPreservingScroll();
  }, 12000);
}

const updateState: {
  checking: boolean;
  installing: boolean;
  result: UpdateCheckResult | null;
  progress: UpdateProgressEvent | null;
  error: string | null;
  outcomeStatus: string | null;
  outcomeMessage: string | null;
  lastCheckedAt: number;
  lastFailedAt: number;
} = {
  checking: false,
  installing: false,
  result: null,
  progress: null,
  error: null,
  outcomeStatus: null,
  outcomeMessage: null,
  lastCheckedAt: 0,
  lastFailedAt: 0,
};

const scrollOffsets: Record<string, number> = {};

function isSameKey(a: AccountKey | null, b: AccountRecord): boolean {
  return Boolean(a && a.providerId === b.providerId && a.uuid === b.uuid);
}

function sameAccountKey(a: AccountKey, b: AccountKey): boolean {
  return a.providerId === b.providerId && a.uuid === b.uuid;
}

function isMobilePlatform(platform: ManagerSnapshot["platform"] | undefined): boolean {
  return platform === "android" || platform === "ios";
}

function syncViewModeFromSnapshot(state: ManagerSnapshot | null): void {
  if (!state) return;
  if (isMobilePlatform(state.platform)) {
    viewMode = "provider";
    return;
  }
  if (state.activeGroupId) {
    viewMode = "groups";
  } else if (state.activeAccount || state.switchingAccount) {
    viewMode = "provider";
  }
}

function findAccountPageError(
  state: ManagerSnapshot | null,
  key: AccountKey
): string | null {
  if (!state?.accountErrors) return null;
  const hit = state.accountErrors.find(
    (e) => e.providerId === key.providerId && e.uuid === key.uuid
  );
  return hit && hit.message.trim() ? hit.message.trim() : null;
}

function isAccountGroupEligible(state: ManagerSnapshot, acc: AccountRecord): boolean {
  const provider = state.providers.find((p) => p.id === acc.providerId);
  if (!provider || !provider.enabled) return false;
  if (acc.authState !== "authenticated") return false;
  if (typeof acc.cookieExpiresAtMs === "number" && acc.cookieExpiresAtMs <= Date.now()) {
    return false;
  }
  if (findAccountPageError(state, acc)) return false;
  return true;
}

function formatShortTime(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function groupHasNeedsAuth(group: GroupChatRecord): boolean {
  return group.messages.some((m) => m.status === "needs_auth");
}

function isExternalProviderId(providerId: ProviderId): boolean {
  return providerId === "api" || providerId === "cli";
}

function findAccountRecord(state: ManagerSnapshot, key: AccountKey | null): AccountRecord | undefined {
  if (!key || isExternalProviderId(key.providerId)) return undefined;
  return state.accounts.find((a) => a.providerId === key.providerId && a.uuid === key.uuid);
}

function findExternalAgent(state: ManagerSnapshot, key: AccountKey | null): ExternalAgentRecord | undefined {
  if (!key || !isExternalProviderId(key.providerId)) return undefined;
  return (state.externalAgents ?? []).find((e) => e.id === key.uuid);
}

function resolveMemberMeta(
  state: ManagerSnapshot,
  key: AccountKey
): {
  key: AccountKey;
  isExternal: boolean;
  acc?: AccountRecord;
  ext?: ExternalAgentRecord;
  alias: string;
  providerShort: string;
  loggedOut: boolean;
  pageError: string | null;
  warm: boolean;
} {
  if (isExternalProviderId(key.providerId)) {
    const ext = findExternalAgent(state, key);
    return {
      key,
      isExternal: true,
      ext,
      alias: ext?.name ?? key.uuid.slice(0, 6),
      providerShort: key.providerId === "api" ? "API" : "CLI",
      loggedOut: !ext,
      pageError: null,
      warm: Boolean(ext),
    };
  }
  const acc = findAccountRecord(state, key);
  const pInfo = state.providers.find((p) => p.id === key.providerId);
  const loggedOut = !acc || acc.authState === "logged_out";
  return {
    key,
    isExternal: false,
    acc,
    alias: acc?.alias ?? key.uuid.slice(0, 6),
    providerShort: pInfo?.shortName ?? key.providerId.toUpperCase(),
    loggedOut,
    pageError: loggedOut ? null : findAccountPageError(state, key),
    warm: state.warmAccounts.some((w) => sameAccountKey(w, key)),
  };
}

function resolveMemberCategory(
  state: ManagerSnapshot,
  key: AccountKey
): { id: string; label: string } {
  if (key.providerId === "api") {
    return { id: "api", label: "OpenAI API" };
  }
  if (key.providerId === "cli") {
    return { id: "cli", label: "本地 CLI" };
  }
  const pInfo = state.providers.find((p) => p.id === key.providerId);
  return {
    id: pInfo?.id ?? key.providerId,
    label: pInfo?.shortName ?? key.providerId.toUpperCase(),
  };
}

function groupByMemberCategory<T extends { key: AccountKey }>(
  state: ManagerSnapshot,
  items: T[]
): Array<{ id: string; label: string; items: T[] }> {
  const buckets = new Map<string, { id: string; label: string; items: T[] }>();
  for (const item of items) {
    const cat = resolveMemberCategory(state, item.key);
    let bucket = buckets.get(cat.id);
    if (!bucket) {
      bucket = { ...cat, items: [] };
      buckets.set(cat.id, bucket);
    }
    bucket.items.push(item);
  }
  const orderedIds: string[] = [
    ...state.providers.map((p) => p.id),
    "api",
    "cli",
  ];
  const result: Array<{ id: string; label: string; items: T[] }> = [];
  for (const id of orderedIds) {
    const b = buckets.get(id);
    if (b && b.items.length > 0) {
      result.push(b);
      buckets.delete(id);
    }
  }
  for (const b of buckets.values()) {
    if (b.items.length > 0) {
      result.push(b);
    }
  }
  return result;
}

function resolveGroupMessageSenderName(state: ManagerSnapshot, msg: GroupChatMessage): string {
  if (msg.sender && isExternalProviderId(msg.sender.providerId)) {
    return findExternalAgent(state, msg.sender)?.name || msg.senderName;
  }
  const acc = findAccountRecord(state, msg.sender);
  return acc?.alias || msg.senderName;
}

function renderMemberAvatarNode(
  _state: ManagerSnapshot,
  acc: AccountRecord | undefined,
  fallbackName: string,
  extraClass = "",
  externalKind?: ExternalAgentKind
): string {
  if (externalKind) {
    const extCls = externalKind === "openai_api" ? "ext-api" : "ext-cli";
    const badge = externalKind === "openai_api" ? "API" : "CLI";
    return `<div class="avatar ${extCls} ${escapeHtml(extraClass)}">${badge}</div>`;
  }
  if (acc?.avatarUrl) {
    return `<div class="avatar has-photo ${escapeHtml(extraClass)}"><img src="${escapeHtml(acc.avatarUrl)}" class="avatar-photo" alt="" draggable="false" /></div>`;
  }
  const cls = acc ? acc.providerId : "custom";
  const initial = Array.from((acc?.alias || fallbackName).trim())[0]?.toUpperCase() || "#";
  return `<div class="avatar ${cls} ${escapeHtml(extraClass)}">${escapeHtml(initial)}</div>`;
}

function effectiveSwitchingKey(state: ManagerSnapshot): AccountKey | null {
  return state.switchingAccount ?? optimisticSwitching;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderProviderLogo(provider: ProviderInfo, extraClass = ""): string {
  const cfg = PROVIDER_LOGOS[provider.id];
  if (cfg) {
    const cls = `${extraClass} ${cfg.fullBleed ? "full-bleed" : ""} ${cfg.monoDark ? "mono-dark-logo" : ""}`.trim();
    return `<img src="${cfg.src}" alt="${escapeHtml(provider.shortName)}" class="${cls}" draggable="false" />`;
  }
  const initial = Array.from(provider.shortName.trim())[0]?.toUpperCase() || "A";
  return `<span class="group-provider-tag ${escapeHtml(extraClass)}">${escapeHtml(initial)}</span>`;
}

function renderCategoryIcon(
  state: ManagerSnapshot,
  cat: { id: string; label: string }
): string {
  if (cat.id === "api") {
    return `<span class="cat-ext-icon ext-api" aria-hidden="true">API</span>`;
  }
  if (cat.id === "cli") {
    return `<span class="cat-ext-icon ext-cli" aria-hidden="true">&gt;_</span>`;
  }
  const provider = state.providers.find((p) => p.id === cat.id);
  if (provider) {
    return renderProviderLogo(provider, "cat-logo-img");
  }
  const initial = Array.from(cat.label.trim())[0]?.toUpperCase() || "A";
  return `<span class="group-provider-tag">${escapeHtml(initial)}</span>`;
}

function computeAvatarText(acc: AccountRecord): string {
  if (acc.autoAlias) {
    return "@";
  }
  const clean = acc.alias.trim();
  const first = Array.from(clean)[0];
  return first ? first.toUpperCase() : "#";
}

function splitQuotaValue(raw: string): { main: string; note: string | null } {
  const m = raw.trim().match(/^(.*?)\s*[（(]([^）)]+)[）)]\s*$/);
  if (m && m[1].trim()) {
    return { main: m[1].trim(), note: m[2].trim() };
  }
  return { main: raw.trim(), note: null };
}

type QuotaStatus = "normal" | "extra" | "low" | "critical";

function resolveQuotaState(acc: AccountRecord): {
  status: QuotaStatus;
  label: string;
  extraDepleted: boolean;
} {
  if (acc.quotaDetail) {
    const d = acc.quotaDetail;
    const planUsed = d.planUsedPercent;
    const planVal = d.planValue || "";
    const planDepleted =
      (planUsed !== null && planUsed >= 100) ||
      planVal.includes("用尽") ||
      planVal.includes("DEPLETED");
    const planLow =
      !planDepleted &&
      ((planUsed !== null && planUsed >= 85) ||
        planVal.includes("偏低") ||
        planVal.includes("LOW"));
    const hasExtra = Boolean(
      d.extraLabel || d.extraValue || d.extraUsedPercent !== null
    );
    const extraUsed = d.extraUsedPercent;
    const extraVal = d.extraValue || "";
    const extraDepleted =
      hasExtra &&
      ((extraUsed !== null && extraUsed >= 100) ||
        extraVal.includes("用尽") ||
        extraVal.includes("耗尽") ||
        extraVal.includes("DEPLETED") ||
        /剩余\s*0(\s|个|口|$)/.test(extraVal));
    const extraLow =
      hasExtra &&
      !extraDepleted &&
      ((extraUsed !== null && extraUsed >= 85) ||
        extraVal.includes("偏低") ||
        extraVal.includes("LOW"));
    const extraIsCap = Boolean(d.extraLabel && /月|monthly/i.test(d.extraLabel));

    if (extraIsCap) {
      if (planDepleted) {
        return { status: "critical", label: "日满", extraDepleted };
      }
      if (extraDepleted) {
        return { status: "critical", label: "月满", extraDepleted };
      }
      if (planLow || extraLow) {
        return { status: "low", label: "偏低", extraDepleted };
      }
      return { status: "normal", label: "", extraDepleted };
    }

    if (planDepleted) {
      if (!hasExtra || extraDepleted) {
        return { status: "critical", label: "用尽", extraDepleted };
      }
      if (extraLow) {
        return { status: "low", label: "偏低", extraDepleted };
      }
      return { status: "extra", label: "用额外", extraDepleted };
    }
    if (planLow) {
      if (!hasExtra || extraDepleted) {
        return { status: "low", label: "偏低", extraDepleted };
      }
      return { status: "normal", label: "", extraDepleted };
    }
    return { status: "normal", label: "", extraDepleted };
  }

  if (acc.quotaPercent === 0 || Boolean(acc.quotaText?.includes("用尽"))) {
    return { status: "critical", label: "用尽", extraDepleted: false };
  }
  if (
    (typeof acc.quotaPercent === "number" && acc.quotaPercent <= 15) ||
    Boolean(acc.quotaText?.includes("偏低"))
  ) {
    return { status: "low", label: "偏低", extraDepleted: false };
  }
  return { status: "normal", label: "", extraDepleted: false };
}

function scoreAccountAvailability(
  acc: AccountRecord,
  pageError?: string | null
): {
  tier: number;
  score: number;
  available: boolean;
} {
  if (acc.authState === "logged_out" || pageError) {
    return { tier: 0, score: -1, available: false };
  }
  const q = resolveQuotaState(acc);
  if (q.status === "critical") {
    return { tier: 1, score: 0, available: false };
  }
  const d = acc.quotaDetail;
  if (d) {
    if (d.planUsedPercent === null && /无限|UNLIMITED/i.test(d.planValue)) {
      return { tier: 6, score: 100, available: true };
    }
    const extraIsCap = Boolean(d.extraLabel && /月|monthly/i.test(d.extraLabel));
    if (q.status === "extra") {
      const extraRem =
        typeof d.extraUsedPercent === "number" ? Math.max(0, 100 - d.extraUsedPercent) : 50;
      return { tier: 2, score: 200 + extraRem, available: true };
    }
    let rem =
      typeof d.planUsedPercent === "number"
        ? Math.max(0, 100 - d.planUsedPercent)
        : typeof acc.quotaPercent === "number"
          ? acc.quotaPercent
          : 50;
    if (extraIsCap && typeof d.extraUsedPercent === "number") {
      rem = Math.min(rem, Math.max(0, 100 - d.extraUsedPercent));
    }
    return { tier: q.status === "low" ? 3 : 5, score: rem, available: true };
  }
  if (typeof acc.quotaPercent === "number") {
    return { tier: q.status === "low" ? 3 : 5, score: acc.quotaPercent, available: true };
  }
  if (acc.authState === "authenticated") {
    return { tier: 4, score: 50, available: true };
  }
  return { tier: 2, score: 50, available: true };
}

function compareAccountAvailability(
  a: AccountRecord,
  b: AccountRecord,
  state?: ManagerSnapshot | null
): number {
  const sa = scoreAccountAvailability(a, state ? findAccountPageError(state, a) : null);
  const sb = scoreAccountAvailability(b, state ? findAccountPageError(state, b) : null);
  if (sb.tier !== sa.tier) return sb.tier - sa.tier;
  return sb.score - sa.score;
}

function formatShortResetText(raw: string | null): string {
  const clean = (raw ?? "").trim();
  if (!clean) return "";
  const m = clean.match(/^(?:每周限额将在|周限额重置时间[:：]?|将在)\s*(.+?)\s*$/);
  if (m && m[1]) {
    const inner = m[1].trim();
    return inner.endsWith("重置") ? inner : `${inner}重置`;
  }
  return clean;
}

function formatShortExtraText(
  detail: QuotaDetail,
  extraDepleted: boolean
): string | null {
  if (
    !detail.extraLabel &&
    !detail.extraValue &&
    detail.extraUsedPercent === null
  ) {
    return null;
  }
  const prefix = detail.extraLabel && /月|monthly/i.test(detail.extraLabel) ? "月" : "额外";
  if (extraDepleted) {
    return `${prefix} 100% · 已耗尽`;
  }
  const parsed = splitQuotaValue(detail.extraValue ?? "");
  const pctPart =
    typeof detail.extraUsedPercent === "number"
      ? `${detail.extraUsedPercent}%`
      : parsed.main.replace(/^已使用\s*/, "").trim();
  const notePart = parsed.note
    ? parsed.note
        .replace(/^剩余\s*/, "余 ")
        .replace(/个?口令$/, "")
        .replace(/\s*tokens?$/i, "")
        .trim()
    : "";
  if (pctPart && notePart) {
    return `${prefix} ${pctPart} · ${notePart}`;
  }
  if (notePart) {
    return `${prefix} · ${notePart}`;
  }
  if (pctPart) {
    return `${prefix} ${pctPart}`;
  }
  return detail.extraLabel ?? "额外额度";
}

type CookieExpiryTone = "normal" | "warn" | "critical" | "expired" | "session";

interface CookieExpiryDisplay {
  tone: CookieExpiryTone;
  shortText: string;
  detailText: string;
  tooltip: string;
}

function formatLocalDateTime(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function formatCookieExpiry(acc: AccountRecord): CookieExpiryDisplay | null {
  if (acc.authState === "logged_out") return null;

  if (typeof acc.cookieExpiresAtMs === "number") {
    const diffMs = acc.cookieExpiresAtMs - Date.now();
    const absStr = formatLocalDateTime(acc.cookieExpiresAtMs);
    if (diffMs <= 0) {
      return {
        tone: "expired",
        shortText: "Cookie 已过期",
        detailText: `Cookie 已于 ${absStr} 过期（需重新登录或刷新）`,
        tooltip: `Cookie 过期时间：${absStr}`,
      };
    }

    const days = Math.floor(diffMs / 86_400_000);
    const hours = Math.floor((diffMs % 86_400_000) / 3_600_000);
    const mins = Math.max(1, Math.floor((diffMs % 3_600_000) / 60_000));

    const shortText =
      days >= 365
        ? `Cookie 约 ${(days / 365).toFixed(1).replace(/\.0$/, "")} 年`
        : days >= 1
          ? `Cookie 剩 ${days} 天`
          : hours >= 1
            ? `Cookie 剩 ${hours} 小时`
            : `Cookie 剩 ${mins} 分钟`;

    const fullRem =
      days >= 1 && hours > 0
        ? `${days} 天 ${hours} 小时`
        : days >= 1
          ? `${days} 天`
          : hours >= 1
            ? `${hours} 小时 ${mins} 分钟`
            : `${mins} 分钟`;

    const tone: CookieExpiryTone =
      diffMs <= 24 * 3_600_000
        ? "critical"
        : diffMs <= 7 * 86_400_000
          ? "warn"
          : "normal";

    return {
      tone,
      shortText,
      detailText: `Cookie 有效期至 ${absStr}（剩余 ${fullRem}）`,
      tooltip: `Cookie 有效期至 ${absStr}`,
    };
  }

  if (acc.cookieSessionOnly) {
    return {
      tone: "session",
      shortText: "仅本次会话有效",
      detailText: "会话级 Cookie（关闭应用后可能需重新登录）",
      tooltip: "浏览器会话级 Cookie，未设置固定过期时间",
    };
  }

  return null;
}

function renderQuotaDetailPanel(
  detail: QuotaDetail,
  status: QuotaStatus,
  extraDepleted: boolean
): string {
  const shortVal =
    typeof detail.planUsedPercent === "number"
      ? `${detail.planUsedPercent}%`
      : splitQuotaValue(detail.planValue).main;
  const clamped =
    typeof detail.planUsedPercent === "number"
      ? Math.max(0, Math.min(100, detail.planUsedPercent))
      : null;
  const barHtml =
    clamped !== null
      ? `<div class="quota-bar-track ${status}"><div class="quota-bar-fill ${status} ${clamped > 0 ? "has-fill" : ""}" style="width:${clamped}%"></div></div>`
      : `<div class="quota-compact-spacer"></div>`;

  const rawReset = detail.planSubtext ?? "";
  const shortReset = formatShortResetText(rawReset);
  const shortExtra = formatShortExtraText(detail, extraDepleted);
  const fullExtraTitle = [
    detail.extraLabel,
    detail.extraValue,
    detail.extraSubtext,
  ]
    .filter(Boolean)
    .join(" · ");

  const metaHtml =
    shortReset || shortExtra
      ? `
        <div class="quota-compact-meta">
          <span class="quota-sub-text" title="${escapeHtml(rawReset)}">${escapeHtml(shortReset)}</span>
          ${shortExtra ? `<span class="quota-extra-pill ${status}" title="${escapeHtml(fullExtraTitle)}">${escapeHtml(shortExtra)}</span>` : ""}
        </div>
      `
      : "";

  return `
    <div class="quota-compact-main">
      <span class="quota-plan-label ${status}">${escapeHtml(detail.planLabel)}</span>
      ${barHtml}
      <span class="quota-val ${status}">${escapeHtml(shortVal)}</span>
    </div>
    ${metaHtml}
  `;
}

function saveScrollPosition(): void {
  if (!snapshot) return;
  const listEl = document.querySelector<HTMLElement>(".account-list");
  if (listEl) {
    if (viewMode === "groups") {
      lastGroupListScroll = listEl.scrollTop;
    } else {
      scrollOffsets[snapshot.selectedProviderId] = listEl.scrollTop;
    }
  }
  const msgEl = document.querySelector<HTMLElement>(".group-messages");
  if (msgEl && snapshot.activeGroupId) {
    groupScrolledToBottom = msgEl.scrollHeight - msgEl.scrollTop - msgEl.clientHeight <= 64;
    groupScrollOffsets[snapshot.activeGroupId] = msgEl.scrollTop;
  }
  const modalOverlayEl = document.querySelector<HTMLElement>(".inline-modal");
  if (modalOverlayEl) {
    modalOverlayScrollTop = modalOverlayEl.scrollTop;
  }
  const pickerEl = document.querySelector<HTMLElement>(".group-member-picker");
  if (pickerEl) {
    groupMemberPickerScrollTop = pickerEl.scrollTop;
  }
  const modalCardEl = document.querySelector<HTMLElement>(".modal-card");
  if (modalCardEl) {
    modalCardScrollTop = modalCardEl.scrollTop;
  }
}

function restoreScrollPosition(): void {
  if (!snapshot) return;
  const listEl = document.querySelector<HTMLElement>(".account-list");
  if (listEl) {
    listEl.scrollTop =
      viewMode === "groups"
        ? lastGroupListScroll
        : (scrollOffsets[snapshot.selectedProviderId] ?? 0);
  }
  const msgEl = document.querySelector<HTMLElement>(".group-messages");
  if (msgEl && snapshot.activeGroupId) {
    if (forceGroupScrollBottom || groupScrolledToBottom) {
      msgEl.scrollTop = msgEl.scrollHeight;
      forceGroupScrollBottom = false;
    } else if (snapshot.activeGroupId in groupScrollOffsets) {
      msgEl.scrollTop = groupScrollOffsets[snapshot.activeGroupId];
    } else {
      msgEl.scrollTop = msgEl.scrollHeight;
    }
  }
  if (modal.kind !== "none") {
    const modalOverlayEl = document.querySelector<HTMLElement>(".inline-modal");
    if (modalOverlayEl) {
      modalOverlayEl.scrollTop = modalOverlayScrollTop;
    }
    const pickerEl = document.querySelector<HTMLElement>(".group-member-picker");
    if (pickerEl) {
      pickerEl.scrollTop = groupMemberPickerScrollTop;
    }
    const modalCardEl = document.querySelector<HTMLElement>(".modal-card");
    if (modalCardEl) {
      modalCardEl.scrollTop = modalCardScrollTop;
    }
  }
}

function rerenderPreservingScroll(): void {
  saveScrollPosition();
  render();
  restoreScrollPosition();
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let idx = 0;
  let val = bytes;
  while (val >= 1024 && idx < units.length - 1) {
    val /= 1024;
    idx += 1;
  }
  return idx === 0 ? `${Math.round(val)} B` : `${val.toFixed(1)} ${units[idx]}`;
}

function formatProgressSummary(evt: UpdateProgressEvent): string {
  if (evt.stage === "verifying") return "校验中";
  if (evt.stage === "installing") return "正在替换";
  if (evt.stage === "ready") return "完成";
  if (evt.stage === "needs_permission") return "待授权";
  const pct = typeof evt.percent === "number" ? `${evt.percent}%` : "";
  const transferred = evt.total
    ? `${formatBytes(evt.downloaded)} / ${formatBytes(evt.total)}`
    : evt.downloaded > 0
      ? formatBytes(evt.downloaded)
      : "";
  if (pct && transferred) return `${pct} · ${transferred}`;
  return pct || transferred;
}

async function runCheckForUpdates(silent: boolean): Promise<void> {
  if (updateState.checking || updateState.installing) return;
  updateState.checking = true;
  if (!silent) {
    updateState.error = null;
    updateState.outcomeStatus = null;
    updateState.outcomeMessage = null;
    if (modal.kind === "update") {
      rerenderPreservingScroll();
    }
  }
  try {
    const res = await invoke<UpdateCheckResult>("check_for_updates");
    updateState.result = res;
    updateState.error = null;
    updateState.lastCheckedAt = Date.now();
    updateState.lastFailedAt = 0;
  } catch (err) {
    updateState.lastFailedAt = Date.now();
    if (!silent) {
      updateState.error = String(err);
    }
  } finally {
    updateState.checking = false;
    if (!silent || updateState.result?.hasUpdate || modal.kind === "update") {
      rerenderPreservingScroll();
    }
  }
}

async function maybeAutoCheckForUpdates(): Promise<void> {
  if (updateState.checking || updateState.installing) return;
  const now = Date.now();
  if (updateState.lastCheckedAt === 0) {
    if (updateState.lastFailedAt === 0 || now - updateState.lastFailedAt >= 2 * 60 * 1000) {
      await runCheckForUpdates(true);
    }
    return;
  }
  const interval = updateState.result?.hasUpdate ? 2 * 60 * 60 * 1000 : 30 * 60 * 1000;
  if (now - updateState.lastCheckedAt >= interval) {
    await runCheckForUpdates(true);
  }
}

async function runDownloadAndInstall(): Promise<void> {
  const res = updateState.result;
  if (!res?.downloadUrl || updateState.checking || updateState.installing) return;
  updateState.installing = true;
  updateState.error = null;
  updateState.outcomeStatus = null;
  updateState.outcomeMessage = null;
  lastRenderedUpdateStage = "downloading";
  updateState.progress = {
    stage: "downloading",
    downloaded: 0,
    total: res.size,
    percent: 0,
    message: "正在准备下载...",
  };
  rerenderPreservingScroll();
  try {
    const outcome = await invoke<InstallUpdateOutcome>("download_and_install_update", {
      request: {
        version: res.latestVersion,
        downloadUrl: res.downloadUrl,
        sha256: res.sha256,
        filename: res.filename,
      },
    });
    updateState.outcomeStatus = outcome.status;
    updateState.outcomeMessage = outcome.message;
  } catch (err) {
    updateState.error = String(err);
  } finally {
    updateState.installing = false;
    rerenderPreservingScroll();
  }
}

async function setModal(next: ModalMode): Promise<void> {
  const prevKind = modal.kind;
  const wasActive = prevKind !== "none";
  const isActive = next.kind !== "none";
  if (next.kind !== prevKind) {
    shouldAnimateModal = isActive;
    modalOverlayScrollTop = 0;
    groupMemberPickerScrollTop = 0;
    modalCardScrollTop = 0;
  }
  modal = next;
  if (wasActive !== isActive) {
    try {
      snapshot = await invoke<ManagerSnapshot>("set_modal_active", {
        active: isActive,
      });
    } catch (err) {
      if (snapshot) {
        snapshot = { ...snapshot, notice: String(err) };
      }
    }
  }
  render();
  if (next.kind === "add" || next.kind === "rename") {
    const input = document.querySelector<HTMLInputElement>("#modal-alias-input");
    if (input) {
      input.focus();
      input.select();
    }
  } else if (next.kind === "group") {
    const input = document.querySelector<HTMLInputElement>("#modal-group-name-input");
    if (input) {
      input.focus();
      input.select();
    }
  } else if (next.kind === "group-knowledge") {
    document.querySelector<HTMLTextAreaElement>("#modal-group-context-input")?.focus();
  } else if (next.kind === "settings") {
    document.querySelector<HTMLInputElement>("#modal-shortcut-input")?.focus();
  } else if (next.kind === "remove" || next.kind === "delete-group") {
    document.querySelector<HTMLButtonElement>("#modal-confirm-btn")?.focus();
  }
}

async function callCommand(
  cmd: string,
  args: Record<string, unknown> = {},
  optimisticTarget: AccountKey | null = null
): Promise<boolean> {
  if (busy) return false;
  busy = true;
  saveScrollPosition();
  if (optimisticTarget) {
    optimisticSwitching = optimisticTarget;
    if (snapshot) {
      snapshot = { ...snapshot, activeAccount: null };
    }
    render();
    restoreScrollPosition();
  }
  try {
    snapshot = await invoke<ManagerSnapshot>(cmd, args);
    syncViewModeFromSnapshot(snapshot);
    return true;
  } catch (err) {
    if (
      (cmd === "add_account" && modal.kind === "add") ||
      (cmd === "rename_account" && modal.kind === "rename") ||
      (cmd === "clear_account_data" && modal.kind === "rename") ||
      (cmd === "import_account_cookies" && modal.kind === "rename") ||
      ((cmd === "create_group_chat" || cmd === "update_group_chat") && modal.kind === "group") ||
      ((cmd === "update_group_knowledge" || cmd === "reset_group_thread_binding") &&
        modal.kind === "group-knowledge") ||
      ((cmd === "upsert_external_agent" || cmd === "delete_external_agent") &&
        modal.kind === "external-agents") ||
      (cmd === "update_app_settings" && modal.kind === "settings")
    ) {
      modal.error = String(err);
    } else if (snapshot) {
      snapshot = { ...snapshot, notice: String(err) };
    }
    return false;
  } finally {
    optimisticSwitching = null;
    busy = false;
    render();
    restoreScrollPosition();
  }
}

function currentProvider(state: ManagerSnapshot): ProviderInfo {
  return (
    state.providers.find((p) => p.id === state.selectedProviderId) ??
    state.providers[0]
  );
}

function renderUpdateModal(): string {
  const res = updateState.result;
  const curVer = res ? res.currentVersion : "...";
  const hasUpdate = Boolean(res?.hasUpdate);
  const title = hasUpdate
    ? `发现新版本 v${escapeHtml(res!.latestVersion)}`
    : "检查更新";

  const versionRow = res
    ? `
      <div class="update-version-row">
        <span class="update-version-pill">当前 v${escapeHtml(curVer)}</span>
        ${
          hasUpdate
            ? `<span>→</span><span class="update-version-pill latest">最新 v${escapeHtml(res.latestVersion)}</span>`
            : ""
        }
        ${res.size ? `<span class="update-size-label">${escapeHtml(formatBytes(res.size))}</span>` : ""}
      </div>
    `
    : `<div class="modal-desc">正在连接 GitHub Releases 检查版本信息...</div>`;

  let bodyHtml = "";
  if (updateState.checking && !res) {
    bodyHtml = `
      <div class="update-progress-wrap">
        <div class="update-progress-track"><div class="update-progress-fill indeterminate"></div></div>
      </div>
    `;
  } else if (res) {
    const noteText = res.releaseNotes.trim();
    if (hasUpdate) {
      bodyHtml += `<div class="update-notes">${escapeHtml(noteText || "包含稳定性改进与问题修复。")}</div>`;
      if (!res.downloadUrl) {
        bodyHtml += `<div class="modal-desc">当前发布尚未包含本平台自动安装包，请点击「发布页」手动下载。</div>`;
      }
    } else {
      bodyHtml += `<div class="modal-desc">${escapeHtml(noteText || "当前已是最新版本。")}</div>`;
    }
  }

  const prog = updateState.progress;
  const showProgress =
    prog &&
    prog.stage !== "error" &&
    (updateState.installing ||
      updateState.outcomeStatus === "restarting" ||
      prog.stage === "ready");
  if (showProgress) {
    lastRenderedUpdateStage = prog.stage;
    const isIndeterminate = prog.percent === null;
    const widthStyle = isIndeterminate ? "" : `style="width:${Math.max(0, Math.min(100, prog.percent!))}%"`;
    bodyHtml += `
      <div class="update-progress-wrap">
        <div class="update-progress-meta">
          <span id="update-progress-text">${escapeHtml(prog.message)}</span>
          <span id="update-progress-pct" class="update-progress-pct">${escapeHtml(formatProgressSummary(prog))}</span>
        </div>
        <div class="update-progress-track">
          <div id="update-progress-bar" class="update-progress-fill ${isIndeterminate ? "indeterminate" : ""}" ${widthStyle}></div>
        </div>
      </div>
    `;
  }

  if (updateState.outcomeMessage) {
    bodyHtml += `<div class="update-outcome-box">${escapeHtml(updateState.outcomeMessage)}</div>`;
  }
  if (updateState.error) {
    bodyHtml += `<div class="update-error-box">${escapeHtml(updateState.error)}</div>`;
  }

  const canInstall = hasUpdate && Boolean(res?.downloadUrl);
  const disablePrimary =
    updateState.checking ||
    updateState.installing ||
    updateState.outcomeStatus === "restarting";
  const primaryBtn = canInstall
    ? `<button id="update-install-btn" class="btn primary" type="button" ${disablePrimary ? "disabled" : ""}>${
        updateState.installing
          ? "更新中..."
          : updateState.outcomeStatus === "needs_permission"
            ? "已授权，继续安装"
            : snapshot?.platform === "ios"
              ? "前往下载 IPA"
              : "立即下载并更新"
      }</button>`
    : `<button id="update-recheck-btn" class="btn primary" type="button" ${disablePrimary ? "disabled" : ""}>${
        updateState.checking ? "检查中..." : "重新检查"
      }</button>`;

  const pageBtn = res?.releasePageUrl
    ? `<button id="update-open-page-btn" class="btn" type="button">发布页</button>`
    : "";

  return `
    <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
      <div class="modal-card">
        <div class="modal-title">${title}</div>
        ${versionRow}
        ${bodyHtml}
        <div class="modal-footer">
          <button id="modal-cancel-btn" class="btn" type="button" ${updateState.installing ? "disabled" : ""}>关闭</button>
          ${pageBtn}
          ${primaryBtn}
        </div>
      </div>
    </div>
  `;
}

function supportsAccountProxy(platform: ManagerSnapshot["platform"]): boolean {
  return platform === "macos" || platform === "windows" || platform === "ios";
}

function formatGroupPickerHint(
  eligibleCount: number,
  ineligibleCount: number,
  selectedCount: number,
  hasPickerContent: boolean
): string {
  if (eligibleCount === 0 && !hasPickerContent) {
    return "暂无可用成员，请先登录网页账号或添加 API / CLI 成员。";
  }
  if (ineligibleCount > 0) {
    return `已自动过滤 ${ineligibleCount} 个未登录或登录失效网页账号（已选 ${selectedCount}/30）。`;
  }
  return `支持网页账号与 API / CLI 混合协同（已选 ${selectedCount}/30）。`;
}

function renderModal(state: ManagerSnapshot, provider: ProviderInfo): string {
  const platform = state.platform;
  if (modal.kind === "none") return "";
  if (modal.kind === "update") {
    return renderUpdateModal();
  }
  if (modal.kind === "settings") {
    const saving = Boolean(modal.saving);
    const shortcutPresets = [
      { label: "默认", val: "CommandOrControl+Shift+M" },
      { label: "Alt+Space", val: "Alt+Space" },
      { label: "Cmd/Ctrl+Alt+A", val: "CommandOrControl+Alt+A" },
      { label: "禁用", val: "" },
    ];
    const statusText = state.globalShortcutActive
      ? "快捷键已生效"
      : state.globalShortcutError
        ? state.globalShortcutError
        : state.settings.globalShortcut
          ? "未激活"
          : "已禁用";
    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card">
          <div class="modal-title">桌面端设置</div>
          <label class="setting-row" for="modal-tray-checkbox">
            <span class="setting-label">关闭主窗口时最小化到系统托盘</span>
            <input id="modal-tray-checkbox" class="setting-checkbox" type="checkbox" ${modal.minimizeToTray ? "checked" : ""} ${saving ? "disabled" : ""} />
          </label>
          <label class="setting-row" for="modal-auto-approve-checkbox">
            <span class="setting-label">自动审批权限请求（前台与后台保活账号）</span>
            <input id="modal-auto-approve-checkbox" class="setting-checkbox" type="checkbox" ${modal.autoApprove ? "checked" : ""} ${saving ? "disabled" : ""} />
          </label>
          <div class="modal-hint">开启后，已预热保活的工具授权弹窗将自动点击允许，并跳过删除/退出/支付等高危操作。</div>
          <div class="setting-field">
            <div class="setting-field-head">
              <label class="modal-desc" for="modal-shortcut-input">全局呼出 / 隐藏快捷键</label>
              <span class="shortcut-status-pill ${state.globalShortcutActive ? "ok" : state.globalShortcutError ? "err" : ""}" title="${escapeHtml(statusText)}">${escapeHtml(state.globalShortcutActive ? "已生效" : state.globalShortcutError ? "注册失败" : "未启用")}</span>
            </div>
            <input id="modal-shortcut-input" class="modal-input" maxlength="80" placeholder="CommandOrControl+Shift+M（留空禁用）" value="${escapeHtml(modal.globalShortcut)}" autocomplete="off" spellcheck="false" ${saving ? "disabled" : ""} />
            <div class="preset-chips">
              ${shortcutPresets
                .map(
                  (p) =>
                    `<button class="preset-chip" data-shortcut-preset="${escapeHtml(p.val)}" type="button" ${saving ? "disabled" : ""}>${escapeHtml(p.label)}</button>`
                )
                .join("")}
            </div>
            ${state.globalShortcutError && !modal.error ? `<div class="modal-hint error-hint">${escapeHtml(state.globalShortcutError)}</div>` : `<div class="modal-hint">在任意应用中按下快捷键即可显示或隐藏主窗口；托盘菜单可完全退出。</div>`}
          </div>
          ${modal.error ? `<div id="modal-error-box" class="update-error-box" role="alert">${escapeHtml(modal.error)}</div>` : ""}
          <div class="modal-footer">
            <button id="modal-cancel-btn" class="btn" type="button" ${saving ? "disabled" : ""}>取消</button>
            <button id="modal-confirm-btn" class="btn primary" type="button" ${saving ? "disabled" : ""}>${saving ? "保存中..." : "保存"}</button>
          </div>
        </div>
      </div>
    `;
  }
  if (modal.kind === "add" || modal.kind === "rename") {
    const creating = modal.kind === "add";
    const saving = Boolean(modal.saving);
    const proxyTesting = Boolean(modal.proxyTesting);
    const supportsProxy = supportsAccountProxy(platform);
    const title = creating
      ? `新增 ${escapeHtml(provider.shortName)} 账号`
      : "账号设置";
    const aliasPlaceholder = creating
      ? "备注名称（留空则自动获取登录邮箱）"
      : "备注名称（留空保留原备注）";
    const testRes = modal.proxyTestResult;
    const renameAcc = modal.kind === "rename" ? modal.account : null;
    const currentAcc = renameAcc
      ? (state.accounts.find(
          (a) => a.providerId === renameAcc.providerId && a.uuid === renameAcc.uuid
        ) ?? renameAcc)
      : null;
    const cookieInfo = currentAcc ? formatCookieExpiry(currentAcc) : null;
    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card">
          <div class="modal-title">${title}</div>
          <input id="modal-alias-input" class="modal-input" maxlength="60" placeholder="${aliasPlaceholder}" aria-label="备注名称" value="${escapeHtml(modal.draft)}" ${saving ? "disabled" : ""} />
          ${supportsProxy ? `
            <label class="modal-desc" for="modal-proxy-input">HTTP CONNECT 代理（留空使用系统网络）</label>
            <div class="proxy-input-row">
              <input id="modal-proxy-input" class="modal-input" maxlength="512" placeholder="http://127.0.0.1:7890" value="${escapeHtml(modal.proxyDraft)}" autocomplete="off" spellcheck="false" ${saving ? "disabled" : ""} />
              ${!isMobilePlatform(platform) ? `<button id="proxy-test-btn" class="btn mini-test-btn" type="button" ${saving || proxyTesting ? "disabled" : ""}>${proxyTesting ? "测速中..." : "测试连接"}</button>` : ""}
            </div>
            ${testRes ? `<div id="proxy-test-result" class="proxy-test-box ${testRes.ok ? "ok" : "err"}">${escapeHtml(testRes.message)}</div>` : ""}
            <div class="modal-hint">仅作用于内嵌页面；外部浏览器登录与更新检查不走此代理。</div>
          ` : ""}
          <label class="modal-desc" for="modal-timezone-input">自定义时区（留空跟随系统）</label>
          <input id="modal-timezone-input" class="modal-input" maxlength="64" placeholder="IANA 时区（例如 America/New_York、Asia/Tokyo）" value="${escapeHtml(modal.timezoneDraft)}" autocomplete="off" spellcheck="false" ${saving ? "disabled" : ""} />
          <div class="preset-chips">
            ${TIMEZONE_PRESETS.map(
              (p) =>
                `<button class="preset-chip" data-timezone-preset="${escapeHtml(p.value)}" type="button" ${saving ? "disabled" : ""}>${escapeHtml(p.label)}</button>`
            ).join("")}
          </div>
          <div class="modal-hint">${platform === "windows" ? "通过 WebView2 原生接口按账号独立生效，留空则使用系统时区。" : "文档加载前注入该账号独立页面，支持夏令时，留空则使用系统时区。"}</div>
          ${cookieInfo ? `<div class="cookie-detail-box ${cookieInfo.tone}" title="${escapeHtml(cookieInfo.tooltip)}"><span class="cookie-expiry-dot"></span><span>${escapeHtml(cookieInfo.detailText)}</span></div>` : ""}
          <div class="cookie-import-section">
            <label class="modal-desc" for="modal-cookie-import-input">${creating ? "导入 Cookie（可选，支持从 Chrome 等浏览器登录后粘贴）" : "导入 Cookie（支持从 Chrome 等浏览器登录后粘贴）"}</label>
            <textarea id="modal-cookie-import-input" class="modal-textarea compact-cookie-textarea" rows="3" placeholder="粘贴 Cookie 请求头（a=1; b=2）、Netscape cookies.txt 或浏览器导出的 JSON 数组..." ${saving ? "disabled" : ""}>${escapeHtml(modal.cookieDraft ?? "")}</textarea>
            ${creating ? `
              <div class="modal-hint">留空则创建后手动登录；粘贴 Cookie 后点击「创建并打开」将直接导入并登录。</div>
            ` : `
              <div class="cookie-import-actions">
                <span class="modal-hint">适用于 Google OAuth 无法在内置窗口直接登录的场景；导入后自动刷新当前账号。</span>
                <button id="modal-import-cookie-btn" class="btn mini-test-btn" type="button" ${saving || modal.cookieImporting ? "disabled" : ""}>${modal.cookieImporting ? "导入中..." : "导入 Cookie"}</button>
              </div>
            `}
          </div>
          ${modal.error ? `<div id="modal-error-box" class="update-error-box" role="alert">${escapeHtml(modal.error)}</div>` : ""}
          <div class="modal-footer">
            ${!creating ? `<button id="modal-clear-data-btn" class="btn danger-outline left-action" type="button" ${saving ? "disabled" : ""}>${modal.confirmClear ? "确认清空？" : "清空登录缓存"}</button>` : ""}
            <button id="modal-cancel-btn" class="btn" type="button" ${saving ? "disabled" : ""}>取消</button>
            <button id="modal-confirm-btn" class="btn primary" type="button" ${saving ? "disabled" : ""}>${saving ? (creating ? "创建中..." : "保存中...") : (creating ? "创建并打开" : "保存")}</button>
          </div>
        </div>
      </div>
    `;
  }
  if (modal.kind === "group") {
    const gModal = modal;
    const editing = Boolean(gModal.group);
    const saving = Boolean(gModal.saving);
    const extAgents = state.externalAgents ?? [];
    const eligibleCount =
      state.accounts.filter((a) => isAccountGroupEligible(state, a)).length + extAgents.length;
    const ineligibleCount = state.accounts.length + extAgents.length - eligibleCount;

    const providerGroupsHtml = state.providers
      .filter((p) => p.enabled)
      .map((p) => {
        const accs = state.accounts.filter((a) => {
          if (a.providerId !== p.id) return false;
          if (isAccountGroupEligible(state, a)) return true;
          return Boolean(
            editing &&
              a.authState !== "logged_out" &&
              gModal.selectedMembers.some((m) => m.providerId === a.providerId && m.uuid === a.uuid)
          );
        });
        if (accs.length === 0) return "";
        return `
          <div class="group-picker-section">
            <div class="group-picker-provider">${renderProviderLogo(p, "group-picker-plogo")}<span>${escapeHtml(p.shortName)}</span></div>
            <div class="group-member-list">
              ${accs
                .map((acc) => {
                  const checked = gModal.selectedMembers.some(
                    (m) => m.providerId === acc.providerId && m.uuid === acc.uuid
                  );
                  const cookieHint = formatCookieExpiry(acc);
                  return `
                    <button
                      class="group-member-option ${checked ? "selected" : ""}"
                      data-toggle-group-member="1"
                      data-member-provider="${escapeHtml(acc.providerId)}"
                      data-member-uuid="${escapeHtml(acc.uuid)}"
                      type="button"
                      ${saving ? "disabled" : ""}
                    >
                      <span class="group-check-box">${checked ? "✓" : ""}</span>
                      ${renderMemberAvatarNode(state, acc, acc.alias, "mini")}
                      <span class="group-option-meta">
                        <span class="group-option-name">${escapeHtml(acc.alias)}</span>
                        ${cookieHint ? `<span class="group-option-sub">${escapeHtml(cookieHint.shortText)}</span>` : ""}
                      </span>
                    </button>
                  `;
                })
                .join("")}
            </div>
          </div>
        `;
      })
      .filter(Boolean)
      .join("");

    const extKindGroups: Array<{ kind: ExternalAgentKind; title: string; pid: "api" | "cli" }> = [
      { kind: "openai_api", title: "OpenAI API", pid: "api" },
      { kind: "local_cli", title: "本地 CLI", pid: "cli" },
    ];
    const externalGroupHtml = extKindGroups
      .map(({ kind, title, pid }) => {
        const list = extAgents.filter((e) => e.kind === kind);
        if (list.length === 0) return "";
        return `
          <div class="group-picker-section">
            <div class="group-picker-provider"><span>${escapeHtml(title)}</span><span class="group-provider-tag">直连</span></div>
            <div class="group-member-list">
              ${list
                .map((ext) => {
                  const checked = gModal.selectedMembers.some(
                    (m) => m.providerId === pid && m.uuid === ext.id
                  );
                  const sub =
                    ext.kind === "openai_api"
                      ? ext.model
                      : ext.command;
                  return `
                    <button
                      class="group-member-option ${checked ? "selected" : ""}"
                      data-toggle-group-member="1"
                      data-member-provider="${pid}"
                      data-member-uuid="${escapeHtml(ext.id)}"
                      type="button"
                      ${saving ? "disabled" : ""}
                    >
                      <span class="group-check-box">${checked ? "✓" : ""}</span>
                      ${renderMemberAvatarNode(state, undefined, ext.name, "mini", ext.kind)}
                      <span class="group-option-meta">
                        <span class="group-option-name">${escapeHtml(ext.name)}</span>
                        <span class="group-option-sub">${escapeHtml(sub)}</span>
                      </span>
                    </button>
                  `;
                })
                .join("")}
            </div>
          </div>
        `;
      })
      .filter(Boolean)
      .join("");

    const pickerContent = `${providerGroupsHtml}${externalGroupHtml}`;
    const hintText = formatGroupPickerHint(
      eligibleCount,
      ineligibleCount,
      gModal.selectedMembers.length,
      Boolean(pickerContent)
    );

    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card group-modal-card">
          <div class="modal-title">${editing ? `编辑「${escapeHtml(gModal.group!.name)}」` : "创建群聊"}</div>
          <input id="modal-group-name-input" class="modal-input" maxlength="32" placeholder="群聊名称（如：产品头脑风暴）" aria-label="群聊名称" value="${escapeHtml(gModal.nameDraft)}" ${saving ? "disabled" : ""} />
          <div class="group-picker-head-row">
            <div class="modal-desc">选择参与群聊的网页账号或 API / CLI 成员</div>
            <button id="open-external-agents-from-group-btn" class="mini-btn primary-text" type="button" ${saving ? "disabled" : ""}>+ 管理 API/CLI</button>
          </div>
          <div class="group-member-picker">
            ${pickerContent || `<div class="group-picker-empty">当前没有可用成员，可登录网页或点击右上角添加 API / CLI</div>`}
          </div>
          <div id="modal-group-hint" class="modal-hint">${escapeHtml(hintText)}</div>
          ${gModal.error ? `<div id="modal-error-box" class="update-error-box" role="alert">${escapeHtml(gModal.error)}</div>` : ""}
          <div class="modal-footer">
            <button id="modal-cancel-btn" class="btn" type="button" ${saving ? "disabled" : ""}>取消</button>
            <button id="modal-confirm-btn" class="btn primary" type="button" ${saving || gModal.selectedMembers.length === 0 ? "disabled" : ""}>${saving ? (editing ? "保存中..." : "创建中...") : (editing ? "保存" : "创建群聊")}</button>
          </div>
        </div>
      </div>
    `;
  }
  if (modal.kind === "group-knowledge") {
    const kModal = modal;
    const saving = Boolean(kModal.saving);
    const enabledCount = kModal.skillsDraft.filter((s) => s.enabled).length;
    const latestGroup = state.groupChats.find((g) => g.id === kModal.group.id) ?? kModal.group;

    let tabBody = "";
    if (kModal.tab === "context") {
      tabBody = `
        <div class="modal-desc">所有被 @ 唤醒的成员在首次进入或配置变更后自动增量同步此上下文，API / CLI 成员每轮自动携带。</div>
        <textarea
          id="modal-group-context-input"
          class="modal-textarea"
          rows="8"
          maxlength="8000"
          placeholder="例如：项目采用 Tauri + Rust + TypeScript；核心约束是低内存；当前正在排查…"
          ${saving ? "disabled" : ""}
        >${escapeHtml(kModal.contextDraft)}</textarea>
        <div class="modal-hint"><span id="modal-group-context-count">${kModal.contextDraft.length}</span>/8000 字符 · 也可点消息「沉淀上下文」或顶部「沉淀本轮结论」</div>
      `;
    } else if (kModal.tab === "skills") {
      const presetsHtml = GROUP_SKILL_PRESETS.map((p, idx) => {
        const added = kModal.skillsDraft.some((s) => s.name === p.name);
        return `<button class="skill-preset-chip" data-add-skill-preset="${idx}" type="button" ${saving || added ? "disabled" : ""}>${added ? `✓ ${escapeHtml(p.name)}` : `+ ${escapeHtml(p.name)}`}</button>`;
      }).join("");

      const skillsHtml =
        kModal.skillsDraft.length === 0
          ? `<div class="group-picker-empty">暂未添加共享技能，可点击上方预设或下方自定义添加。</div>`
          : kModal.skillsDraft
              .map(
                (s) => `
              <div class="skill-card ${s.enabled ? "" : "disabled"}">
                <div class="skill-card-head">
                  <label class="skill-toggle-label">
                    <input type="checkbox" data-toggle-skill-id="${escapeHtml(s.id)}" ${s.enabled ? "checked" : ""} ${saving ? "disabled" : ""} />
                    <span class="skill-card-name">${escapeHtml(s.name)}</span>
                  </label>
                  <div class="skill-card-ops">
                    <button class="mini-btn" data-edit-skill-id="${escapeHtml(s.id)}" type="button" ${saving ? "disabled" : ""}>编辑</button>
                    <button class="mini-btn danger-text" data-delete-skill-id="${escapeHtml(s.id)}" type="button" ${saving ? "disabled" : ""}>删除</button>
                  </div>
                </div>
                <div class="skill-card-desc">${escapeHtml(s.description || s.content)}</div>
              </div>
            `
              )
              .join("");

      tabBody = `
        <div class="skill-presets-row">
          <span class="skill-presets-label">预设：</span>
          ${presetsHtml}
        </div>
        <div class="skills-list">${skillsHtml}</div>
        <div class="skill-editor-box">
          <div class="skill-editor-title">${kModal.editingSkillId ? "编辑共享技能" : "添加自定义技能"}</div>
          <input id="modal-skill-name-input" class="modal-input" maxlength="48" placeholder="技能名称（如：代码审查规范）" value="${escapeHtml(kModal.skillNameDraft)}" ${saving ? "disabled" : ""} />
          <input id="modal-skill-desc-input" class="modal-input" maxlength="120" placeholder="简短说明（选填）" value="${escapeHtml(kModal.skillDescDraft)}" ${saving ? "disabled" : ""} />
          <textarea id="modal-skill-content-input" class="modal-textarea" rows="3" maxlength="2000" placeholder="输入注入给各 AI 成员的技能指令原文…" ${saving ? "disabled" : ""}>${escapeHtml(kModal.skillContentDraft)}</textarea>
          <div class="skill-editor-actions">
            ${kModal.editingSkillId ? `<button id="modal-skill-cancel-edit-btn" class="mini-btn" type="button" ${saving ? "disabled" : ""}>取消编辑</button>` : ""}
            <button id="modal-skill-save-btn" class="mini-btn primary-text" type="button" ${saving ? "disabled" : ""}>${kModal.editingSkillId ? "更新技能" : "+ 添加技能"}</button>
          </div>
        </div>
      `;
    } else if (kModal.tab === "roles") {
      const roleRowsHtml = latestGroup.members
        .map((m) => {
          const meta = resolveMemberMeta(state, m);
          const curRole =
            kModal.rolesDraft.find((r) => sameAccountKey(r.member, m))?.role ?? "";
          const binding = !meta.isExternal
            ? (latestGroup.threadBindings ?? []).find((tb) => sameAccountKey(tb.member, m))
            : undefined;
          let boundShort = "";
          if (binding) {
            try {
              const u = new URL(binding.threadUrl);
              boundShort = `${u.hostname}${u.pathname}`;
            } catch {
              boundShort = binding.threadUrl;
            }
          }
          const bindingHtml = meta.isExternal
            ? `<div class="role-thread-row"><span class="thread-stateless-tag">无状态直连（每轮携带完整上下文）</span></div>`
            : binding
              ? `
                <div class="role-thread-row">
                  <span class="thread-bound-pill" title="${escapeHtml(binding.threadUrl)}">已绑定专属会话 · ${escapeHtml(boundShort)}</span>
                  <button class="mini-btn" data-reset-thread-provider="${escapeHtml(m.providerId)}" data-reset-thread-uuid="${escapeHtml(m.uuid)}" type="button" ${saving ? "disabled" : ""}>重置会话</button>
                </div>
              `
              : `<div class="role-thread-row"><span class="thread-unbound-tag">首次发消息时自动创建专属会话（不污染日常聊天）</span></div>`;

          return `
            <div class="role-row">
              <div class="role-member-meta">
                ${renderMemberAvatarNode(state, meta.acc, meta.alias, "mini", meta.ext?.kind)}
                <span class="role-member-name">${escapeHtml(meta.alias)}</span>
                <span class="group-provider-tag">${escapeHtml(meta.providerShort)}</span>
              </div>
              <input
                class="modal-input role-input"
                maxlength="120"
                data-role-provider="${escapeHtml(m.providerId)}"
                data-role-uuid="${escapeHtml(m.uuid)}"
                placeholder="成员分工（如：前端性能优化与交互评审）"
                value="${escapeHtml(curRole)}"
                ${saving ? "disabled" : ""}
              />
              ${bindingHtml}
            </div>
          `;
        })
        .join("");

      tabBody = `
        <div class="modal-desc">配置各成员专属分工指令，并可查看/重置网页成员绑定的群聊独立会话 URL。</div>
        <div class="role-list">${roleRowsHtml || `<div class="group-picker-empty">暂无成员</div>`}</div>
      `;
    } else {
      const snap = kModal.adaptersSnap;
      const chipsHtml = (snap?.adapters ?? [])
        .map(
          (a) =>
            `<span class="adapter-chip" title="${escapeHtml(a.hostPatterns.join(", "))}">${escapeHtml(a.name)} (${escapeHtml(a.id)})</span>`
        )
        .join("");
      tabBody = `
        <div class="modal-desc">声明式网站适配器（内置 Muse / Hark 规则，可按需热覆盖选择器）与 Fetch/SSE 只读挂钩协同工作。</div>
        <div class="adapter-chip-row">${chipsHtml || `<span class="modal-hint">正在读取规则…</span>`}</div>
        <div class="modal-hint">覆盖规则文件（保存即热生效，无需重编译）：<code>${escapeHtml(snap?.overridePath ?? "site_adapters.json")}</code></div>
        <textarea
          id="modal-adapters-json-input"
          class="modal-textarea code"
          rows="7"
          placeholder='可选：输入 JSON 数组覆盖内置规则，例如 [{"id":"muse","name":"Muse","hostPatterns":["muse.ai"],"composerSelectors":["textarea"]}]'
          ${saving ? "disabled" : ""}
        >${escapeHtml(kModal.adaptersJsonDraft)}</textarea>
        <div class="skill-editor-actions">
          <button id="modal-adapters-reset-btn" class="mini-btn" type="button" ${saving || !snap?.hasOverrides ? "disabled" : ""}>恢复内置默认</button>
        </div>
      `;
    }

    const confirmLabel =
      kModal.tab === "adapters"
        ? saving ? "保存中..." : "保存并热生效"
        : saving ? "保存中..." : "保存配置";

    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card group-knowledge-modal-card">
          <div class="modal-title">共享上下文 & 技能 · ${escapeHtml(latestGroup.name)}</div>
          <div class="knowledge-tabs">
            <button class="knowledge-tab ${kModal.tab === "context" ? "active" : ""}" data-knowledge-tab="context" type="button" ${saving ? "disabled" : ""}>共享上下文</button>
            <button class="knowledge-tab ${kModal.tab === "skills" ? "active" : ""}" data-knowledge-tab="skills" type="button" ${saving ? "disabled" : ""}>共享技能 (${enabledCount})</button>
            <button class="knowledge-tab ${kModal.tab === "roles" ? "active" : ""}" data-knowledge-tab="roles" type="button" ${saving ? "disabled" : ""}>分工 &amp; 会话</button>
            <button class="knowledge-tab ${kModal.tab === "adapters" ? "active" : ""}" data-knowledge-tab="adapters" type="button" ${saving ? "disabled" : ""}>站点规则${kModal.adaptersSnap?.hasOverrides ? " *" : ""}</button>
          </div>
          ${tabBody}
          ${kModal.error ? `<div id="modal-error-box" class="update-error-box" role="alert">${escapeHtml(kModal.error)}</div>` : ""}
          <div class="modal-footer">
            <button id="modal-cancel-btn" class="btn" type="button" ${saving ? "disabled" : ""}>取消</button>
            <button id="modal-confirm-btn" class="btn primary" type="button" ${saving ? "disabled" : ""}>${confirmLabel}</button>
          </div>
        </div>
      </div>
    `;
  }
  if (modal.kind === "external-agents") {
    const eModal = modal;
    const saving = Boolean(eModal.saving);
    const agents = state.externalAgents ?? [];
    const presetsHtml = EXTERNAL_AGENT_PRESETS.map(
      (p, idx) =>
        `<button class="skill-preset-chip" data-ext-preset-idx="${idx}" type="button" ${saving ? "disabled" : ""}>+ ${escapeHtml(p.label)}</button>`
    ).join("");
    const listHtml =
      agents.length === 0
        ? `<div class="group-picker-empty">暂未配置 API 或本地 CLI 成员，可点击下方预设或手动添加。</div>`
        : agents
            .map((ext) => {
              const desc =
                ext.kind === "openai_api"
                  ? `${ext.model} · ${ext.baseUrl}`
                  : `${ext.command} ${ext.args.join(" ")}`.trim();
              return `
                <div class="ext-agent-card">
                  <div class="ext-agent-main">
                    ${renderMemberAvatarNode(state, undefined, ext.name, "mini", ext.kind)}
                    <div class="ext-agent-meta">
                      <div class="ext-agent-name">${escapeHtml(ext.name)}</div>
                      <div class="ext-agent-sub">${escapeHtml(desc)}</div>
                    </div>
                  </div>
                  <div class="skill-card-ops">
                    <button class="mini-btn" data-edit-ext-id="${escapeHtml(ext.id)}" type="button" ${saving ? "disabled" : ""}>编辑</button>
                    <button class="mini-btn danger-text" data-delete-ext-id="${escapeHtml(ext.id)}" type="button" ${saving ? "disabled" : ""}>删除</button>
                  </div>
                </div>
              `;
            })
            .join("");

    const fieldsHtml =
      eModal.formKind === "openai_api"
        ? `
          <input id="ext-agent-url-input" class="modal-input" maxlength="260" placeholder="Base URL（例如 https://api.deepseek.com/v1 或 http://127.0.0.1:11434/v1）" value="${escapeHtml(eModal.baseUrlDraft)}" ${saving ? "disabled" : ""} />
          <input id="ext-agent-key-input" class="modal-input" maxlength="512" type="password" placeholder="API Key（本机 Ollama 可留空）" value="${escapeHtml(eModal.apiKeyDraft)}" ${saving ? "disabled" : ""} />
          <input id="ext-agent-model-input" class="modal-input" maxlength="120" placeholder="模型名称（例如 gpt-4o / deepseek-chat / qwen2.5:14b）" value="${escapeHtml(eModal.modelDraft)}" ${saving ? "disabled" : ""} />
        `
        : `
          <input id="ext-agent-cmd-input" class="modal-input" maxlength="260" placeholder="CLI 命令（例如 claude / codex / gemini）" value="${escapeHtml(eModal.commandDraft)}" ${saving ? "disabled" : ""} />
          <input id="ext-agent-args-input" class="modal-input" maxlength="400" placeholder="命令参数，空格分隔（例如 -p 或 exec {prompt}；无 {prompt} 时走 stdin）" value="${escapeHtml(eModal.argsDraft)}" ${saving ? "disabled" : ""} />
          <input id="ext-agent-workdir-input" class="modal-input" maxlength="260" placeholder="工作目录（选填，例如 /Users/name/project）" value="${escapeHtml(eModal.workdirDraft)}" ${saving ? "disabled" : ""} />
        `;

    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card group-knowledge-modal-card">
          <div class="modal-title">API / 本地 CLI 混合成员</div>
          <div class="modal-desc">配置直连 OpenAI 兼容接口或本地 CLI（如 Claude Code / Codex），与网页端账号混编入同一个群聊。</div>
          <div class="ext-agent-list">${listHtml}</div>
          <div class="skill-presets-row">
            <span class="skill-presets-label">快速填充：</span>
            ${presetsHtml}
          </div>
          <div class="skill-editor-box">
            <div class="ext-form-head">
              <span class="skill-editor-title">${eModal.editingId ? "编辑成员" : "新增直连成员"}</span>
              <div class="ext-kind-switch">
                <button class="mini-btn ${eModal.formKind === "openai_api" ? "primary-text" : ""}" data-ext-kind="openai_api" type="button" ${saving ? "disabled" : ""}>OpenAI API</button>
                <button class="mini-btn ${eModal.formKind === "local_cli" ? "primary-text" : ""}" data-ext-kind="local_cli" type="button" ${saving ? "disabled" : ""}>本地 CLI</button>
              </div>
            </div>
            <input id="ext-agent-name-input" class="modal-input" maxlength="40" placeholder="群聊显示名称（如：Claude-Code / DeepSeek）" value="${escapeHtml(eModal.nameDraft)}" ${saving ? "disabled" : ""} />
            ${fieldsHtml}
            <div class="skill-editor-actions">
              ${eModal.editingId ? `<button id="ext-agent-cancel-edit-btn" class="mini-btn" type="button" ${saving ? "disabled" : ""}>取消编辑</button>` : ""}
              <button id="ext-agent-save-btn" class="mini-btn primary-text" type="button" ${saving ? "disabled" : ""}>${saving ? "保存中..." : eModal.editingId ? "更新成员" : "+ 保存成员"}</button>
            </div>
          </div>
          ${eModal.error ? `<div id="modal-error-box" class="update-error-box" role="alert">${escapeHtml(eModal.error)}</div>` : ""}
          <div class="modal-footer">
            <button id="modal-cancel-btn" class="btn primary" type="button" ${saving ? "disabled" : ""}>完成</button>
          </div>
        </div>
      </div>
    `;
  }
  if (modal.kind === "delete-group") {
    return `
      <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
        <div class="modal-card">
          <div class="modal-title">删除群聊「${escapeHtml(modal.group.name)}」</div>
          <div class="modal-desc">将删除该群聊及本地群聊记录。各成员账号与原生网页历史不受影响。</div>
          <div class="modal-footer">
            <button id="modal-cancel-btn" class="btn" type="button">取消</button>
            <button id="modal-confirm-btn" class="btn danger" type="button">确认删除</button>
          </div>
        </div>
      </div>
    `;
  }
  return `
    <div class="${modalOverlayClass()}" role="dialog" aria-modal="true">
      <div class="modal-card">
        <div class="modal-title">移除「${escapeHtml(modal.account.alias)}」</div>
        <div class="modal-desc">将关闭该视图并彻底清除本机独立浏览资料（Cookie 与存储）。云端账号及历史对话仍由官方保留。</div>
        <div class="modal-footer">
          <button id="modal-cancel-btn" class="btn" type="button">取消</button>
          <button id="modal-confirm-btn" class="btn danger" type="button">确认移除</button>
        </div>
      </div>
    </div>
  `;
}

function renderMiddleContent(state: ManagerSnapshot, provider: ProviderInfo): string {
  const providerAccounts = state.accounts.filter((a) => a.providerId === provider.id);
  const switchingKey = effectiveSwitchingKey(state);
  const isMobile = isMobilePlatform(state.platform);
  const supportsProxy = supportsAccountProxy(state.platform);
  const supportsCookieCopy = supportsProxy || isMobile;
  const canReorder = providerAccounts.length > 1 && state.conflicts.length === 0;

  const noticeHtml = state.notice
    ? `
      <div class="notice-bar">
        <span>${escapeHtml(state.notice)}</span>
        <button id="dismiss-notice-btn" class="notice-close" type="button" title="关闭提示">✕</button>
      </div>
    `
    : "";

  const conflictsHtml = state.conflicts
    .map(
      (c) => `
      <div class="conflict-box">
        <div>检测到 UUID 归属冲突（${escapeHtml(c.uuid.slice(0, 8))}…），请选择保留归属服务：</div>
        <div class="conflict-actions">
          ${c.providers
            .map(
              (pid) =>
                `<button class="mini-btn" data-resolve-uuid="${escapeHtml(c.uuid)}" data-resolve-provider="${escapeHtml(pid)}" type="button">归属 ${escapeHtml(pid.toUpperCase())}</button>`
            )
            .join("")}
        </div>
      </div>
    `
    )
    .join("");

  const hasUpdate = Boolean(updateState.result?.hasUpdate);
  const mobileThemeBtn = isMobile
    ? `<button id="theme-toggle-btn" class="ghost-icon-btn" type="button" title="主题：${themeLabel(themeMode)}（点击切换）">${themeIcon(themeMode)}</button>`
    : "";
  const mobileUpdateBtn = isMobile
    ? `<button id="check-update-btn" class="ghost-icon-btn update-trigger-btn ${hasUpdate ? "has-update" : ""}" type="button" title="${hasUpdate ? `发现新版本 v${escapeHtml(updateState.result!.latestVersion)}` : "检查更新"}">${ICON_UPDATE}${hasUpdate ? `<span class="update-dot"></span>` : ""}</button>`
    : "";

  const headerBtnHtml = provider.enabled
    ? `
      ${mobileThemeBtn}
      ${mobileUpdateBtn}
      ${!isMobile ? `<button id="sync-all-btn" class="ghost-icon-btn" type="button" title="刷新已就绪账号额度">${ICON_REFRESH}</button>` : ""}
      <button id="open-site-btn" class="ghost-icon-btn" type="button" title="在浏览器中打开 ${escapeHtml(provider.shortName)} 官网">${ICON_EXTERNAL}</button>
      <button id="add-account-btn" class="icon-btn primary" type="button">+ 新增</button>
    `
    : `${mobileThemeBtn}${mobileUpdateBtn}<button id="open-site-btn" class="icon-btn primary" type="button">打开官网 ${ICON_EXTERNAL}</button>`;

  const listToolbarHtml =
    provider.enabled && providerAccounts.length >= 2 && state.conflicts.length === 0
      ? `
        <div class="list-toolbar">
          <button id="smart-switch-btn" class="toolbar-chip" type="button" title="切换到剩余可用额度最多的账号">
            ${ICON_BOLT}<span>切到余量最多</span>
          </button>
          <button id="sort-quota-btn" class="toolbar-chip" type="button" title="按剩余可用额度从高到低排序">
            ${ICON_SORT}<span>按额度排序</span>
          </button>
        </div>
      `
      : "";

  let listBody = "";
  if (!provider.enabled) {
    listBody = `
      <div class="empty-list">
        <div class="empty-icon-circle">${renderProviderLogo(provider)}</div>
        <div><strong>${escapeHtml(provider.name)}</strong> 暂未开放内嵌会话</div>
        <div>待核验官方网页直登路径与嵌入条款后启用，可点击右上角访问官网。</div>
      </div>
    `;
  } else if (providerAccounts.length === 0) {
    listBody = `
      <div class="empty-list">
        <div class="empty-icon-circle">${ICON_USER_PLUS}</div>
        <div>暂无 ${escapeHtml(provider.shortName)} 账号</div>
        <div>点击右上角「+ 新增」创建独立登录环境</div>
      </div>
    `;
  } else {
    listBody = providerAccounts
      .map((acc, idx) => {
        const switching = isSameKey(switchingKey, acc);
        const active = !switching && isSameKey(state.activeAccount, acc);
        const avatarText = computeAvatarText(acc);
        const isLoggedOut = acc.authState === "logged_out";
        const pageError = isLoggedOut ? null : findAccountPageError(state, acc);
        const hasPageError = Boolean(pageError);
        const hasDetail = !isLoggedOut && !hasPageError && Boolean(acc.quotaDetail);
        const cookieInfo = isLoggedOut || hasPageError ? null : formatCookieExpiry(acc);

        const quotaState =
          isLoggedOut || hasPageError
            ? { status: "normal" as QuotaStatus, label: "", extraDepleted: false }
            : resolveQuotaState(acc);
        const {
          status: alertStatus,
          label: alertLabel,
          extraDepleted,
        } = quotaState;

        const isMac = navigator.userAgent.includes("Mac");
        const shortcutHint =
          !isMobile && idx < 9
            ? `<span class="shortcut-pill" title="快捷切号">${isMac ? "⌘" : "^"}${idx + 1}</span>`
            : "";
        const proxyBadgeHtml =
          supportsProxy && acc.proxyUrl
            ? `<span class="proxy-pill" title="代理：${escapeHtml(acc.proxyUrl)}">代理</span>`
            : "";
        const timezoneBadgeHtml = acc.timezone
          ? `<span class="proxy-pill" title="时区：${escapeHtml(acc.timezone)}">${escapeHtml(acc.timezone)}</span>`
          : "";
        const alertBadgeHtml = switching
          ? `<span class="loading-status-badge">加载中…</span>`
          : isLoggedOut
            ? `<span class="auth-status-badge">未登录</span>`
            : hasPageError
              ? `<span class="auth-status-badge error">状态异常</span>`
              : alertStatus !== "normal"
                ? `<span class="quota-warning-badge ${alertStatus}">${alertLabel}</span>`
                : "";

        let compactSubHtml = "";
        if (switching && !hasDetail && !hasPageError) {
          compactSubHtml = `<div class="account-sub loading-text"><span class="quota-label-text">正在加载…</span></div>`;
        } else if (isLoggedOut) {
          const loginHint = active
            ? isMobile
              ? "登录已失效，点击进入重新登录"
              : "登录已失效，请在右侧重新登录"
            : acc.autoAlias
              ? "尚未登录，点击前往登录"
              : "登录已失效，点击重新登录";
          compactSubHtml = `<div class="account-sub auth-logged-out"><span class="auth-dot-inline"></span><span class="quota-label-text">${loginHint}</span></div>`;
        } else if (hasPageError) {
          compactSubHtml = `<div class="account-sub auth-page-error"><span class="auth-dot-inline error"></span><span class="quota-label-text">${escapeHtml(pageError!)}</span><button class="inline-relogin-btn" data-relogin-uuid="${escapeHtml(acc.uuid)}" type="button">清理重登</button></div>`;
        } else if (!hasDetail) {
          const fallbackLabel = acc.autoAlias
            ? "登录后自动同步额度"
            : "正在同步额度明细…";
          const quotaLabel = acc.quotaText ?? fallbackLabel;
          const isEmpty =
            acc.quotaPercent === 0 || Boolean(acc.quotaText?.includes("用尽"));
          const isLow =
            !isEmpty &&
            ((typeof acc.quotaPercent === "number" && acc.quotaPercent <= 20) ||
              Boolean(acc.quotaText?.includes("偏低")));
          const subClass = isEmpty
            ? "account-sub quota-empty"
            : isLow
              ? "account-sub quota-low"
              : "account-sub";
          const barHtml =
            typeof acc.quotaPercent === "number"
              ? `<span class="mini-quota-track"><span class="mini-quota-fill ${isEmpty ? "empty" : isLow ? "low" : ""}" style="width:${Math.max(0, Math.min(100, acc.quotaPercent))}%"></span></span>`
              : "";
          compactSubHtml = `<div class="${subClass}">${barHtml}<span class="quota-label-text">${escapeHtml(quotaLabel)}</span></div>`;
        }

        const rowClass = [
          "account-row",
          hasDetail ? "has-detail" : "",
          active ? "active" : "",
          switching ? "switching" : "",
          isLoggedOut ? "logged-out" : "",
          hasPageError ? "has-page-error" : "",
          alertStatus !== "normal" ? `alert-${alertStatus}` : "",
        ]
          .filter(Boolean)
          .join(" ");

        const avatarClass = [
          "avatar",
          acc.autoAlias ? "pending-identity" : acc.providerId,
        ].join(" ");

        const avatarNodeHtml = acc.avatarUrl
          ? `<div class="avatar has-photo"><img src="${escapeHtml(acc.avatarUrl)}" class="avatar-photo" alt="" draggable="false" /></div>`
          : `<div class="${avatarClass}">${escapeHtml(avatarText)}</div>`;

        const showManualAliasHint =
          !isLoggedOut &&
          !hasPageError &&
          acc.authState === "authenticated" &&
          acc.autoAlias &&
          acc.alias === "已登录账号";

        return `
          <div class="${rowClass}" data-switch-uuid="${escapeHtml(acc.uuid)}" ${!isMobile && canReorder ? `draggable="true"` : ""}>
            <div class="account-header">
              <div class="avatar-wrap">
                ${avatarNodeHtml}
                ${switching ? `<span class="avatar-spinner"></span>` : ""}
                ${active ? `<span class="status-dot ${isLoggedOut ? "offline" : hasPageError ? "error" : ""}"></span>` : ""}
              </div>
              <div class="account-meta">
                <div class="account-alias-row">
                  <span class="account-alias">${escapeHtml(acc.alias)}</span>
                  ${shortcutHint}
                  ${proxyBadgeHtml}
                  ${timezoneBadgeHtml}
                  ${alertBadgeHtml}
                </div>
                ${showManualAliasHint ? `<div class="alias-manual-hint"><span>未获取到邮箱</span><button class="inline-rename-link" data-rename-uuid="${escapeHtml(acc.uuid)}" type="button">手动改名</button></div>` : ""}
                ${compactSubHtml}
                ${!hasDetail && !isLoggedOut && !hasPageError && !switching && cookieInfo ? `<div class="cookie-expiry-line inline ${cookieInfo.tone}" title="${escapeHtml(cookieInfo.tooltip)}"><span class="cookie-expiry-dot"></span><span>${escapeHtml(cookieInfo.shortText)}</span>${acc.alias ? `<span class="cookie-acct-name" title="${escapeHtml(acc.alias)}">${escapeHtml(acc.alias)}</span>` : ""}</div>` : ""}
              </div>
              <div class="row-ops">
                ${canReorder ? `
                  <div class="reorder-stack">
                    <button class="reorder-btn" data-move-uuid="${escapeHtml(acc.uuid)}" data-move-dir="up" title="上移" aria-label="上移" type="button" ${idx === 0 ? "disabled" : ""}>▲</button>
                    <button class="reorder-btn" data-move-uuid="${escapeHtml(acc.uuid)}" data-move-dir="down" title="下移" aria-label="下移" type="button" ${idx === providerAccounts.length - 1 ? "disabled" : ""}>▼</button>
                  </div>
                ` : ""}
                ${!isMobile ? `
                  <button class="action-icon-btn ${switching ? "spinning" : ""}" data-reload-uuid="${escapeHtml(acc.uuid)}" title="强制刷新页面" type="button" ${switching ? "disabled" : ""}>${ICON_REFRESH}</button>
                ` : ""}
                ${supportsCookieCopy ? `
                  <button class="action-icon-btn copy-cookie-btn" data-copy-cookie-uuid="${escapeHtml(acc.uuid)}" data-copy-label="复制 Cookie" aria-label="复制 Cookie" type="button">
                    <span class="copy-cookie-icon" aria-hidden="true"></span>
                  </button>
                ` : ""}
                <button class="action-icon-btn" data-rename-uuid="${escapeHtml(acc.uuid)}" title="账号设置" type="button">${ICON_EDIT}</button>
                <button class="action-icon-btn danger" data-remove-uuid="${escapeHtml(acc.uuid)}" title="从本机移除" type="button">${ICON_TRASH}</button>
              </div>
            </div>
            ${!isLoggedOut && !hasPageError && acc.quotaDetail ? renderQuotaDetailPanel(acc.quotaDetail, alertStatus, extraDepleted) : ""}
            ${hasDetail && cookieInfo ? `<div class="cookie-expiry-line ${cookieInfo.tone}" title="${escapeHtml(cookieInfo.tooltip)}"><span class="cookie-expiry-dot"></span><span>${escapeHtml(cookieInfo.shortText)}</span>${acc.alias ? `<span class="cookie-acct-name" title="${escapeHtml(acc.alias)}">${escapeHtml(acc.alias)}</span>` : ""}</div>` : ""}
          </div>
        `;
      })
      .join("");
  }

  return `
    <div class="column-header">
      <div class="provider-title-wrap">
        <span class="provider-title">${escapeHtml(provider.name)}</span>
        ${
          !provider.enabled
            ? `<span class="status-pill">${escapeHtml(provider.statusText)}</span>`
            : providerAccounts.length > 0
              ? `<span class="count-badge">${providerAccounts.length}</span>`
              : ""
        }
      </div>
      <div class="header-actions">${headerBtnHtml}</div>
    </div>
    ${listToolbarHtml}
    ${noticeHtml}
    ${conflictsHtml}
    <div class="account-list">${listBody}</div>
    ${renderDownloadToast()}
    ${renderModal(state, provider)}
  `;
}

function renderDownloadToast(): string {
  if (!downloadToast) return "";
  const isStarted = downloadToast.status === "started";
  const isDone = downloadToast.status === "completed";
  const isFail = downloadToast.status === "failed";
  const label = isStarted
    ? `正在下载：${downloadToast.filename}`
    : isDone
      ? `下载完成：${downloadToast.filename}`
      : `下载失败：${downloadToast.filename}${downloadToast.error ? `（${downloadToast.error}）` : ""}`;
  return `
    <div id="download-toast" class="download-toast ${isFail ? "error" : isDone ? "success" : ""}" role="status" aria-live="polite">
      <span class="download-toast-text" title="${escapeHtml(label)}">${escapeHtml(label)}</span>
      ${isDone ? `<button id="download-reveal-btn" class="download-toast-action" data-download-id="${downloadToast.id}" type="button">在文件夹中显示</button>` : ""}
      <button id="download-close-btn" class="download-toast-close" type="button" aria-label="关闭下载提示">✕</button>
    </div>
  `;
}

function renderRightStage(state: ManagerSnapshot, provider: ProviderInfo): string {
  const switchingKey = effectiveSwitchingKey(state);
  const switchingAcc = switchingKey
    ? state.accounts.find(
        (a) => a.providerId === switchingKey.providerId && a.uuid === switchingKey.uuid
      )
    : undefined;

  if (switchingKey) {
    const targetName = switchingAcc?.alias ?? provider.shortName;
    return `
      <main class="right-placeholder">
        <div class="stage-card">
          <div class="stage-logo-orbit">
            <span class="stage-orbit-ring"></span>
            <div class="stage-logo-tile">${renderProviderLogo(provider)}</div>
          </div>
          <div class="placeholder-title">正在加载「${escapeHtml(targetName)}」</div>
          <div class="placeholder-desc">首次打开正在渲染 ${escapeHtml(provider.shortName)} 页面，加载完成后再次切换将瞬间直达…</div>
          <div class="progress-track"><div class="progress-bar"></div></div>
        </div>
      </main>
    `;
  }

  if (!provider.enabled) {
    return `
      <main class="right-placeholder">
        <div class="stage-card">
          <div class="stage-logo-tile">${renderProviderLogo(provider)}</div>
          <div class="placeholder-title">${escapeHtml(provider.name)} · 待接入</div>
          <div class="placeholder-desc">首版暂未启用应用内嵌会话。访问官网不会覆盖上次已打开的账号记录。</div>
          <button id="stage-open-site-btn" class="btn primary" type="button">打开 ${escapeHtml(provider.shortName)} 官网</button>
        </div>
      </main>
    `;
  }

  return `
    <main class="right-placeholder">
      <div class="stage-card">
        <div class="stage-logo-tile">${renderProviderLogo(provider)}</div>
        <div class="placeholder-title">${escapeHtml(provider.name)}</div>
        <div class="placeholder-desc">从左侧选择一个账号进入对话，或新建账号开启独立隔离登录。</div>
        <button id="stage-add-btn" class="btn primary" type="button">+ 新增 ${escapeHtml(provider.shortName)} 账号</button>
      </div>
    </main>
  `;
}

async function openCreateGroupModal(state: ManagerSnapshot): Promise<void> {
  await setModal({
    kind: "group",
    nameDraft: `群聊 ${state.groupChats.length + 1}`,
    selectedMembers: [],
  });
}

async function openEditGroupModal(group: GroupChatRecord): Promise<void> {
  await setModal({
    kind: "group",
    group,
    nameDraft: group.name,
    selectedMembers: group.members.map((m) => ({ providerId: m.providerId, uuid: m.uuid })),
  });
}

async function openGroupKnowledgeModal(
  group: GroupChatRecord,
  tab: "context" | "skills" | "roles" | "adapters" = "context"
): Promise<void> {
  await setModal({
    kind: "group-knowledge",
    group,
    tab,
    contextDraft: group.sharedContext ?? "",
    skillsDraft: (group.sharedSkills ?? []).map((s) => ({ ...s })),
    rolesDraft: (group.memberRoles ?? []).map((r) => ({
      member: { ...r.member },
      role: r.role,
    })),
    skillNameDraft: "",
    skillDescDraft: "",
    skillContentDraft: "",
    editingSkillId: null,
    adaptersSnap: null,
    adaptersJsonDraft: "",
    adaptersLoaded: false,
  });
}

async function openExternalAgentsModal(editAgent?: ExternalAgentRecord): Promise<void> {
  await setModal({
    kind: "external-agents",
    editingId: editAgent?.id ?? null,
    formKind: editAgent?.kind ?? "openai_api",
    nameDraft: editAgent?.name ?? "",
    baseUrlDraft: editAgent?.baseUrl ?? "",
    apiKeyDraft: editAgent?.apiKey ?? "",
    modelDraft: editAgent?.model ?? "",
    commandDraft: editAgent?.command ?? "",
    argsDraft: editAgent ? editAgent.args.join(" ") : "",
    workdirDraft: editAgent?.workdir ?? "",
  });
}

function isAsciiWordChar(ch: string | undefined): boolean {
  return Boolean(ch && /^[A-Za-z0-9_]$/.test(ch));
}

const ALL_MENTION_KEYWORDS = ["全体成员", "所有人", "全体", "all"];

function scanTextMentions<T extends { alias: string }>(
  text: string,
  members: T[]
): Array<{ start: number; end: number; isAll: boolean; member?: T }> {
  const sorted = [...members]
    .filter((m) => m.alias.trim().length > 0)
    .sort((a, b) => b.alias.length - a.alias.length);
  const hits: Array<{ start: number; end: number; isAll: boolean; member?: T }> = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] !== "@" || (i > 0 && isAsciiWordChar(text[i - 1]))) {
      i += 1;
      continue;
    }
    const rest = text.slice(i + 1);
    const restLower = rest.toLowerCase();
    let matched = false;
    for (const kw of ALL_MENTION_KEYWORDS) {
      if (restLower.startsWith(kw)) {
        const after = rest[kw.length];
        if (!isAsciiWordChar(after)) {
          hits.push({ start: i, end: i + 1 + kw.length, isAll: true });
          i += 1 + kw.length;
          matched = true;
          break;
        }
      }
    }
    if (matched) continue;
    for (const m of sorted) {
      const alias = m.alias;
      if (restLower.startsWith(alias.toLowerCase())) {
        const after = rest[alias.length];
        const lastAliasChar = alias[alias.length - 1];
        if (!(isAsciiWordChar(lastAliasChar) && isAsciiWordChar(after))) {
          hits.push({ start: i, end: i + 1 + alias.length, isAll: false, member: m });
          i += 1 + alias.length;
          matched = true;
          break;
        }
      }
    }
    if (!matched) {
      i += 1;
    }
  }
  return hits;
}

function extractComposerAtMentions(
  text: string,
  members: Array<{ key: AccountKey; alias: string }>
): { hasMention: boolean; isAll: boolean; targets: AccountKey[]; labels: string[] } {
  const hits = scanTextMentions(text, members);
  if (hits.length === 0) {
    return { hasMention: false, isAll: false, targets: [], labels: [] };
  }
  if (hits.some((h) => h.isAll)) {
    return { hasMention: true, isAll: true, targets: [], labels: ["全体成员"] };
  }
  const targets: AccountKey[] = [];
  const labels: string[] = [];
  for (const h of hits) {
    if (h.member && !targets.some((t) => sameAccountKey(t, h.member!.key))) {
      targets.push(h.member.key);
      labels.push(h.member.alias);
    }
  }
  return { hasMention: targets.length > 0, isAll: false, targets, labels };
}

function renderMessageTextWithMentions(content: string, memberAliases: string[]): string {
  const hits = scanTextMentions(
    content,
    memberAliases.map((alias) => ({ alias }))
  );
  if (hits.length === 0) {
    return escapeHtml(content);
  }
  let out = "";
  let cursor = 0;
  for (const hit of hits) {
    if (hit.start > cursor) {
      out += escapeHtml(content.slice(cursor, hit.start));
    }
    out += `<span class="group-mention-tag">${escapeHtml(content.slice(hit.start, hit.end))}</span>`;
    cursor = hit.end;
  }
  if (cursor < content.length) {
    out += escapeHtml(content.slice(cursor));
  }
  return out;
}

function formatReviewStageChain(labels: string[]): string {
  if (labels.length < 2) return "";
  return labels
    .map((label, idx) => {
      const stage =
        idx === 0
          ? "初稿"
          : idx === labels.length - 1 && labels.length >= 3
            ? "综合定稿"
            : "交叉评审";
      return `${label}（${stage}）`;
    })
    .join(" → ");
}

function formatComposerHintText(
  draft: string,
  selectedTargets: AccountKey[],
  members: Array<{ key: AccountKey; alias: string; categoryId: string }>,
  workflowMode: GroupWorkflowMode = "parallel",
  maxRelayHops = 2,
  selectedAutoChannels: Array<{ id: string; label: string }> = []
): string {
  const inline = extractComposerAtMentions(draft, members);
  const modeSuffix =
    workflowMode === "review"
      ? " · 评审流（初稿 → 评审 → 定稿）"
      : workflowMode === "a2a_relay"
        ? ` · A2A接力（最多 ${maxRelayHops} 跳）`
        : "";
  if (inline.hasMention) {
    if (inline.isAll) {
      return `检测到 @全体成员：将按排队限流向全员转发${modeSuffix}`;
    }
    const inlineCats = new Set(
      inline.targets
        .map((tk) => members.find((m) => sameAccountKey(m.key, tk))?.categoryId)
        .filter((x): x is string => Boolean(x))
    );
    const extraAuto = selectedAutoChannels.filter((c) => !inlineCats.has(c.id));
    if (workflowMode === "review") {
      const reviewLabels = [
        ...inline.labels.map((l) => `@${l}`),
        ...extraAuto.map((c) => `${c.label}(自动)`),
      ];
      if (reviewLabels.length >= 2) {
        return `检测到正文 @，评审流顺序：${formatReviewStageChain(reviewLabels)}`;
      }
    }
    if (extraAuto.length > 0) {
      return `检测到正文 @${inline.labels.join("、@")}：将同时转发给 ${extraAuto
        .map((c) => `${c.label}(自动选号)`)
        .join("、")}${modeSuffix}`;
    }
    return `检测到正文 @${inline.labels.join("、@")}：将仅转发给被提及成员${modeSuffix}`;
  }
  const hasExplicit = selectedTargets.length > 0;
  const hasAuto = selectedAutoChannels.length > 0;
  if (hasExplicit || hasAuto) {
    const parts: string[] = [];
    for (const t of selectedTargets) {
      parts.push(
        `@${members.find((m) => sameAccountKey(m.key, t))?.alias ?? t.uuid.slice(0, 6)}`
      );
    }
    for (const ac of selectedAutoChannels) {
      parts.push(`${ac.label}(自动按额度选号)`);
    }
    if (workflowMode === "review" && parts.length >= 2) {
      return `评审流顺序：${formatReviewStageChain(parts)}`;
    }
    return `仅向 ${parts.join("、")} 发送${modeSuffix}`;
  }
  if (workflowMode === "review") {
    return "评审流模式：按成员顺序串行执行（初稿 → 交叉评审 → 综合定稿）；按顺序点选上方成员或在正文按顺序 @ 可自定义先后顺序";
  }
  if (workflowMode === "a2a_relay") {
    return `A2A 接力模式：成员回复正文包含 @成员 将自动触发下一跳（上限 ${maxRelayHops} 跳）`;
  }
  return "向全体成员发送（输入 @ 可点名，Enter 发送，Shift+Enter 换行）";
}

function renderGroupMiddleContent(state: ManagerSnapshot, provider: ProviderInfo): string {
  const noticeHtml = state.notice
    ? `
      <div class="notice-bar">
        <span>${escapeHtml(state.notice)}</span>
        <button id="dismiss-notice-btn" class="notice-close" type="button" title="关闭提示">✕</button>
      </div>
    `
    : "";

  const extCount = (state.externalAgents ?? []).length;

  const listBody =
    state.groupChats.length === 0
      ? `
        <div class="empty-list">
          <div class="empty-icon-circle">${ICON_GROUP}</div>
          <div>暂无群聊</div>
          <div>点击右上角「+ 创建群聊」，可混编网页账号与 API / CLI 成员协同对话</div>
        </div>
      `
      : state.groupChats
          .map((g) => {
            const active = state.activeGroupId === g.id;
            const hasNeedsAuth = groupHasNeedsAuth(g);
            const isStreaming = g.messages.some(
              (m) => m.status === "queued" || m.status === "sending" || m.status === "streaming"
            );
            const lastMsg = g.messages[g.messages.length - 1];
            const lastSenderName = lastMsg
              ? resolveGroupMessageSenderName(state, lastMsg)
              : "";
            const preview = lastMsg
              ? lastMsg.status === "needs_auth"
                ? `[待授权] ${lastSenderName}`
                : lastMsg.role === "user"
                  ? `我: ${lastMsg.content}`
                  : lastMsg.status === "queued"
                    ? `${lastSenderName}: 排队等待回复…`
                    : lastMsg.content
                      ? `${lastSenderName}: ${lastMsg.content}`
                      : `${lastSenderName} 正在回复…`
              : `${g.members.length} 位成员 · 点击开始群聊`;

            const avatarStack = g.members
              .slice(0, 3)
              .map((m) => {
                const meta = resolveMemberMeta(state, m);
                return renderMemberAvatarNode(
                  state,
                  meta.acc,
                  meta.alias || "成员",
                  "group-mini-avatar",
                  meta.ext?.kind
                );
              })
              .join("");

            return `
              <div class="group-row ${active ? "active" : ""}" data-select-group-id="${escapeHtml(g.id)}">
                <div class="group-avatar-stack">
                  ${avatarStack || `<div class="avatar custom group-mini-avatar">群</div>`}
                </div>
                <div class="group-row-meta">
                  <div class="group-row-top">
                    <span class="account-alias">${escapeHtml(g.name)}</span>
                    <span class="group-count-pill">${g.members.length}人</span>
                    ${hasNeedsAuth ? `<span class="quota-warning-badge low">待授权</span>` : isStreaming ? `<span class="loading-status-badge">回复中</span>` : ""}
                  </div>
                  <div class="group-row-preview">${escapeHtml(preview)}</div>
                </div>
                <div class="row-ops">
                  <button class="action-icon-btn" data-edit-group-id="${escapeHtml(g.id)}" title="编辑群成员与名称" type="button">${ICON_EDIT}</button>
                  <button class="action-icon-btn danger" data-delete-group-id="${escapeHtml(g.id)}" title="删除群聊" type="button">${ICON_TRASH}</button>
                </div>
              </div>
            `;
          })
          .join("");

  return `
    <div class="column-header">
      <div class="provider-title-wrap">
        <span class="provider-title">多账号群聊</span>
        ${state.groupChats.length > 0 ? `<span class="count-badge">${state.groupChats.length}</span>` : ""}
      </div>
      <div class="header-actions">
        <button id="manage-external-agents-btn" class="icon-btn ext-header-btn" type="button" title="管理直连 OpenAI API 与本地 CLI 成员">API/CLI${extCount > 0 ? ` (${extCount})` : ""}</button>
        <button id="create-group-btn" class="icon-btn primary" type="button">+ 创建群聊</button>
      </div>
    </div>
    ${noticeHtml}
    <div class="account-list">${listBody}</div>
    ${renderDownloadToast()}
    ${renderModal(state, provider)}
  `;
}

function getLatestRoundCompletedReplies(
  state: ManagerSnapshot,
  group: GroupChatRecord
): { userText: string; replies: Array<{ speaker: string; text: string }> } | null {
  let lastUserIdx = -1;
  for (let i = group.messages.length - 1; i >= 0; i -= 1) {
    if (group.messages[i].role === "user") {
      lastUserIdx = i;
      break;
    }
  }
  if (lastUserIdx < 0) return null;
  const replies = group.messages
    .slice(lastUserIdx + 1)
    .filter((m) => m.role === "assistant" && m.status === "done" && m.content.trim().length > 0)
    .map((m) => ({
      speaker: resolveGroupMessageSenderName(state, m),
      text: m.content.trim(),
    }));
  if (replies.length === 0) return null;
  return { userText: group.messages[lastUserIdx].content.trim(), replies };
}

function renderGroupRightStage(state: ManagerSnapshot): string {
  const activeGroup = state.activeGroupId
    ? state.groupChats.find((g) => g.id === state.activeGroupId)
    : undefined;

  if (!activeGroup) {
    return `
      <main class="right-placeholder">
        <div class="stage-card">
          <div class="stage-logo-tile group-stage-icon">${ICON_GROUP}</div>
          <div class="placeholder-title">创建你的跨服务 AI 群聊</div>
          <div class="placeholder-desc">支持 Muse / Hark 网页账号与 OpenAI API / 本地 CLI 混编入群，内置并行、评审流、A2A 接力与独立会话绑定。</div>
          <div class="stage-card-actions">
            <button id="stage-create-group-btn" class="btn primary" type="button">+ 创建群聊</button>
          </div>
        </div>
      </main>
    `;
  }

  const workflowMode: GroupWorkflowMode = activeGroup.workflowMode ?? "parallel";
  const maxRelayHops = activeGroup.maxRelayHops ?? 2;
  const draftVal = groupDrafts[activeGroup.id] ?? "";
  const selectedTargets = (groupTargets[activeGroup.id] ?? []).filter((t) =>
    activeGroup.members.some((m) => sameAccountKey(m, t))
  );
  const memberEntries = activeGroup.members.map((m) => {
    const meta = resolveMemberMeta(state, m);
    const isNeedsAuth = activeGroup.messages.some(
      (msg) =>
        msg.status === "needs_auth" && msg.sender && sameAccountKey(msg.sender, m)
    );
    const isBusy = activeGroup.messages.some(
      (msg) =>
        (msg.status === "queued" || msg.status === "sending" || msg.status === "streaming") &&
        msg.sender &&
        sameAccountKey(msg.sender, m)
    );
    const hasBinding = !meta.isExternal && Boolean(
      (activeGroup.threadBindings ?? []).some((tb) => sameAccountKey(tb.member, m))
    );
    return {
      key: m,
      isExternal: meta.isExternal,
      acc: meta.acc,
      ext: meta.ext,
      providerShort: meta.providerShort,
      alias: meta.alias,
      loggedOut: meta.loggedOut,
      warm: meta.warm,
      isNeedsAuth,
      isBusy,
      hasBinding,
    };
  });

  const targetCategories = groupByMemberCategory(
    state,
    memberEntries.filter((m) => !m.loggedOut)
  );
  const explicitTargetCats = new Set(
    selectedTargets.map((t) => resolveMemberCategory(state, t).id)
  );
  const selectedAutoIds = (groupAutoChannels[activeGroup.id] ?? []).filter(
    (catId) =>
      !explicitTargetCats.has(catId) &&
      targetCategories.some((c) => c.id === catId && c.items.length >= 2)
  );
  if ((groupAutoChannels[activeGroup.id] ?? []).length !== selectedAutoIds.length) {
    groupAutoChannels[activeGroup.id] = selectedAutoIds;
  }
  const selectedAutoChannels = selectedAutoIds
    .map((id) => {
      const cat = targetCategories.find((c) => c.id === id);
      return cat ? { id: cat.id, label: cat.label } : null;
    })
    .filter((x): x is { id: string; label: string } => Boolean(x));

  const memberAliases = memberEntries.map((m) => m.alias);
  const composerMembers = memberEntries.map((m) => ({
    key: m.key,
    alias: m.alias,
    categoryId: resolveMemberCategory(state, m.key).id,
  }));
  const targetHint = formatComposerHintText(
    draftVal,
    selectedTargets,
    composerMembers,
    workflowMode,
    maxRelayHops,
    selectedAutoChannels
  );
  const knowledgeCount =
    (activeGroup.sharedContext?.trim() ? 1 : 0) +
    (activeGroup.sharedSkills ?? []).filter((s) => s.enabled).length;
  const canPinRound = Boolean(getLatestRoundCompletedReplies(state, activeGroup));

  const msgsHtml =
    activeGroup.messages.length === 0
      ? `
        <div class="group-empty-chat">
          <div class="group-empty-title">群聊已就绪</div>
          <div class="group-empty-desc">可在下方切换「并行」「评审流」或「A2A 接力」协作模式；输入 @ 可模糊点名，网页成员将自动绑定独立专属会话。</div>
        </div>
      `
      : activeGroup.messages
          .map((msg: GroupChatMessage) => {
            if (msg.role === "user") {
              const targetSummary =
                msg.targets.length > 0 && msg.targets.length < activeGroup.members.length
                  ? `<div class="group-msg-targets">@${msg.targets
                      .map((t) => {
                        const meta = resolveMemberMeta(state, t);
                        return meta.alias;
                      })
                      .map(escapeHtml)
                      .join(" · @")}</div>`
                  : "";
              return `
                <div class="group-msg user">
                  <div class="group-bubble user">
                    ${targetSummary}
                    <div class="group-msg-text">${renderMessageTextWithMentions(msg.content, memberAliases)}</div>
                    <div class="group-msg-foot">${formatShortTime(msg.createdAtMs)} · 已转发</div>
                    <div class="group-save-ctx-wrap">
                      <button class="group-pin-icon-btn" data-pin-msg-id="${escapeHtml(msg.id)}" data-tooltip="沉淀到共享上下文" aria-label="沉淀到共享上下文" type="button">${ICON_PIN}</button>
                    </div>
                  </div>
                </div>
              `;
            }

            const senderKey = msg.sender;
            const meta = senderKey ? resolveMemberMeta(state, senderKey) : null;
            const senderDisplayName = resolveGroupMessageSenderName(state, msg);
            const jumpAttrs = senderKey
              ? meta?.isExternal
                ? `data-open-ext-agent-id="${escapeHtml(senderKey.uuid)}"`
                : `data-jump-provider="${escapeHtml(senderKey.providerId)}" data-jump-uuid="${escapeHtml(senderKey.uuid)}"`
              : "";
            const statusPill =
              msg.status === "queued"
                ? `<span class="group-status-pill queued">排队中…</span>`
                : msg.status === "sending"
                  ? `<span class="group-status-pill sending">投递中…</span>`
                  : msg.status === "streaming"
                    ? `<span class="group-status-pill streaming">回复中…</span>`
                    : msg.status === "needs_auth"
                      ? `<span class="group-status-pill warn">待授权确认</span>`
                      : msg.status === "error"
                        ? `<span class="group-status-pill err">未完成</span>`
                        : "";
            const stagePill = msg.stageLabel
              ? `<span class="group-stage-pill">${escapeHtml(msg.stageLabel)}</span>`
              : "";

            const bodyContent = msg.content.trim()
              ? `<div class="group-msg-text">${renderMessageTextWithMentions(msg.content, memberAliases)}</div>`
              : msg.status === "queued"
                ? `<div class="group-msg-pending">等待前序阶段或槽位释放后唤醒…</div>`
                : msg.status === "sending" || msg.status === "streaming"
                  ? `<div class="group-msg-pending">${meta?.isExternal ? "正在直连执行并流式接收回复…" : "正在该账号原生对话中生成回复…"}</div>`
                  : "";

            const canSyncReply = Boolean(
              senderKey &&
                !meta?.isExternal &&
                msg.status !== "queued" &&
                msg.status !== "sending" &&
                msg.status !== "streaming"
            );
            const isSyncing = syncingMsgIds.has(msg.id);
            const syncBtnHtml = canSyncReply
              ? `<button class="group-pin-icon-btn ${isSyncing ? "is-spinning" : ""}" data-sync-msg-id="${escapeHtml(msg.id)}" data-tooltip="同步最新回复（不重发）" aria-label="同步最新回复" type="button" ${isSyncing ? "disabled" : ""}>${ICON_REFRESH}</button>`
              : "";
            const pinBtnHtml = msg.content.trim()
              ? `<button class="group-pin-icon-btn" data-pin-msg-id="${escapeHtml(msg.id)}" data-tooltip="沉淀到共享上下文" aria-label="沉淀到共享上下文" type="button">${ICON_PIN}</button>`
              : "";
            const pinHoverAction =
              syncBtnHtml || pinBtnHtml
                ? `<div class="group-save-ctx-wrap ${isSyncing ? "is-active" : ""}">${syncBtnHtml}${pinBtnHtml}</div>`
                : "";

            const authCard =
              msg.status === "needs_auth"
                ? `
                  <div class="group-auth-card">
                    <span>${escapeHtml(msg.error || "检测到安全授权弹窗，请切到该账号确认后重试")}</span>
                    <div class="group-error-ops">
                      <button class="mini-btn primary-text" data-retry-msg-id="${escapeHtml(msg.id)}" type="button">重试</button>
                      ${canSyncReply ? `<button class="mini-btn" data-sync-msg-id="${escapeHtml(msg.id)}" type="button" ${isSyncing ? "disabled" : ""}>${isSyncing ? "同步中…" : "同步回复"}</button>` : ""}
                      ${senderKey ? `<button class="mini-btn warn-btn" type="button" ${jumpAttrs}>前往该账号授权 →</button>` : ""}
                    </div>
                  </div>
                `
                : msg.status === "error" && msg.error
                  ? `
                    <div class="group-error-box">
                      <span>${escapeHtml(msg.error)}</span>
                      <div class="group-error-ops">
                        <button class="mini-btn primary-text" data-retry-msg-id="${escapeHtml(msg.id)}" type="button">重试</button>
                        ${canSyncReply ? `<button class="mini-btn" data-sync-msg-id="${escapeHtml(msg.id)}" type="button" ${isSyncing ? "disabled" : ""}>${isSyncing ? "同步中…" : "同步回复"}</button>` : ""}
                        ${senderKey ? `<button class="mini-btn" type="button" ${jumpAttrs}>${meta?.isExternal ? "检查配置" : "前往查看"}</button>` : ""}
                      </div>
                    </div>
                  `
                  : "";

            return `
              <div class="group-msg assistant">
                <button class="group-msg-avatar-btn" type="button" title="${meta?.isExternal ? `点击编辑「${escapeHtml(senderDisplayName)}」配置` : `点击切换到「${escapeHtml(senderDisplayName)}」原生会话页`}" ${jumpAttrs}>
                  ${renderMemberAvatarNode(state, meta?.acc, senderDisplayName, "", meta?.ext?.kind)}
                </button>
                <div class="group-msg-col">
                  <div class="group-msg-head">
                    <span class="group-msg-name">${escapeHtml(senderDisplayName)}</span>
                    ${meta ? `<span class="group-provider-tag">${escapeHtml(meta.providerShort)}</span>` : ""}
                    ${msg.autoRoutedChannel ? `<span class="group-auto-pill" title="由渠道按剩余额度自动选号">自动选号</span>` : ""}
                    ${stagePill}
                    <span class="group-msg-time">${formatShortTime(msg.createdAtMs)}</span>
                    ${statusPill}
                  </div>
                  <div class="group-bubble assistant ${msg.status === "needs_auth" ? "needs-auth" : msg.status === "error" ? "has-error" : ""}">
                    ${bodyContent}
                    ${authCard}
                    ${pinHoverAction}
                  </div>
                </div>
              </div>
            `;
          })
          .join("");

  return `
    <main class="group-chat-stage">
      <div class="group-chat-header">
        <div class="group-header-main">
          <div class="group-header-title-row">
            <h2 class="group-chat-title">${escapeHtml(activeGroup.name)}</h2>
            <span class="count-badge">${activeGroup.members.length} 位成员</span>
          </div>
          <div class="group-member-bar">
            ${groupByMemberCategory(state, memberEntries)
              .map(
                (cat) => `
                  <div class="member-category-group">
                    <span class="member-category-badge" title="${escapeHtml(cat.label)}" aria-label="${escapeHtml(cat.label)}">${renderCategoryIcon(state, cat)}</span>
                    ${cat.items
                      .map((m) => {
                        const chipAttrs = m.isExternal
                          ? `data-open-ext-agent-id="${escapeHtml(m.key.uuid)}"`
                          : `data-jump-provider="${escapeHtml(m.key.providerId)}" data-jump-uuid="${escapeHtml(m.key.uuid)}"`;
                        const chipTitle = m.isExternal
                          ? `点击编辑「${escapeHtml(m.alias)}」直连配置`
                          : m.hasBinding
                            ? `已绑定群专属会话 · 点击进入「${escapeHtml(m.alias)}」(${escapeHtml(cat.label)}) 页面`
                            : `点击进入「${escapeHtml(m.alias)}」(${escapeHtml(cat.label)}) 原生会话页`;
                        return `
                          <button
                            class="group-member-chip ${m.isNeedsAuth ? "warn" : m.isBusy ? "busy" : m.loggedOut ? "offline" : m.warm ? "warm" : ""}"
                            ${chipAttrs}
                            title="${chipTitle}"
                            type="button"
                          >
                            <span class="group-chip-dot"></span>
                            <span>${escapeHtml(m.alias)}</span>
                            ${m.hasBinding ? `<span class="group-chip-provider">会话</span>` : ""}
                          </button>
                        `;
                      })
                      .join("")}
                  </div>
                `
              )
              .join("")}
          </div>
        </div>
        <div class="group-header-actions">
          ${newTopicConfirmGroupId === activeGroup.id
            ? `
              <div class="new-topic-confirm">
                <span class="new-topic-confirm-label">开启新话题：</span>
                <button id="group-new-topic-keep-btn" class="btn primary" type="button" title="清空记录并让网页成员各自新开对话，保留共享上下文与技能">保留共享上下文</button>
                <button id="group-new-topic-clean-btn" class="btn" type="button" title="清空记录、共享上下文，网页成员各自新开对话（技能与角色保留）">全部清空</button>
                <button id="group-new-topic-cancel-btn" class="btn" type="button">取消</button>
              </div>
            `
            : `
          <button id="group-new-topic-btn" class="btn" type="button" title="清空记录并让所有网页成员新开原生对话，开始一个干净的话题">新话题</button>
          ${canPinRound ? `<button id="group-pin-round-btn" class="btn knowledge-btn" type="button" title="将最近一轮各成员完成回复浓缩追加至共享上下文">${ICON_PIN}<span>沉淀本轮结论</span></button>` : ""}
          <button id="group-knowledge-btn" class="btn knowledge-btn" type="button">共享上下文 &amp; 技能${knowledgeCount > 0 ? `<span class="count-badge">${knowledgeCount}</span>` : ""}</button>
          <button id="group-edit-members-btn" class="btn" type="button">编辑成员</button>
          <button id="group-clear-msgs-btn" class="btn" type="button" ${activeGroup.messages.length === 0 ? "disabled" : ""}>清空记录</button>
            `}
        </div>
      </div>
      <div class="group-messages">${msgsHtml}</div>
      <div class="group-composer">
        <div class="group-workflow-bar">
          <div class="workflow-mode-group">
            <span class="workflow-label">模式</span>
            <button class="workflow-mode-btn ${workflowMode === "parallel" ? "active" : ""}" data-workflow-mode="parallel" type="button">并行</button>
            <button class="workflow-mode-btn ${workflowMode === "review" ? "active" : ""}" data-workflow-mode="review" type="button">评审流</button>
            <button class="workflow-mode-btn ${workflowMode === "a2a_relay" ? "active" : ""}" data-workflow-mode="a2a_relay" type="button">A2A接力</button>
          </div>
          ${workflowMode === "a2a_relay" ? `
            <div class="workflow-hops-group">
              <span class="workflow-label">接力上限</span>
              <button class="workflow-hop-btn ${maxRelayHops === 1 ? "active" : ""}" data-relay-hops="1" type="button">1跳</button>
              <button class="workflow-hop-btn ${maxRelayHops === 2 ? "active" : ""}" data-relay-hops="2" type="button">2跳</button>
              <button class="workflow-hop-btn ${maxRelayHops === 3 ? "active" : ""}" data-relay-hops="3" type="button">3跳</button>
            </div>
          ` : ""}
        </div>
        <div class="group-target-chips grouped">
          <button class="group-target-chip ${selectedTargets.length === 0 && selectedAutoIds.length === 0 ? "active" : ""}" data-group-target="all" type="button">全体成员</button>
          ${targetCategories
            .map((cat) => {
              const allCatSelected =
                cat.items.length > 0 &&
                cat.items.every((m) => selectedTargets.some((t) => sameAccountKey(t, m.key)));
              const isAutoActive = selectedAutoIds.includes(cat.id);
              const autoBtnHtml =
                cat.items.length >= 2
                  ? `<button
                      class="group-target-chip target-auto-chip ${isAutoActive ? "active" : ""}"
                      data-group-target-auto="${escapeHtml(cat.id)}"
                      title="${escapeHtml(cat.label)} 按剩余额度自动轮换选号（与该类具体 @ 互斥）"
                      type="button"
                    >自动</button>`
                  : "";
              return `
                <div class="target-category-group">
                  <button
                    class="target-category-badge ${allCatSelected || isAutoActive ? "active" : ""}"
                    data-group-target-cat="${escapeHtml(cat.id)}"
                    title="${escapeHtml(cat.label)} · 点击全选或取消该类全部成员"
                    aria-label="${escapeHtml(cat.label)}"
                    type="button"
                  >${renderCategoryIcon(state, cat)}</button>
                  <div class="target-category-items">
                    ${autoBtnHtml}
                    ${cat.items
                      .map((m) => {
                        const selectedIdx = selectedTargets.findIndex((t) =>
                          sameAccountKey(t, m.key)
                        );
                        const activeTarget = selectedIdx >= 0;
                        const orderPrefix =
                          workflowMode === "review" && activeTarget
                            ? `${selectedIdx + 1}·`
                            : "";
                        return `
                          <button
                            class="group-target-chip ${activeTarget ? "active" : ""}"
                            data-group-target-provider="${escapeHtml(m.key.providerId)}"
                            data-group-target-uuid="${escapeHtml(m.key.uuid)}"
                            type="button"
                          >${orderPrefix}@${escapeHtml(m.alias)}</button>
                        `;
                      })
                      .join("")}
                  </div>
                </div>
              `;
            })
            .join("")}
        </div>
        <div id="group-target-hint" class="group-composer-hint">${escapeHtml(targetHint)}</div>
        <div class="group-input-wrap">
          <div id="group-mention-menu" class="group-mention-menu" hidden></div>
          <div class="group-input-row">
            <textarea
              id="group-chat-input"
              class="group-chat-input"
              rows="2"
              maxlength="4000"
              placeholder="输入消息，输入 @ 可唤起成员点名（Enter 发送，Shift+Enter 换行）"
            >${escapeHtml(draftVal)}</textarea>
            <button id="group-send-btn" class="btn primary group-send-btn" type="button" ${groupSending ? "disabled" : ""}>${groupSending ? "发送中…" : "发送"}</button>
          </div>
        </div>
      </div>
    </main>
  `;
}

function render(): void {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root || !snapshot) return;

  const focusedInput = document.activeElement;
  const preservedInputIds = new Set([
    "modal-alias-input",
    "modal-proxy-input",
    "modal-timezone-input",
    "modal-shortcut-input",
    "modal-group-name-input",
    "modal-group-context-input",
    "modal-skill-name-input",
    "modal-skill-desc-input",
    "modal-skill-content-input",
    "modal-adapters-json-input",
    "ext-agent-name-input",
    "ext-agent-url-input",
    "ext-agent-key-input",
    "ext-agent-model-input",
    "ext-agent-cmd-input",
    "ext-agent-args-input",
    "ext-agent-workdir-input",
    "group-chat-input",
  ]);
  const accountInputSelection =
    (focusedInput instanceof HTMLInputElement || focusedInput instanceof HTMLTextAreaElement) &&
    preservedInputIds.has(focusedInput.id)
      ? {
          id: focusedInput.id,
          start: focusedInput.selectionStart,
          end: focusedInput.selectionEnd,
          direction: focusedInput.selectionDirection,
        }
      : null;

  const provider = currentProvider(snapshot);
  const isMobile = isMobilePlatform(snapshot.platform);
  const isGroupsView = !isMobile && viewMode === "groups";
  const anyGroupNeedsAuth = snapshot.groupChats.some(groupHasNeedsAuth);

  if (isMobile) {
    root.innerHTML = `
      <div class="mobile-shell">
        <div class="mobile-tabs">
          ${snapshot.providers
            .map(
              (p) => `
              <button class="mobile-tab ${p.id === provider.id ? "active" : ""}" data-provider="${escapeHtml(p.id)}" type="button">
                ${renderProviderLogo(p, "mobile-tab-logo")}
                <span>${escapeHtml(p.shortName)}</span>
              </button>
            `
            )
            .join("")}
        </div>
        <div class="middle-column" style="width:100%;min-width:0;flex:1;border-right:none;">
          ${renderMiddleContent(snapshot, provider)}
        </div>
      </div>
    `;
  } else {
    root.innerHTML = `
      <div class="desktop-shell">
        <aside class="service-rail">
          <div class="rail-brand" title="叠号 · AI 助手账号切换器">
            <img src="/logos/app-logo.svg" alt="叠号" class="rail-brand-img" draggable="false" />
          </div>
          <div class="rail-divider"></div>
          <div class="rail-providers">
            <button id="group-chat-rail-btn" class="rail-btn ${isGroupsView ? "active" : ""}" title="多账号群聊（桌面端）" type="button">
              <div class="rail-logo-box rail-group-icon">${ICON_GROUP}${anyGroupNeedsAuth ? `<span class="warn-dot" aria-hidden="true"></span>` : ""}</div>
              <span class="rail-label">群聊</span>
            </button>
            <div class="rail-divider"></div>
            ${snapshot.providers
              .map(
                (p) => `
                <button class="rail-btn ${!isGroupsView && p.id === provider.id ? "active" : ""} ${!p.enabled ? "disabled" : ""}" data-provider="${escapeHtml(p.id)}" title="${escapeHtml(p.name)}" type="button">
                  <div class="rail-logo-box">${renderProviderLogo(p, "rail-logo-img")}</div>
                  <span class="rail-label">${escapeHtml(p.shortName)}</span>
                  ${!p.enabled ? `<span class="rail-pending-tag">待接入</span>` : ""}
                </button>
              `
              )
              .join("")}
          </div>
          <div class="rail-footer">
            <button
              id="settings-btn"
              class="rail-util-btn ${snapshot.globalShortcutError ? "has-warn" : ""}"
              title="${snapshot.globalShortcutError ? escapeHtml(snapshot.globalShortcutError) : "托盘常驻与全局快捷键设置"}"
              type="button"
            >
              ${ICON_SETTINGS}
              <span class="rail-util-label">设置</span>
              ${snapshot.globalShortcutError ? `<span class="warn-dot" aria-hidden="true"></span>` : ""}
            </button>
            <button
              id="theme-toggle-btn"
              class="rail-util-btn"
              title="外观主题：${themeLabel(themeMode)}（点击切换）"
              type="button"
            >
              ${themeIcon(themeMode)}
              <span class="rail-util-label">${themeLabel(themeMode)}</span>
            </button>
            <button
              id="check-update-btn"
              class="rail-util-btn ${updateState.result?.hasUpdate ? "has-update" : ""}"
              title="${updateState.result?.hasUpdate ? `发现新版本 v${escapeHtml(updateState.result.latestVersion)}（点击更新）` : "检查更新"}"
              type="button"
            >
              ${ICON_UPDATE}
              <span class="rail-util-label">${updateState.result?.hasUpdate ? "新版本" : "更新"}</span>
              ${updateState.result?.hasUpdate ? `<span class="update-dot" aria-hidden="true"></span>` : ""}
            </button>
          </div>
        </aside>
        <section class="middle-column">
          ${isGroupsView ? renderGroupMiddleContent(snapshot, provider) : renderMiddleContent(snapshot, provider)}
        </section>
        ${isGroupsView ? renderGroupRightStage(snapshot) : renderRightStage(snapshot, provider)}
      </div>
    `;
  }

  bindEvents(snapshot, provider);
  if (accountInputSelection) {
    const input = document.querySelector<HTMLInputElement | HTMLTextAreaElement>(
      `#${accountInputSelection.id}`
    );
    input?.focus({ preventScroll: true });
    input?.setSelectionRange(
      accountInputSelection.start,
      accountInputSelection.end,
      accountInputSelection.direction ?? undefined
    );
  }
  shouldAnimateModal = false;
}

function bindEvents(state: ManagerSnapshot, provider: ProviderInfo): void {
  document.querySelector("#group-chat-rail-btn")?.addEventListener("click", async () => {
    if (viewMode === "groups") return;
    if (modal.kind !== "none") {
      await setModal({ kind: "none" });
    }
    viewMode = "groups";
    const targetId = state.activeGroupId ?? state.groupChats[0]?.id ?? null;
    await callCommand("select_group_chat", { groupId: targetId });
  });

  document.querySelectorAll<HTMLElement>("[data-provider]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const nextId = btn.dataset.provider as ProviderId;
      if (!nextId || (viewMode === "provider" && nextId === state.selectedProviderId)) return;
      if (modal.kind !== "none") {
        await setModal({ kind: "none" });
      }
      viewMode = "provider";
      await callCommand("select_provider", { providerId: nextId });
    });
  });

  const handleOpenCreateGroup = async () => {
    await openCreateGroupModal(state);
  };
  document.querySelector("#create-group-btn")?.addEventListener("click", handleOpenCreateGroup);
  document.querySelector("#stage-create-group-btn")?.addEventListener("click", handleOpenCreateGroup);

  const handleOpenExternalAgents = async () => {
    await openExternalAgentsModal();
  };
  document
    .querySelector("#manage-external-agents-btn")
    ?.addEventListener("click", handleOpenExternalAgents);
  document
    .querySelector("#open-external-agents-from-group-btn")
    ?.addEventListener("click", handleOpenExternalAgents);

  document.querySelectorAll<HTMLElement>("[data-open-ext-agent-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none") return;
      const eid = btn.dataset.openExtAgentId;
      const ext = (state.externalAgents ?? []).find((a) => a.id === eid);
      if (ext) {
        await openExternalAgentsModal(ext);
      }
    });
  });

  document.querySelectorAll<HTMLElement>("[data-edit-group-id]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const gid = btn.dataset.editGroupId;
      const g = state.groupChats.find((x) => x.id === gid);
      if (g) {
        await openEditGroupModal(g);
      }
    });
  });

  document.querySelectorAll<HTMLElement>("[data-delete-group-id]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const gid = btn.dataset.deleteGroupId;
      const g = state.groupChats.find((x) => x.id === gid);
      if (g) {
        await setModal({ kind: "delete-group", group: g });
      }
    });
  });

  document.querySelectorAll<HTMLElement>("[data-select-group-id]").forEach((row) => {
    row.addEventListener("click", async () => {
      if (modal.kind !== "none") return;
      const gid = row.dataset.selectGroupId;
      if (!gid || gid === state.activeGroupId) return;
      forceGroupScrollBottom = true;
      await callCommand("select_group_chat", { groupId: gid });
    });
  });

  document.querySelector("#group-knowledge-btn")?.addEventListener("click", async () => {
    const activeGroup = state.activeGroupId
      ? state.groupChats.find((g) => g.id === state.activeGroupId)
      : undefined;
    if (activeGroup) {
      await openGroupKnowledgeModal(activeGroup);
    }
  });

  document.querySelectorAll<HTMLButtonElement>("[data-pin-msg-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none" || !state.activeGroupId) return;
      const msgId = btn.dataset.pinMsgId;
      const activeGroup = state.groupChats.find((g) => g.id === state.activeGroupId);
      const msg = activeGroup?.messages.find((m) => m.id === msgId);
      if (!msg || !msg.content.trim()) return;
      const prefix =
        msg.role === "user"
          ? "[用户]"
          : `[${resolveGroupMessageSenderName(state, msg)}]`;
      await callCommand("append_group_shared_context", {
        groupId: state.activeGroupId,
        snippet: `${prefix}: ${msg.content.trim()}`,
      });
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-retry-msg-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none" || !state.activeGroupId) return;
      const msgId = btn.dataset.retryMsgId;
      if (!msgId) return;
      await callCommand("retry_group_message", {
        groupId: state.activeGroupId,
        msgId,
      });
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-sync-msg-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none" || !state.activeGroupId || busy) return;
      const msgId = btn.dataset.syncMsgId;
      if (!msgId || syncingMsgIds.has(msgId)) return;
      syncingMsgIds.add(msgId);
      rerenderPreservingScroll();
      try {
        await callCommand("sync_group_message_reply", {
          groupId: state.activeGroupId,
          msgId,
        });
      } finally {
        syncingMsgIds.delete(msgId);
        rerenderPreservingScroll();
      }
    });
  });

  document.querySelector("#group-edit-members-btn")?.addEventListener("click", async () => {
    const activeGroup = state.activeGroupId
      ? state.groupChats.find((g) => g.id === state.activeGroupId)
      : undefined;
    if (activeGroup) {
      await openEditGroupModal(activeGroup);
    }
  });

  document.querySelector("#group-clear-msgs-btn")?.addEventListener("click", async () => {
    if (!state.activeGroupId) return;
    await callCommand("clear_group_messages", { groupId: state.activeGroupId });
  });

  document.querySelector("#group-new-topic-btn")?.addEventListener("click", () => {
    if (modal.kind !== "none" || !state.activeGroupId) return;
    newTopicConfirmGroupId = state.activeGroupId;
    rerenderPreservingScroll();
  });

  document.querySelector("#group-new-topic-cancel-btn")?.addEventListener("click", () => {
    newTopicConfirmGroupId = null;
    rerenderPreservingScroll();
  });

  const startNewTopic = async (keepSharedContext: boolean) => {
    const gid = state.activeGroupId;
    newTopicConfirmGroupId = null;
    if (!gid) return;
    groupDrafts[gid] = "";
    groupTargets[gid] = [];
    groupAutoChannels[gid] = [];
    await callCommand("start_group_new_topic", { groupId: gid, keepSharedContext });
  };

  document.querySelector("#group-new-topic-keep-btn")?.addEventListener("click", () => {
    void startNewTopic(true);
  });

  document.querySelector("#group-new-topic-clean-btn")?.addEventListener("click", () => {
    void startNewTopic(false);
  });

  document.querySelector("#group-pin-round-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "none" || !state.activeGroupId) return;
    const activeGroup = state.groupChats.find((g) => g.id === state.activeGroupId);
    if (!activeGroup) return;
    const round = getLatestRoundCompletedReplies(state, activeGroup);
    if (!round) return;
    const head = round.userText
      ? `已有结论（议题：${round.userText.replace(/\s+/g, " ").slice(0, 80)}）`
      : "已有结论";
    const lines = round.replies.map(
      (r, i) => `- 观点${i + 1}: ${r.text.replace(/\s+/g, " ").slice(0, 280)}`
    );
    const snippet = `${head}\n${lines.join("\n")}`.slice(0, 1800);
    await callCommand("append_group_shared_context", {
      groupId: state.activeGroupId,
      snippet,
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-workflow-mode]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none" || !state.activeGroupId) return;
      const mode = btn.dataset.workflowMode as GroupWorkflowMode | undefined;
      const activeGroup = state.groupChats.find((g) => g.id === state.activeGroupId);
      if (!mode || !activeGroup || (activeGroup.workflowMode ?? "parallel") === mode) return;
      await callCommand("update_group_workflow", {
        groupId: activeGroup.id,
        workflowMode: mode,
        maxRelayHops: activeGroup.maxRelayHops ?? 2,
      });
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-relay-hops]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none" || !state.activeGroupId) return;
      const hops = Number(btn.dataset.relayHops ?? "2");
      const activeGroup = state.groupChats.find((g) => g.id === state.activeGroupId);
      if (!activeGroup || !Number.isFinite(hops) || (activeGroup.maxRelayHops ?? 2) === hops) {
        return;
      }
      await callCommand("update_group_workflow", {
        groupId: activeGroup.id,
        workflowMode: activeGroup.workflowMode ?? "a2a_relay",
        maxRelayHops: hops,
      });
    });
  });

  document.querySelectorAll<HTMLElement>("[data-jump-provider]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "none") return;
      const pid = btn.dataset.jumpProvider;
      const uuid = btn.dataset.jumpUuid;
      if (!pid || !uuid) return;
      viewMode = "provider";
      const alreadyWarm = state.warmAccounts.some(
        (k) => k.providerId === pid && k.uuid === uuid
      );
      await callCommand(
        "switch_account",
        { providerId: pid, uuid },
        alreadyWarm ? null : { providerId: pid, uuid }
      );
    });
  });

  document.querySelector('[data-group-target="all"]')?.addEventListener("click", () => {
    if (!state.activeGroupId) return;
    groupTargets[state.activeGroupId] = [];
    groupAutoChannels[state.activeGroupId] = [];
    rerenderPreservingScroll();
  });

  document.querySelectorAll<HTMLElement>("[data-group-target-auto]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const gid = state.activeGroupId;
      const catId = btn.dataset.groupTargetAuto;
      if (!gid || !catId) return;
      const currentAuto = [...(groupAutoChannels[gid] ?? [])];
      const idx = currentAuto.indexOf(catId);
      if (idx >= 0) {
        currentAuto.splice(idx, 1);
      } else {
        currentAuto.push(catId);
        // Same-channel mutual exclusion: enabling channel auto clears any @account in that channel
        groupTargets[gid] = (groupTargets[gid] ?? []).filter(
          (t) => resolveMemberCategory(state, t).id !== catId
        );
      }
      groupAutoChannels[gid] = currentAuto;
      rerenderPreservingScroll();
    });
  });

  document.querySelectorAll<HTMLElement>("[data-group-target-cat]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const gid = state.activeGroupId;
      const catId = btn.dataset.groupTargetCat;
      if (!gid || !catId) return;
      const activeGroup = state.groupChats.find((g) => g.id === gid);
      if (!activeGroup) return;
      const catMembers = activeGroup.members.filter((m) => {
        const meta = resolveMemberMeta(state, m);
        return !meta.loggedOut && resolveMemberCategory(state, m).id === catId;
      });
      if (catMembers.length === 0) return;
      const current = [...(groupTargets[gid] ?? [])];
      const allSelected = catMembers.every((cm) =>
        current.some((cur) => sameAccountKey(cur, cm))
      );
      if (allSelected) {
        groupTargets[gid] = current.filter(
          (cur) => !catMembers.some((cm) => sameAccountKey(cur, cm))
        );
      } else {
        for (const cm of catMembers) {
          if (!current.some((cur) => sameAccountKey(cur, cm))) {
            current.push({ providerId: cm.providerId, uuid: cm.uuid });
          }
        }
        groupTargets[gid] = current;
        groupAutoChannels[gid] = (groupAutoChannels[gid] ?? []).filter((c) => c !== catId);
      }
      rerenderPreservingScroll();
    });
  });

  document.querySelectorAll<HTMLElement>("[data-group-target-provider]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const gid = state.activeGroupId;
      const pid = btn.dataset.groupTargetProvider;
      const uuid = btn.dataset.groupTargetUuid;
      if (!gid || !pid || !uuid) return;
      const current = [...(groupTargets[gid] ?? [])];
      const idx = current.findIndex((k) => k.providerId === pid && k.uuid === uuid);
      if (idx >= 0) {
        current.splice(idx, 1);
      } else {
        current.push({ providerId: pid, uuid });
        const catId = resolveMemberCategory(state, { providerId: pid, uuid }).id;
        groupAutoChannels[gid] = (groupAutoChannels[gid] ?? []).filter((c) => c !== catId);
      }
      groupTargets[gid] = current;
      rerenderPreservingScroll();
    });
  });

  const submitGroupMessage = async () => {
    const gid = state.activeGroupId;
    if (!gid || groupSending) return;
    const raw = (groupDrafts[gid] ?? "").trim();
    if (!raw) return;
    const group = state.groupChats.find((g) => g.id === gid);
    const chosen = (groupTargets[gid] ?? []).filter((t) =>
      group?.members.some((m) => sameAccountKey(m, t))
    );
    const membersForMention = (group?.members ?? []).map((m) => ({
      key: m,
      alias: resolveMemberMeta(state, m).alias,
    }));
    const inline = extractComposerAtMentions(raw, membersForMention);
    let chosenAutoChannels: string[] = [];
    if (!inline.isAll) {
      const explicitForConflict = inline.hasMention ? inline.targets : chosen;
      const conflictCats = new Set(
        explicitForConflict.map((t) => resolveMemberCategory(state, t).id)
      );
      chosenAutoChannels = (groupAutoChannels[gid] ?? []).filter(
        (catId) =>
          !conflictCats.has(catId) &&
          (group?.members ?? []).some((m) => {
            const meta = resolveMemberMeta(state, m);
            return !meta.loggedOut && resolveMemberCategory(state, m).id === catId;
          })
      );
      groupAutoChannels[gid] = chosenAutoChannels;
    }
    groupSending = true;
    forceGroupScrollBottom = true;
    groupDrafts[gid] = "";
    render();
    restoreScrollPosition();
    let ok = false;
    try {
      ok = await callCommand("send_group_message", {
        groupId: gid,
        content: raw,
        targetMembers: chosen.length > 0 ? chosen : null,
        autoChannels: chosenAutoChannels.length > 0 ? chosenAutoChannels : null,
      });
    } finally {
      groupSending = false;
      if (!ok && !groupDrafts[gid]) {
        groupDrafts[gid] = raw;
      }
      render();
      restoreScrollPosition();
    }
  };

  const groupInputEl = document.querySelector<HTMLTextAreaElement>("#group-chat-input");
  if (groupInputEl && state.activeGroupId) {
    const activeGid = state.activeGroupId;
    const activeGroup = state.groupChats.find((g) => g.id === activeGid);
    const mentionMenuEl = document.querySelector<HTMLElement>("#group-mention-menu");
    const targetHintEl = document.querySelector<HTMLElement>("#group-target-hint");

    type MentionCandidate = { label: string; sub: string; insert: string; groupLabel?: string };
    const composerMembers = (activeGroup?.members ?? []).map((m) => ({
      key: m,
      alias: resolveMemberMeta(state, m).alias,
      categoryId: resolveMemberCategory(state, m).id,
    }));
    const activeWorkflowMode: GroupWorkflowMode = activeGroup?.workflowMode ?? "parallel";
    const activeRelayHops = activeGroup?.maxRelayHops ?? 2;
    const composerCategories = activeGroup
      ? groupByMemberCategory(
          state,
          activeGroup.members
            .filter((m) => !resolveMemberMeta(state, m).loggedOut)
            .map((m) => ({ key: m }))
        )
      : [];
    const mentionCandidates: MentionCandidate[] = activeGroup
      ? [
          {
            label: "全体成员",
            sub: "向全部有效成员转发",
            insert: "全体成员",
          },
          ...groupByMemberCategory(
            state,
            activeGroup.members.map((m) => ({ key: m }))
          ).flatMap((cat) =>
            cat.items.map(({ key: m }) => {
              const meta = resolveMemberMeta(state, m);
              const roleText = (activeGroup.memberRoles ?? []).find((r) =>
                sameAccountKey(r.member, m)
              )?.role;
              return {
                label: meta.alias,
                sub: meta.loggedOut
                  ? `${cat.label} · 未登录`
                  : meta.pageError
                    ? `${cat.label} · 状态异常`
                    : roleText
                      ? `${cat.label} · ${roleText}`
                      : cat.label,
                insert: meta.alias,
                groupLabel: cat.label,
              };
            })
          ),
        ]
      : [];

    let filteredMentions: MentionCandidate[] = [];
    let activeMentionStart = -1;
    let activeMentionIdx = 0;

    const findActiveAtQuery = (
      val: string,
      cursor: number
    ): { start: number; query: string } | null => {
      const before = val.slice(0, cursor);
      for (let i = before.length - 1; i >= 0; i -= 1) {
        const ch = before[i];
        if (/\s/.test(ch)) return null;
        if (ch === "@") {
          if (i > 0 && isAsciiWordChar(before[i - 1])) return null;
          return { start: i, query: before.slice(i + 1) };
        }
      }
      return null;
    };

    const syncInlineMentionAutoExclusion = () => {
      if (!activeGroup) return;
      const inline = extractComposerAtMentions(groupInputEl.value, composerMembers);
      if (!inline.hasMention) return;
      const prevAuto = groupAutoChannels[activeGid] ?? [];
      if (prevAuto.length === 0) return;

      let nextAuto = prevAuto;
      if (inline.isAll) {
        nextAuto = [];
      } else {
        const conflictCats = new Set(
          inline.targets.map((t) => resolveMemberCategory(state, t).id)
        );
        nextAuto = prevAuto.filter((catId) => !conflictCats.has(catId));
      }

      if (nextAuto.length !== prevAuto.length) {
        groupAutoChannels[activeGid] = nextAuto;
        const removed = prevAuto.filter((c) => !nextAuto.includes(c));
        for (const catId of removed) {
          document
            .querySelector(`[data-group-target-auto="${CSS.escape(catId)}"]`)
            ?.classList.remove("active");
          const catInfo = composerCategories.find((c) => c.id === catId);
          const allSelected = Boolean(
            catInfo &&
              catInfo.items.length > 0 &&
              catInfo.items.every((m) =>
                (groupTargets[activeGid] ?? []).some((t) => sameAccountKey(t, m.key))
              )
          );
          if (!allSelected) {
            document
              .querySelector(`[data-group-target-cat="${CSS.escape(catId)}"]`)
              ?.classList.remove("active");
          }
        }
        const validSelected = (groupTargets[activeGid] ?? []).filter((t) =>
          activeGroup.members.some((m) => sameAccountKey(m, t))
        );
        if (validSelected.length === 0 && nextAuto.length === 0) {
          document.querySelector('[data-group-target="all"]')?.classList.add("active");
        }
      }
    };

    const updateComposerHint = () => {
      if (!targetHintEl || !activeGroup) return;
      const validSelected = (groupTargets[activeGid] ?? []).filter((t) =>
        activeGroup.members.some((m) => sameAccountKey(m, t))
      );
      const validAutoLabels = (groupAutoChannels[activeGid] ?? [])
        .map((id) => {
          const cat = composerCategories.find((c) => c.id === id);
          return cat ? { id: cat.id, label: cat.label } : null;
        })
        .filter((x): x is { id: string; label: string } => Boolean(x));
      targetHintEl.textContent = formatComposerHintText(
        groupInputEl.value,
        validSelected,
        composerMembers,
        activeWorkflowMode,
        activeRelayHops,
        validAutoLabels
      );
    };

    const hideMentionMenu = () => {
      filteredMentions = [];
      activeMentionStart = -1;
      activeMentionIdx = 0;
      if (mentionMenuEl) {
        mentionMenuEl.hidden = true;
        mentionMenuEl.innerHTML = "";
      }
    };

    const applyMentionCandidate = (idx: number) => {
      const picked = filteredMentions[idx];
      if (!picked || activeMentionStart < 0) return;
      const cursor = groupInputEl.selectionStart ?? groupInputEl.value.length;
      const val = groupInputEl.value;
      const inserted = `@${picked.insert} `;
      const nextVal = val.slice(0, activeMentionStart) + inserted + val.slice(cursor);
      groupInputEl.value = nextVal;
      groupDrafts[activeGid] = nextVal;
      const nextCursor = activeMentionStart + inserted.length;
      groupInputEl.setSelectionRange(nextCursor, nextCursor);
      hideMentionMenu();
      syncInlineMentionAutoExclusion();
      updateComposerHint();
    };

    const renderMentionMenu = () => {
      if (!mentionMenuEl || filteredMentions.length === 0) {
        hideMentionMenu();
        return;
      }
      if (activeMentionIdx >= filteredMentions.length) activeMentionIdx = 0;
      if (activeMentionIdx < 0) activeMentionIdx = filteredMentions.length - 1;
      let prevGroup = "";
      mentionMenuEl.innerHTML = filteredMentions
        .map((c, i) => {
          let headerHtml = "";
          if (c.groupLabel && c.groupLabel !== prevGroup) {
            prevGroup = c.groupLabel;
            headerHtml = `<div class="group-mention-group-title">${escapeHtml(c.groupLabel)}</div>`;
          }
          return `
            ${headerHtml}
            <button
              class="group-mention-item ${i === activeMentionIdx ? "active" : ""}"
              data-mention-idx="${i}"
              type="button"
            >
              <span class="group-mention-item-label">@${escapeHtml(c.label)}</span>
              <span class="group-mention-item-sub">${escapeHtml(c.sub)}</span>
            </button>
          `;
        })
        .join("");
      mentionMenuEl.hidden = false;
      mentionMenuEl.querySelectorAll<HTMLButtonElement>("[data-mention-idx]").forEach((btn) => {
        btn.addEventListener("mousedown", (ev) => {
          ev.preventDefault();
          const idx = Number(btn.dataset.mentionIdx ?? "-1");
          if (idx >= 0) applyMentionCandidate(idx);
        });
      });
      mentionMenuEl
        .querySelector<HTMLElement>(".group-mention-item.active")
        ?.scrollIntoView({ block: "nearest" });
    };

    const syncMentionMenu = () => {
      if (!mentionMenuEl || mentionCandidates.length === 0) {
        hideMentionMenu();
        return;
      }
      const cursor = groupInputEl.selectionStart ?? groupInputEl.value.length;
      const activeAt = findActiveAtQuery(groupInputEl.value, cursor);
      if (!activeAt) {
        hideMentionMenu();
        return;
      }
      const q = activeAt.query.trim().toLowerCase();
      const matched = mentionCandidates.filter(
        (c) =>
          !q ||
          c.label.toLowerCase().includes(q) ||
          c.sub.toLowerCase().includes(q)
      );
      if (matched.length === 0) {
        hideMentionMenu();
        return;
      }
      activeMentionStart = activeAt.start;
      filteredMentions = matched;
      activeMentionIdx = 0;
      renderMentionMenu();
    };

    groupInputEl.addEventListener("input", () => {
      groupDrafts[activeGid] = groupInputEl.value;
      syncInlineMentionAutoExclusion();
      updateComposerHint();
      syncMentionMenu();
    });
    groupInputEl.addEventListener("click", () => {
      syncMentionMenu();
    });
    groupInputEl.addEventListener("blur", () => {
      window.setTimeout(hideMentionMenu, 120);
    });
    groupInputEl.addEventListener("keydown", (ev) => {
      if (mentionMenuEl && !mentionMenuEl.hidden && filteredMentions.length > 0) {
        if (ev.key === "ArrowDown") {
          ev.preventDefault();
          activeMentionIdx = (activeMentionIdx + 1) % filteredMentions.length;
          renderMentionMenu();
          return;
        }
        if (ev.key === "ArrowUp") {
          ev.preventDefault();
          activeMentionIdx =
            (activeMentionIdx - 1 + filteredMentions.length) % filteredMentions.length;
          renderMentionMenu();
          return;
        }
        if ((ev.key === "Enter" && !ev.isComposing) || ev.key === "Tab") {
          ev.preventDefault();
          applyMentionCandidate(activeMentionIdx);
          return;
        }
        if (ev.key === "Escape") {
          ev.preventDefault();
          ev.stopPropagation();
          hideMentionMenu();
          return;
        }
      }
      if (ev.key === "Enter" && !ev.isComposing && (!ev.shiftKey || ev.metaKey || ev.ctrlKey)) {
        ev.preventDefault();
        void submitGroupMessage();
      }
    });
  }

  document.querySelector("#group-send-btn")?.addEventListener("click", () => {
    void submitGroupMessage();
  });

  document.querySelector("#theme-toggle-btn")?.addEventListener("click", () => {
    cycleThemeMode();
  });

  document.querySelector("#settings-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "none") return;
    await setModal({
      kind: "settings",
      minimizeToTray: state.settings.minimizeToTray,
      globalShortcut: state.settings.globalShortcut,
      autoApprove: Boolean(state.settings.autoApprove),
    });
  });

  document.querySelector("#smart-switch-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "none" || state.conflicts.length > 0) return;
    const candidates = state.accounts.filter((a) => a.providerId === provider.id);
    if (candidates.length === 0) return;
    const ranked = [...candidates]
      .filter((a) => scoreAccountAvailability(a, findAccountPageError(state, a)).available)
      .sort((a, b) => compareAccountAvailability(a, b, state));
    const best = ranked[0];
    if (!best) {
      if (snapshot) {
        snapshot = { ...snapshot, notice: "暂无可用在线账号，请先登录或等待同步" };
        rerenderPreservingScroll();
      }
      return;
    }
    if (isSameKey(state.activeAccount, best)) {
      if (snapshot) {
        snapshot = {
          ...snapshot,
          notice: "当前账号已是可用余量最多的账号",
        };
        rerenderPreservingScroll();
      }
      return;
    }
    const alreadyWarm = state.warmAccounts.some(
      (k) => k.providerId === best.providerId && k.uuid === best.uuid
    );
    await callCommand(
      "switch_account",
      { providerId: best.providerId, uuid: best.uuid },
      alreadyWarm ? null : { providerId: best.providerId, uuid: best.uuid }
    );
  });

  document.querySelector("#sort-quota-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "none" || state.conflicts.length > 0) return;
    const candidates = state.accounts.filter((a) => a.providerId === provider.id);
    if (candidates.length <= 1) return;
    const sorted = [...candidates].sort((a, b) => compareAccountAvailability(a, b, state));
    const orderedUuids = sorted.map((a) => a.uuid);
    const changed = orderedUuids.some((u, idx) => u !== candidates[idx].uuid);
    if (!changed) {
      if (snapshot) {
        snapshot = {
          ...snapshot,
          notice: "当前已是按可用余量降序排列",
        };
        rerenderPreservingScroll();
      }
      return;
    }
    await callCommand("reorder_accounts", {
      providerId: provider.id,
      orderedUuids,
    });
  });

  const openOfficial = async () => {
    await callCommand("open_official_site", { providerId: provider.id });
  };
  document.querySelector("#open-site-btn")?.addEventListener("click", openOfficial);
  document.querySelector("#stage-open-site-btn")?.addEventListener("click", openOfficial);

  document.querySelector("#sync-all-btn")?.addEventListener("click", async () => {
    const btn = document.querySelector("#sync-all-btn");
    btn?.classList.add("spinning");
    try {
      await callCommand("trigger_quota_sync");
    } finally {
      setTimeout(() => btn?.classList.remove("spinning"), 650);
    }
  });

  const openAddModal = async () => {
    await setModal({ kind: "add", providerId: provider.id, draft: "", proxyDraft: "", timezoneDraft: "", cookieDraft: "" });
  };
  document.querySelector("#add-account-btn")?.addEventListener("click", openAddModal);
  document.querySelector("#stage-add-btn")?.addEventListener("click", openAddModal);

  document.querySelector("#dismiss-notice-btn")?.addEventListener("click", async () => {
    await callCommand("dismiss_notice");
  });

  const toastEl = document.querySelector("#download-toast");
  if (toastEl) {
    toastEl.addEventListener("mouseenter", () => {
      clearDownloadToastTimer();
    });
    toastEl.addEventListener("mouseleave", () => {
      scheduleDownloadToastDismiss();
    });
  }
  document.querySelector<HTMLButtonElement>("#download-reveal-btn")?.addEventListener("click", async (ev) => {
    const rawId = (ev.currentTarget as HTMLButtonElement).dataset.downloadId;
    const downloadId = rawId ? Number(rawId) : NaN;
    if (!Number.isFinite(downloadId)) return;
    try {
      await invoke<void>("reveal_downloaded_file", { downloadId });
    } catch (err) {
      if (snapshot) {
        snapshot = { ...snapshot, notice: String(err) };
        rerenderPreservingScroll();
      }
    }
  });
  document.querySelector("#download-close-btn")?.addEventListener("click", () => {
    clearDownloadToastTimer();
    downloadToast = null;
    rerenderPreservingScroll();
  });

  document.querySelector("#check-update-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "none") return;
    await setModal({ kind: "update" });
    const stale =
      !updateState.result ||
      (!updateState.result.hasUpdate && Date.now() - updateState.lastCheckedAt > 5 * 60 * 1000);
    if (stale && !updateState.checking && !updateState.installing) {
      await runCheckForUpdates(false);
    }
  });

  document.querySelector("#update-recheck-btn")?.addEventListener("click", async () => {
    await runCheckForUpdates(false);
  });

  document.querySelector("#update-install-btn")?.addEventListener("click", async () => {
    await runDownloadAndInstall();
  });

  document.querySelector("#update-open-page-btn")?.addEventListener("click", async () => {
    const pageUrl = updateState.result?.releasePageUrl;
    if (!pageUrl) return;
    try {
      await invoke("open_external_url", { url: pageUrl });
    } catch (err) {
      updateState.error = String(err);
      rerenderPreservingScroll();
    }
  });

  document.querySelectorAll<HTMLElement>("[data-resolve-uuid]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const uuid = btn.dataset.resolveUuid;
      const keepProviderId = btn.dataset.resolveProvider;
      if (uuid && keepProviderId) {
        await callCommand("resolve_uuid_conflict", { keepProviderId, uuid });
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-copy-cookie-uuid]").forEach((button) => {
    let feedbackTimer: number | undefined;
    button.addEventListener("click", async (event) => {
      event.stopPropagation();
      const uuid = button.dataset.copyCookieUuid;
      if (!uuid) return;
      const copyKey = `${provider.id}:${uuid}`;
      if (copyingCookies.has(copyKey)) return;
      copyingCookies.add(copyKey);
      window.clearTimeout(feedbackTimer);
      button.classList.remove("copied");
      button.dataset.copyLabel = "复制中…";
      try {
        await invoke<void>("copy_account_cookie", { providerId: provider.id, uuid });
        button.dataset.copyLabel = "已复制 Cookie";
        button.classList.add("copied");
        feedbackTimer = window.setTimeout(() => {
          button.classList.remove("copied");
          button.dataset.copyLabel = "复制 Cookie";
        }, 1800);
      } catch (err) {
        button.dataset.copyLabel = "复制 Cookie";
        if (snapshot?.selectedProviderId === provider.id) {
          snapshot = { ...snapshot, notice: String(err) };
          rerenderPreservingScroll();
        }
      } finally {
        copyingCookies.delete(copyKey);
      }
    });
  });

  document.querySelectorAll<HTMLElement>("[data-reload-uuid]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const uuid = btn.dataset.reloadUuid;
      if (!uuid) return;
      const isActive =
        snapshot?.activeAccount?.providerId === provider.id &&
        snapshot?.activeAccount?.uuid === uuid;
      const optimistic = isActive ? { providerId: provider.id, uuid } : null;
      await callCommand("reload_account", { providerId: provider.id, uuid }, optimistic);
    });
  });

  document.querySelectorAll<HTMLElement>("[data-relogin-uuid]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const uuid = btn.dataset.reloginUuid;
      if (!uuid) return;
      await callCommand(
        "clear_account_data",
        { providerId: provider.id, uuid },
        { providerId: provider.id, uuid }
      );
    });
  });

  document.querySelectorAll<HTMLElement>("[data-rename-uuid]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const uuid = btn.dataset.renameUuid;
      const acc = state.accounts.find((a) => a.providerId === provider.id && a.uuid === uuid);
      if (acc) {
        await setModal({
          kind: "rename",
          account: acc,
          draft: acc.autoAlias ? "" : acc.alias,
          proxyDraft: acc.proxyUrl ?? "",
          timezoneDraft: acc.timezone ?? "",
          cookieDraft: "",
        });
      }
    });
  });

  document.querySelectorAll<HTMLElement>("[data-remove-uuid]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      const uuid = btn.dataset.removeUuid;
      const acc = state.accounts.find((a) => a.providerId === provider.id && a.uuid === uuid);
      if (acc) {
        await setModal({ kind: "remove", account: acc });
      }
    });
  });

  document.querySelectorAll<HTMLElement>(".row-ops").forEach((ops) => {
    ops.addEventListener("click", (ev) => {
      ev.stopPropagation();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-move-uuid]").forEach((btn) => {
    btn.addEventListener("click", async (ev) => {
      ev.stopPropagation();
      if (modal.kind !== "none" || state.conflicts.length > 0) return;
      const uuid = btn.dataset.moveUuid;
      const dir = btn.dataset.moveDir;
      if (!uuid || (dir !== "up" && dir !== "down")) return;
      const ordered = state.accounts
        .filter((a) => a.providerId === provider.id)
        .map((a) => a.uuid);
      const idx = ordered.indexOf(uuid);
      if (idx === -1) return;
      const targetIdx = dir === "up" ? idx - 1 : idx + 1;
      if (targetIdx < 0 || targetIdx >= ordered.length) return;
      [ordered[idx], ordered[targetIdx]] = [ordered[targetIdx], ordered[idx]];
      await callCommand("reorder_accounts", {
        providerId: provider.id,
        orderedUuids: ordered,
      });
    });
  });

  document.querySelectorAll<HTMLElement>("[data-switch-uuid]").forEach((row) => {
    row.addEventListener("dragstart", (ev) => {
      if (isMobilePlatform(state.platform) || modal.kind !== "none" || state.conflicts.length > 0 || busy) {
        ev.preventDefault();
        return;
      }
      const uuid = row.dataset.switchUuid;
      if (!uuid) {
        ev.preventDefault();
        return;
      }
      draggingAccountUuid = uuid;
      if (ev.dataTransfer) {
        ev.dataTransfer.effectAllowed = "move";
        ev.dataTransfer.setData("text/plain", uuid);
      }
      requestAnimationFrame(() => row.classList.add("dragging"));
    });

    row.addEventListener("dragover", (ev) => {
      if (!draggingAccountUuid) return;
      const targetUuid = row.dataset.switchUuid;
      if (!targetUuid || targetUuid === draggingAccountUuid) return;
      ev.preventDefault();
      if (ev.dataTransfer) {
        ev.dataTransfer.dropEffect = "move";
      }
      const rect = row.getBoundingClientRect();
      const insertBefore = ev.clientY < rect.top + rect.height / 2;
      row.classList.toggle("drag-over-top", insertBefore);
      row.classList.toggle("drag-over-bottom", !insertBefore);
    });

    row.addEventListener("dragleave", (ev) => {
      if (row.contains(ev.relatedTarget as Node | null)) return;
      row.classList.remove("drag-over-top", "drag-over-bottom");
    });

    row.addEventListener("drop", async (ev) => {
      if (!draggingAccountUuid) return;
      ev.preventDefault();
      ev.stopPropagation();
      const srcUuid = draggingAccountUuid;
      const targetUuid = row.dataset.switchUuid;
      const insertBefore = row.classList.contains("drag-over-top");
      row.classList.remove("drag-over-top", "drag-over-bottom");
      if (!targetUuid || targetUuid === srcUuid) return;
      const ordered = state.accounts
        .filter((a) => a.providerId === provider.id)
        .map((a) => a.uuid);
      const fromIdx = ordered.indexOf(srcUuid);
      if (fromIdx === -1) return;
      ordered.splice(fromIdx, 1);
      const toIdx = ordered.indexOf(targetUuid);
      if (toIdx === -1) return;
      ordered.splice(insertBefore ? toIdx : toIdx + 1, 0, srcUuid);
      draggingAccountUuid = null;
      await callCommand("reorder_accounts", {
        providerId: provider.id,
        orderedUuids: ordered,
      });
    });

    row.addEventListener("dragend", () => {
      draggingAccountUuid = null;
      document
        .querySelectorAll<HTMLElement>(".account-row.dragging, .account-row.drag-over-top, .account-row.drag-over-bottom")
        .forEach((el) => {
          el.classList.remove("dragging", "drag-over-top", "drag-over-bottom");
        });
    });

    row.addEventListener("click", async () => {
      if (modal.kind !== "none" || state.conflicts.length > 0) return;
      const uuid = row.dataset.switchUuid;
      if (!uuid) return;
      if (
        state.activeAccount?.providerId === provider.id &&
        state.activeAccount?.uuid === uuid &&
        !isMobilePlatform(state.platform)
      ) {
        return;
      }
      const alreadyWarm = state.warmAccounts.some(
        (k) => k.providerId === provider.id && k.uuid === uuid
      );
      await callCommand(
        "switch_account",
        { providerId: provider.id, uuid },
        alreadyWarm ? null : { providerId: provider.id, uuid }
      );
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-shortcut-preset]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "settings" || modal.saving) return;
      modal.globalShortcut = btn.dataset.shortcutPreset ?? "";
      modal.error = undefined;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-timezone-preset]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if ((modal.kind !== "add" && modal.kind !== "rename") || modal.saving) return;
      modal.timezoneDraft = btn.dataset.timezonePreset ?? "";
      modal.error = undefined;
      render();
    });
  });

  document.querySelector("#proxy-test-btn")?.addEventListener("click", async () => {
    const currentModal = modal;
    if (currentModal.kind !== "add" && currentModal.kind !== "rename") return;
    if (currentModal.saving || currentModal.proxyTesting) return;
    const pid =
      currentModal.kind === "add" ? currentModal.providerId : currentModal.account.providerId;
    currentModal.proxyTesting = true;
    currentModal.proxyTestResult = null;
    currentModal.error = undefined;
    render();
    try {
      const res = await invoke<ProxyTestResult>("test_account_proxy", {
        providerId: pid,
        proxyUrl: currentModal.proxyDraft.trim(),
      });
      if (modal === currentModal) {
        currentModal.proxyTestResult = res;
      }
    } catch (err) {
      if (modal === currentModal) {
        currentModal.error = String(err);
      }
    } finally {
      currentModal.proxyTesting = false;
      if (modal === currentModal) {
        render();
      }
    }
  });

  document.querySelector("#modal-clear-data-btn")?.addEventListener("click", async () => {
    const currentModal = modal;
    if (currentModal.kind !== "rename" || currentModal.saving) return;
    if (!currentModal.confirmClear) {
      currentModal.confirmClear = true;
      render();
      return;
    }
    currentModal.error = undefined;
    currentModal.saving = true;
    render();
    let cleared = false;
    try {
      cleared = await callCommand("clear_account_data", {
        providerId: currentModal.account.providerId,
        uuid: currentModal.account.uuid,
      });
    } finally {
      currentModal.saving = false;
      if (modal === currentModal && !cleared) {
        currentModal.confirmClear = false;
        render();
      }
    }
    if (cleared && modal === currentModal) {
      await setModal({ kind: "none" });
    }
  });

  document.querySelector<HTMLInputElement>("#modal-tray-checkbox")?.addEventListener("change", (ev) => {
    if (modal.kind === "settings") {
      modal.minimizeToTray = (ev.currentTarget as HTMLInputElement).checked;
    }
  });

  document.querySelector<HTMLInputElement>("#modal-auto-approve-checkbox")?.addEventListener("change", (ev) => {
    if (modal.kind === "settings") {
      modal.autoApprove = (ev.currentTarget as HTMLInputElement).checked;
    }
  });

  document.querySelector<HTMLTextAreaElement>("#modal-cookie-import-input")?.addEventListener("input", (ev) => {
    if (modal.kind === "add" || modal.kind === "rename") {
      modal.cookieDraft = (ev.currentTarget as HTMLTextAreaElement).value;
      if (modal.error) {
        modal.error = undefined;
        document.querySelector("#modal-error-box")?.remove();
      }
    }
  });

  document.querySelector("#modal-import-cookie-btn")?.addEventListener("click", async () => {
    const currentModal = modal;
    if (currentModal.kind !== "rename" || currentModal.saving || currentModal.cookieImporting) return;
    const raw = (currentModal.cookieDraft ?? "").trim();
    if (!raw) {
      currentModal.error = "请先粘贴要导入的 Cookie 内容";
      render();
      return;
    }
    currentModal.error = undefined;
    currentModal.saving = true;
    currentModal.cookieImporting = true;
    render();
    let imported = false;
    try {
      imported = await callCommand("import_account_cookies", {
        providerId: currentModal.account.providerId,
        uuid: currentModal.account.uuid,
        rawCookies: raw,
      });
    } finally {
      currentModal.saving = false;
      currentModal.cookieImporting = false;
      if (modal === currentModal && !imported) {
        render();
      }
    }
    if (imported && modal === currentModal) {
      await setModal({ kind: "none" });
    }
  });

  document.querySelectorAll<HTMLButtonElement>("[data-toggle-group-member]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "group" || modal.saving) return;
      const pid = btn.dataset.memberProvider;
      const uuid = btn.dataset.memberUuid;
      if (!pid || !uuid) return;
      const idx = modal.selectedMembers.findIndex(
        (m) => m.providerId === pid && m.uuid === uuid
      );
      let nowChecked = false;
      if (idx >= 0) {
        modal.selectedMembers.splice(idx, 1);
        nowChecked = false;
      } else {
        if (modal.selectedMembers.length >= 30) {
          modal.error = "单个群聊最多支持 30 个成员";
          rerenderPreservingScroll();
          return;
        }
        modal.selectedMembers.push({ providerId: pid, uuid });
        nowChecked = true;
      }
      modal.error = undefined;
      btn.classList.toggle("selected", nowChecked);
      const checkBoxEl = btn.querySelector<HTMLElement>(".group-check-box");
      if (checkBoxEl) {
        checkBoxEl.textContent = nowChecked ? "✓" : "";
      }
      const hintEl = document.querySelector<HTMLElement>("#modal-group-hint");
      if (hintEl) {
        const extCount = (state.externalAgents ?? []).length;
        const eligibleCount =
          state.accounts.filter((a) => isAccountGroupEligible(state, a)).length + extCount;
        const ineligibleCount = state.accounts.length + extCount - eligibleCount;
        hintEl.textContent = formatGroupPickerHint(
          eligibleCount,
          ineligibleCount,
          modal.selectedMembers.length,
          true
        );
      }
      const confirmBtn = document.querySelector<HTMLButtonElement>("#modal-confirm-btn");
      if (confirmBtn) {
        confirmBtn.disabled = modal.selectedMembers.length === 0;
      }
      document.querySelector("#modal-error-box")?.remove();
    });
  });

  const commitKnowledgeSkillDraft = (
    kModal: Extract<ModalMode, { kind: "group-knowledge" }>
  ): boolean => {
    const name = kModal.skillNameDraft.trim();
    const desc = kModal.skillDescDraft.trim();
    const content = kModal.skillContentDraft.trim();
    if (!name || !content) {
      kModal.error = !name ? "请填写技能名称" : "请填写技能指令内容";
      return false;
    }
    if (kModal.editingSkillId) {
      const idx = kModal.skillsDraft.findIndex((s) => s.id === kModal.editingSkillId);
      if (idx >= 0) {
        kModal.skillsDraft[idx] = {
          ...kModal.skillsDraft[idx],
          name,
          description: desc,
          content,
        };
      }
    } else {
      if (kModal.skillsDraft.length >= 30) {
        kModal.error = "单个群聊最多支持 30 个共享技能";
        return false;
      }
      kModal.skillsDraft.push({
        id: `skill_${Date.now()}_${kModal.skillsDraft.length}`,
        name,
        description: desc,
        content,
        enabled: true,
      });
    }
    kModal.skillNameDraft = "";
    kModal.skillDescDraft = "";
    kModal.skillContentDraft = "";
    kModal.editingSkillId = null;
    kModal.error = undefined;
    return true;
  };

  document.querySelectorAll<HTMLButtonElement>("[data-knowledge-tab]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const nextTab = btn.dataset.knowledgeTab as
        | "context"
        | "skills"
        | "roles"
        | "adapters"
        | undefined;
      if (nextTab && modal.tab !== nextTab) {
        modal.tab = nextTab;
        modal.error = undefined;
        render();
        if (nextTab === "adapters" && !modal.adaptersLoaded) {
          const currentModal = modal;
          try {
            const snap = await invoke<SiteAdaptersSnapshot>("get_site_adapters");
            if (modal === currentModal) {
              currentModal.adaptersSnap = snap;
              currentModal.adaptersJsonDraft = snap.overrideJson ?? "";
              currentModal.adaptersLoaded = true;
              render();
            }
          } catch (err) {
            if (modal === currentModal) {
              currentModal.error = String(err);
              currentModal.adaptersLoaded = true;
              render();
            }
          }
        }
      }
    });
  });

  const contextInputEl = document.querySelector<HTMLTextAreaElement>("#modal-group-context-input");
  contextInputEl?.addEventListener("input", () => {
    if (modal.kind !== "group-knowledge") return;
    modal.contextDraft = contextInputEl.value;
    const countEl = document.querySelector<HTMLElement>("#modal-group-context-count");
    if (countEl) {
      countEl.textContent = String(contextInputEl.value.length);
    }
    if (modal.error) {
      modal.error = undefined;
      document.querySelector("#modal-error-box")?.remove();
    }
  });

  document.querySelectorAll<HTMLButtonElement>("[data-add-skill-preset]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const idx = Number(btn.dataset.addSkillPreset ?? "-1");
      const preset = GROUP_SKILL_PRESETS[idx];
      if (!preset) return;
      if (modal.skillsDraft.some((s) => s.name === preset.name)) return;
      if (modal.skillsDraft.length >= 30) {
        modal.error = "单个群聊最多支持 30 个共享技能";
        render();
        return;
      }
      modal.skillsDraft.push({
        id: `skill_${Date.now()}_${idx}`,
        name: preset.name,
        description: preset.description,
        content: preset.content,
        enabled: true,
      });
      modal.error = undefined;
      render();
    });
  });

  document.querySelectorAll<HTMLInputElement>("[data-toggle-skill-id]").forEach((cb) => {
    cb.addEventListener("change", () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const sid = cb.dataset.toggleSkillId;
      const target = modal.skillsDraft.find((s) => s.id === sid);
      if (target) {
        target.enabled = cb.checked;
        modal.error = undefined;
        render();
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-edit-skill-id]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const sid = btn.dataset.editSkillId;
      const target = modal.skillsDraft.find((s) => s.id === sid);
      if (!target) return;
      modal.editingSkillId = target.id;
      modal.skillNameDraft = target.name;
      modal.skillDescDraft = target.description;
      modal.skillContentDraft = target.content;
      modal.error = undefined;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-delete-skill-id]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const sid = btn.dataset.deleteSkillId;
      modal.skillsDraft = modal.skillsDraft.filter((s) => s.id !== sid);
      if (modal.editingSkillId === sid) {
        modal.editingSkillId = null;
        modal.skillNameDraft = "";
        modal.skillDescDraft = "";
        modal.skillContentDraft = "";
      }
      modal.error = undefined;
      render();
    });
  });

  document
    .querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
      "#modal-skill-name-input, #modal-skill-desc-input, #modal-skill-content-input"
    )
    .forEach((el) => {
      el.addEventListener("input", () => {
        if (modal.kind !== "group-knowledge") return;
        if (el.id === "modal-skill-name-input") modal.skillNameDraft = el.value;
        else if (el.id === "modal-skill-desc-input") modal.skillDescDraft = el.value;
        else modal.skillContentDraft = el.value;
        if (modal.error) {
          modal.error = undefined;
          document.querySelector("#modal-error-box")?.remove();
        }
      });
    });

  document.querySelector("#modal-skill-save-btn")?.addEventListener("click", () => {
    if (modal.kind !== "group-knowledge" || modal.saving) return;
    commitKnowledgeSkillDraft(modal);
    render();
  });

  document.querySelector("#modal-skill-cancel-edit-btn")?.addEventListener("click", () => {
    if (modal.kind !== "group-knowledge" || modal.saving) return;
    modal.editingSkillId = null;
    modal.skillNameDraft = "";
    modal.skillDescDraft = "";
    modal.skillContentDraft = "";
    modal.error = undefined;
    render();
  });

  document.querySelectorAll<HTMLInputElement>("[data-role-provider]").forEach((input) => {
    input.addEventListener("input", () => {
      if (modal.kind !== "group-knowledge") return;
      const pid = input.dataset.roleProvider;
      const uuid = input.dataset.roleUuid;
      if (!pid || !uuid) return;
      const key: AccountKey = { providerId: pid, uuid };
      const existing = modal.rolesDraft.find((r) => sameAccountKey(r.member, key));
      if (existing) {
        existing.role = input.value;
      } else {
        modal.rolesDraft.push({ member: key, role: input.value });
      }
      if (modal.error) {
        modal.error = undefined;
        document.querySelector("#modal-error-box")?.remove();
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-reset-thread-provider]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "group-knowledge" || modal.saving) return;
      const providerId = btn.dataset.resetThreadProvider;
      const uuid = btn.dataset.resetThreadUuid;
      if (!providerId || !uuid) return;
      const currentModal = modal;
      currentModal.saving = true;
      currentModal.error = undefined;
      render();
      try {
        await callCommand("reset_group_thread_binding", {
          groupId: currentModal.group.id,
          member: { providerId, uuid },
        });
      } finally {
        currentModal.saving = false;
        render();
      }
    });
  });

  const adaptersJsonEl = document.querySelector<HTMLTextAreaElement>("#modal-adapters-json-input");
  adaptersJsonEl?.addEventListener("input", () => {
    if (modal.kind !== "group-knowledge") return;
    modal.adaptersJsonDraft = adaptersJsonEl.value;
    if (modal.error) {
      modal.error = undefined;
      document.querySelector("#modal-error-box")?.remove();
    }
  });

  document.querySelector("#modal-adapters-reset-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "group-knowledge" || modal.saving) return;
    const currentModal = modal;
    currentModal.saving = true;
    currentModal.error = undefined;
    render();
    try {
      const snap = await invoke<SiteAdaptersSnapshot>("reset_site_adapter_overrides");
      if (modal === currentModal) {
        currentModal.adaptersSnap = snap;
        currentModal.adaptersJsonDraft = snap.overrideJson ?? "";
      }
    } catch (err) {
      if (modal === currentModal) {
        currentModal.error = String(err);
      }
    } finally {
      currentModal.saving = false;
      render();
    }
  });

  document.querySelectorAll<HTMLButtonElement>("[data-ext-kind]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "external-agents" || modal.saving) return;
      const nextKind = btn.dataset.extKind as ExternalAgentKind | undefined;
      if (nextKind && modal.formKind !== nextKind) {
        modal.formKind = nextKind;
        modal.error = undefined;
        render();
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-ext-preset-idx]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "external-agents" || modal.saving) return;
      const idx = Number(btn.dataset.extPresetIdx ?? "-1");
      const preset = EXTERNAL_AGENT_PRESETS[idx];
      if (!preset) return;
      modal.formKind = preset.kind;
      modal.nameDraft = preset.name;
      modal.baseUrlDraft = preset.baseUrl ?? "";
      modal.modelDraft = preset.model ?? "";
      modal.commandDraft = preset.command ?? "";
      modal.argsDraft = preset.args ?? "";
      modal.error = undefined;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-edit-ext-id]").forEach((btn) => {
    btn.addEventListener("click", () => {
      if (modal.kind !== "external-agents" || modal.saving) return;
      const eid = btn.dataset.editExtId;
      const ext = (state.externalAgents ?? []).find((a) => a.id === eid);
      if (!ext) return;
      modal.editingId = ext.id;
      modal.formKind = ext.kind;
      modal.nameDraft = ext.name;
      modal.baseUrlDraft = ext.baseUrl;
      modal.apiKeyDraft = ext.apiKey;
      modal.modelDraft = ext.model;
      modal.commandDraft = ext.command;
      modal.argsDraft = ext.args.join(" ");
      modal.workdirDraft = ext.workdir ?? "";
      modal.error = undefined;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-delete-ext-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      if (modal.kind !== "external-agents" || modal.saving) return;
      const eid = btn.dataset.deleteExtId;
      if (!eid) return;
      const currentModal = modal;
      currentModal.saving = true;
      currentModal.error = undefined;
      render();
      try {
        const deleted = await callCommand("delete_external_agent", { id: eid });
        if (deleted && modal === currentModal && currentModal.editingId === eid) {
          currentModal.editingId = null;
          currentModal.nameDraft = "";
          currentModal.baseUrlDraft = "";
          currentModal.apiKeyDraft = "";
          currentModal.modelDraft = "";
          currentModal.commandDraft = "";
          currentModal.argsDraft = "";
          currentModal.workdirDraft = "";
        }
      } finally {
        currentModal.saving = false;
        render();
      }
    });
  });

  document.querySelector("#ext-agent-cancel-edit-btn")?.addEventListener("click", () => {
    if (modal.kind !== "external-agents" || modal.saving) return;
    modal.editingId = null;
    modal.nameDraft = "";
    modal.baseUrlDraft = "";
    modal.apiKeyDraft = "";
    modal.modelDraft = "";
    modal.commandDraft = "";
    modal.argsDraft = "";
    modal.workdirDraft = "";
    modal.error = undefined;
    render();
  });

  document
    .querySelectorAll<HTMLInputElement>(
      "#ext-agent-name-input, #ext-agent-url-input, #ext-agent-key-input, #ext-agent-model-input, #ext-agent-cmd-input, #ext-agent-args-input, #ext-agent-workdir-input"
    )
    .forEach((input) => {
      input.addEventListener("input", () => {
        if (modal.kind !== "external-agents") return;
        if (input.id === "ext-agent-name-input") modal.nameDraft = input.value;
        else if (input.id === "ext-agent-url-input") modal.baseUrlDraft = input.value;
        else if (input.id === "ext-agent-key-input") modal.apiKeyDraft = input.value;
        else if (input.id === "ext-agent-model-input") modal.modelDraft = input.value;
        else if (input.id === "ext-agent-cmd-input") modal.commandDraft = input.value;
        else if (input.id === "ext-agent-args-input") modal.argsDraft = input.value;
        else if (input.id === "ext-agent-workdir-input") modal.workdirDraft = input.value;
        if (modal.error) {
          modal.error = undefined;
          document.querySelector("#modal-error-box")?.remove();
        }
      });
    });

  document.querySelector("#ext-agent-save-btn")?.addEventListener("click", async () => {
    if (modal.kind !== "external-agents" || modal.saving) return;
    const currentModal = modal;
    currentModal.saving = true;
    currentModal.error = undefined;
    render();
    try {
      const args = currentModal.argsDraft.trim()
        ? currentModal.argsDraft.trim().split(/\s+/)
        : [];
      const saved = await callCommand("upsert_external_agent", {
        id: currentModal.editingId,
        kind: currentModal.formKind,
        name: currentModal.nameDraft.trim(),
        baseUrl: currentModal.baseUrlDraft.trim(),
        apiKey: currentModal.apiKeyDraft.trim(),
        model: currentModal.modelDraft.trim(),
        command: currentModal.commandDraft.trim(),
        args,
        workdir: currentModal.workdirDraft.trim() || null,
      });
      if (saved && modal === currentModal) {
        currentModal.editingId = null;
        currentModal.nameDraft = "";
        currentModal.baseUrlDraft = "";
        currentModal.apiKeyDraft = "";
        currentModal.modelDraft = "";
        currentModal.commandDraft = "";
        currentModal.argsDraft = "";
        currentModal.workdirDraft = "";
      }
    } finally {
      currentModal.saving = false;
      render();
    }
  });

  document
    .querySelectorAll<HTMLInputElement>(
      "#modal-alias-input, #modal-proxy-input, #modal-timezone-input, #modal-shortcut-input, #modal-group-name-input"
    )
    .forEach((input) => {
      input.addEventListener("input", () => {
        if (modal.kind === "add" || modal.kind === "rename") {
          if (input.id === "modal-proxy-input") {
            modal.proxyDraft = input.value;
            modal.proxyTestResult = null;
            document.querySelector("#proxy-test-result")?.remove();
          } else if (input.id === "modal-timezone-input") {
            modal.timezoneDraft = input.value;
          } else {
            modal.draft = input.value;
          }
          if (modal.error) {
            modal.error = undefined;
            document.querySelector("#modal-error-box")?.remove();
          }
        } else if (modal.kind === "group") {
          modal.nameDraft = input.value;
          if (modal.error) {
            modal.error = undefined;
            document.querySelector("#modal-error-box")?.remove();
          }
        } else if (modal.kind === "settings") {
          modal.globalShortcut = input.value;
          if (modal.error) {
            modal.error = undefined;
            document.querySelector("#modal-error-box")?.remove();
          }
        }
      });
      input.addEventListener("keydown", async (ev) => {
        if (ev.key === "Enter" && !ev.isComposing) {
          ev.preventDefault();
          document.querySelector<HTMLButtonElement>("#modal-confirm-btn")?.click();
        } else if (ev.key === "Escape") {
          if (
            (modal.kind === "add" ||
              modal.kind === "rename" ||
              modal.kind === "group" ||
              modal.kind === "settings") &&
            modal.saving
          ) {
            return;
          }
          ev.preventDefault();
          ev.stopPropagation();
          await setModal({ kind: "none" });
        }
      });
    });

  document.querySelector("#modal-cancel-btn")?.addEventListener("click", async () => {
    if (modal.kind === "update" && updateState.installing) return;
    if (
      (modal.kind === "add" ||
        modal.kind === "rename" ||
        modal.kind === "group" ||
        modal.kind === "group-knowledge" ||
        modal.kind === "external-agents" ||
        modal.kind === "settings") &&
      modal.saving
    ) {
      return;
    }
    await setModal({ kind: "none" });
  });

  document.querySelector("#modal-confirm-btn")?.addEventListener("click", async () => {
    const currentModal = modal;
    if (currentModal.kind === "add" || currentModal.kind === "rename") {
      if (currentModal.saving) return;
      const supportsProxy = supportsAccountProxy(state.platform);
      const alias = currentModal.draft.trim();
      const proxyUrl = supportsProxy ? currentModal.proxyDraft.trim() : undefined;
      const timezone = currentModal.timezoneDraft.trim();
      currentModal.error = undefined;
      currentModal.saving = true;
      render();
      let saved = false;
      try {
        const rawCookies = (currentModal.cookieDraft ?? "").trim();
        saved = currentModal.kind === "add"
          ? await callCommand("add_account", {
            providerId: currentModal.providerId,
            alias,
            proxyUrl,
            timezone,
            rawCookies: rawCookies || undefined,
          })
          : await callCommand("rename_account", {
            providerId: currentModal.account.providerId,
            uuid: currentModal.account.uuid,
            alias,
            proxyUrl,
            timezone,
          });
      } finally {
        currentModal.saving = false;
        if (modal === currentModal && !saved) {
          render();
        }
      }
      if (saved && modal === currentModal) {
        await setModal({ kind: "none" });
      }
    } else if (currentModal.kind === "settings") {
      if (currentModal.saving) return;
      currentModal.error = undefined;
      currentModal.saving = true;
      render();
      let saved = false;
      try {
        saved = await callCommand("update_app_settings", {
          minimizeToTray: currentModal.minimizeToTray,
          globalShortcut: currentModal.globalShortcut.trim(),
          autoApprove: currentModal.autoApprove,
        });
      } finally {
        currentModal.saving = false;
        if (modal === currentModal && !saved) {
          render();
        }
      }
      if (saved && modal === currentModal) {
        await setModal({ kind: "none" });
      }
    } else if (currentModal.kind === "group") {
      if (currentModal.saving) return;
      currentModal.error = undefined;
      currentModal.saving = true;
      render();
      let saved = false;
      try {
        if (currentModal.group) {
          saved = await callCommand("update_group_chat", {
            groupId: currentModal.group.id,
            name: currentModal.nameDraft.trim(),
            members: currentModal.selectedMembers,
          });
        } else {
          viewMode = "groups";
          saved = await callCommand("create_group_chat", {
            name: currentModal.nameDraft.trim(),
            members: currentModal.selectedMembers,
          });
        }
      } finally {
        currentModal.saving = false;
        if (modal === currentModal && !saved) {
          render();
        }
      }
      if (saved && modal === currentModal) {
        await setModal({ kind: "none" });
      }
    } else if (currentModal.kind === "group-knowledge") {
      if (currentModal.saving) return;
      if (currentModal.tab === "adapters") {
        currentModal.saving = true;
        currentModal.error = undefined;
        render();
        let saved = false;
        try {
          const snap = await invoke<SiteAdaptersSnapshot>("save_site_adapter_overrides", {
            rawJson: currentModal.adaptersJsonDraft,
          });
          if (modal === currentModal) {
            currentModal.adaptersSnap = snap;
            currentModal.adaptersJsonDraft = snap.overrideJson ?? "";
            saved = true;
          }
        } catch (err) {
          if (modal === currentModal) {
            currentModal.error = String(err);
          }
        } finally {
          currentModal.saving = false;
          if (modal === currentModal && !saved) {
            render();
          }
        }
        if (saved && modal === currentModal) {
          await setModal({ kind: "none" });
        }
        return;
      }
      if (currentModal.skillNameDraft.trim() || currentModal.skillContentDraft.trim()) {
        if (!commitKnowledgeSkillDraft(currentModal)) {
          render();
          return;
        }
      }
      currentModal.saving = true;
      currentModal.error = undefined;
      render();
      let saved = false;
      try {
        saved = await callCommand("update_group_knowledge", {
          groupId: currentModal.group.id,
          sharedContext: currentModal.contextDraft.trim(),
          sharedSkills: currentModal.skillsDraft,
          memberRoles: currentModal.rolesDraft
            .map((r) => ({ member: r.member, role: r.role.trim() }))
            .filter((r) => r.role.length > 0),
        });
      } finally {
        currentModal.saving = false;
        if (modal === currentModal && !saved) {
          render();
        }
      }
      if (saved && modal === currentModal) {
        await setModal({ kind: "none" });
      }
    } else if (currentModal.kind === "delete-group") {
      await setModal({ kind: "none" });
      await callCommand("delete_group_chat", { groupId: currentModal.group.id });
    } else if (currentModal.kind === "remove") {
      await setModal({ kind: "none" });
      await callCommand("remove_account", {
        providerId: currentModal.account.providerId,
        uuid: currentModal.account.uuid,
      });
    }
  });
}

function groupModalVisualSignature(s: ManagerSnapshot): string {
  return JSON.stringify({
    selectedProviderId: s.selectedProviderId,
    activeGroupId: s.activeGroupId,
    notice: s.notice,
    conflicts: s.conflicts,
    providers: s.providers.map((p) => [p.id, p.enabled, p.shortName]),
    accounts: s.accounts.map((a) => [
      a.providerId,
      a.uuid,
      a.alias,
      a.authState,
      a.avatarUrl,
      a.cookieExpiresAtMs,
      a.cookieSessionOnly,
    ]),
    accountErrors: s.accountErrors,
    externalAgents: (s.externalAgents ?? []).map((e) => [
      e.id,
      e.kind,
      e.name,
      e.model,
      e.command,
    ]),
    groupChats: s.groupChats.map((g) => {
      const lastMsg = g.messages[g.messages.length - 1];
      return [
        g.id,
        g.name,
        g.members.length,
        g.messages.length,
        lastMsg ? [lastMsg.id, lastMsg.status, lastMsg.content.length] : null,
      ];
    }),
  });
}

async function bootstrap(): Promise<void> {
  await listen<ManagerSnapshot>("manager-state-changed", (event) => {
    const prevSnapshot = snapshot;
    const nextSnapshot = event.payload;
    if (
      modal.kind === "group" &&
      !modal.saving &&
      prevSnapshot &&
      groupModalVisualSignature(prevSnapshot) === groupModalVisualSignature(nextSnapshot)
    ) {
      snapshot = nextSnapshot;
      syncViewModeFromSnapshot(snapshot);
      if (snapshot.activeAccount || !snapshot.switchingAccount) {
        optimisticSwitching = null;
      }
      return;
    }
    saveScrollPosition();
    snapshot = nextSnapshot;
    syncViewModeFromSnapshot(snapshot);
    if (snapshot.activeAccount || !snapshot.switchingAccount) {
      optimisticSwitching = null;
    }
    render();
    restoreScrollPosition();
  });

  await listen<DownloadNoticeEvent>("download-event", (event) => {
    downloadToast = event.payload;
    clearDownloadToastTimer();
    rerenderPreservingScroll();
    scheduleDownloadToastDismiss();
  });

  await listen<UpdateProgressEvent>("update-progress", (event) => {
    const evt = event.payload;
    updateState.progress = evt;
    if (evt.stage === "error") {
      updateState.error = evt.message;
    }
    if (modal.kind !== "update") {
      return;
    }
    const barEl = document.querySelector<HTMLElement>("#update-progress-bar");
    const textEl = document.querySelector<HTMLElement>("#update-progress-text");
    const pctEl = document.querySelector<HTMLElement>("#update-progress-pct");
    if (barEl && textEl && pctEl && lastRenderedUpdateStage === evt.stage && evt.stage !== "error") {
      textEl.textContent = evt.message;
      pctEl.textContent = formatProgressSummary(evt);
      if (evt.percent === null) {
        barEl.classList.add("indeterminate");
        barEl.style.width = "";
      } else {
        barEl.classList.remove("indeterminate");
        barEl.style.width = `${Math.max(0, Math.min(100, evt.percent))}%`;
      }
      return;
    }
    lastRenderedUpdateStage = evt.stage;
    rerenderPreservingScroll();
  });

  window.addEventListener("keydown", async (ev) => {
    if (modal.kind !== "none") {
      if (
        ev.key === "Escape" &&
        !(modal.kind === "add" && modal.saving) &&
        !(modal.kind === "rename" && modal.saving) &&
        !(modal.kind === "group" && modal.saving) &&
        !(modal.kind === "group-knowledge" && modal.saving) &&
        !(modal.kind === "external-agents" && modal.saving) &&
        !(modal.kind === "settings" && modal.saving) &&
        !(modal.kind === "update" && updateState.installing)
      ) {
        ev.preventDefault();
        await setModal({ kind: "none" });
      }
      return;
    }
    const target = ev.target as HTMLElement | null;
    if (
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable)
    ) {
      return;
    }
    if ((ev.metaKey || ev.ctrlKey) && !ev.shiftKey && !ev.altKey) {
      const digit = parseInt(ev.key, 10);
      if (!isNaN(digit) && digit >= 1 && digit <= 9) {
        ev.preventDefault();
        if (!snapshot) return;
        const providerAccounts = snapshot.accounts.filter(
          (a) => a.providerId === snapshot!.selectedProviderId
        );
        const targetAcc = providerAccounts[digit - 1];
        if (targetAcc && !isSameKey(snapshot.activeAccount, targetAcc)) {
          const alreadyWarm = snapshot.warmAccounts.some(
            (k) =>
              k.providerId === targetAcc.providerId && k.uuid === targetAcc.uuid
          );
          await callCommand(
            "switch_account",
            { providerId: targetAcc.providerId, uuid: targetAcc.uuid },
            alreadyWarm ? null : { providerId: targetAcc.providerId, uuid: targetAcc.uuid }
          );
        }
        return;
      }
      if (
        ev.key === "ArrowUp" ||
        ev.key === "ArrowDown" ||
        ev.key === "[" ||
        ev.key === "]"
      ) {
        ev.preventDefault();
        if (!snapshot) return;
        const providerAccounts = snapshot.accounts.filter(
          (a) => a.providerId === snapshot!.selectedProviderId
        );
        if (providerAccounts.length <= 1) return;
        const curIdx = providerAccounts.findIndex((a) =>
          isSameKey(snapshot!.activeAccount, a)
        );
        const delta = ev.key === "ArrowUp" || ev.key === "[" ? -1 : 1;
        const nextIdx =
          (curIdx + delta + providerAccounts.length) % providerAccounts.length;
        const targetAcc = providerAccounts[nextIdx];
        if (targetAcc && !isSameKey(snapshot.activeAccount, targetAcc)) {
          const alreadyWarm = snapshot.warmAccounts.some(
            (k) =>
              k.providerId === targetAcc.providerId && k.uuid === targetAcc.uuid
          );
          await callCommand(
            "switch_account",
            { providerId: targetAcc.providerId, uuid: targetAcc.uuid },
            alreadyWarm ? null : { providerId: targetAcc.providerId, uuid: targetAcc.uuid }
          );
        }
      }
    }
  });

  snapshot = await invoke<ManagerSnapshot>("get_manager_state");
  syncViewModeFromSnapshot(snapshot);
  render();
  setTimeout(() => {
    void runCheckForUpdates(true);
  }, 3000);
  window.setInterval(() => {
    void maybeAutoCheckForUpdates();
  }, 30 * 60 * 1000);
  window.addEventListener("focus", () => {
    void maybeAutoCheckForUpdates();
  });
  window.addEventListener("online", () => {
    void maybeAutoCheckForUpdates();
  });
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") {
      void maybeAutoCheckForUpdates();
    }
  });
}

void bootstrap();
