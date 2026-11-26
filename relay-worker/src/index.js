const MAX_BODY_BYTES = 2 * 1024 * 1024;
const HEX_ID_RE = /^[a-f0-9]{24,64}$/;
const SHA256_HEX_RE = /^[a-f0-9]{64}$/;

const BASE_HEADERS = {
  "Cache-Control": "no-store",
  "X-Content-Type-Options": "nosniff",
  "Content-Type": "application/json; charset=utf-8",
};

function jsonResp(status, body) {
  return new Response(JSON.stringify(body), {
    status,
    headers: BASE_HEADERS,
  });
}

async function sha256Hex(raw) {
  const bytes = new TextEncoder().encode(raw);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, "0")).join("");
}

function timingSafeEqualHex(a, b) {
  if (typeof a !== "string" || typeof b !== "string" || a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

export class RelayDropDO {
  constructor(state) {
    this.state = state;
  }

  async alarm() {
    await this.state.storage.deleteAll();
  }

  async fetch(request) {
    const method = request.method;
    if (method === "POST") {
      const existing = await this.state.storage.get("drop");
      if (existing) {
        return jsonResp(409, { error: "already_exists" });
      }
      const payload = await request.json();
      const now = Date.now();
      const expiresAtMs = now + payload.ttlSecs * 1000;
      const drop = {
        nonce: payload.nonce,
        ciphertext: payload.ciphertext,
        tokenHash: payload.tokenHash,
        deleteTokenHash: payload.deleteTokenHash,
        remaining: payload.maxDownloads,
        badAttempts: 0,
        expiresAtMs,
      };
      await this.state.storage.put("drop", drop);
      await this.state.storage.setAlarm(expiresAtMs);
      return jsonResp(201, { ok: true, expiresAtMs });
    }

    if (method === "GET") {
      const drop = await this.state.storage.get("drop");
      if (!drop || Date.now() > drop.expiresAtMs) {
        await this.state.storage.deleteAll();
        await this.state.storage.deleteAlarm();
        return jsonResp(404, { error: "not_found" });
      }
      const token = (request.headers.get("x-relay-token") || "").trim();
      if (!token || token.length > 256) {
        return jsonResp(403, { error: "forbidden" });
      }
      const gotHash = await sha256Hex(token);
      if (!timingSafeEqualHex(gotHash, drop.tokenHash)) {
        drop.badAttempts += 1;
        if (drop.badAttempts >= 5) {
          await this.state.storage.deleteAll();
          await this.state.storage.deleteAlarm();
        } else {
          await this.state.storage.put("drop", drop);
        }
        return jsonResp(403, { error: "forbidden" });
      }
      const remaining = drop.remaining - 1;
      const out = { nonce: drop.nonce, ciphertext: drop.ciphertext };
      if (remaining <= 0) {
        await this.state.storage.deleteAll();
        await this.state.storage.deleteAlarm();
      } else {
        drop.remaining = remaining;
        await this.state.storage.put("drop", drop);
      }
      return jsonResp(200, out);
    }

    if (method === "DELETE") {
      const drop = await this.state.storage.get("drop");
      if (!drop) {
        return jsonResp(404, { error: "not_found" });
      }
      const token = (request.headers.get("x-relay-token") || "").trim();
      if (!token || token.length > 256) {
        return jsonResp(403, { error: "forbidden" });
      }
      const gotHash = await sha256Hex(token);
      if (!timingSafeEqualHex(gotHash, drop.deleteTokenHash)) {
        return jsonResp(403, { error: "forbidden" });
      }
      await this.state.storage.deleteAll();
      await this.state.storage.deleteAlarm();
      return jsonResp(200, { deleted: true });
    }

    return jsonResp(405, { error: "method_not_allowed" });
  }
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (request.method === "GET" && path === "/health") {
      return jsonResp(200, { ok: true });
    }

    if (request.method === "POST" && path === "/v1/drop") {
      const contentLen = Number(request.headers.get("content-length") || "0");
      if (contentLen > MAX_BODY_BYTES) {
        return jsonResp(413, { error: "payload_too_large" });
      }
      const raw = await request.text();
      if (new TextEncoder().encode(raw).length > MAX_BODY_BYTES) {
        return jsonResp(413, { error: "payload_too_large" });
      }
      let data;
      try {
        data = JSON.parse(raw);
      } catch {
        return jsonResp(400, { error: "invalid_json" });
      }
      if (
        !data ||
        typeof data.id !== "string" ||
        !HEX_ID_RE.test(data.id) ||
        typeof data.nonce !== "string" ||
        data.nonce.length === 0 ||
        data.nonce.length > 64 ||
        typeof data.ciphertext !== "string" ||
        data.ciphertext.length === 0 ||
        data.ciphertext.length > MAX_BODY_BYTES ||
        typeof data.tokenHash !== "string" ||
        !SHA256_HEX_RE.test(data.tokenHash) ||
        typeof data.deleteTokenHash !== "string" ||
        !SHA256_HEX_RE.test(data.deleteTokenHash)
      ) {
        return jsonResp(400, { error: "invalid_fields" });
      }
      const rawTtl = Number.isFinite(data.ttlSecs) ? Math.trunc(data.ttlSecs) : 180;
      const rawMax = Number.isFinite(data.maxDownloads) ? Math.trunc(data.maxDownloads) : 3;
      const ttlSecs = Math.min(300, Math.max(30, rawTtl));
      const maxDownloads = Math.min(3, Math.max(1, rawMax));
      const stub = env.RELAY_DROPS.get(env.RELAY_DROPS.idFromName(data.id));
      return stub.fetch("https://relay.internal/drop", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          nonce: data.nonce,
          ciphertext: data.ciphertext,
          tokenHash: data.tokenHash,
          deleteTokenHash: data.deleteTokenHash,
          ttlSecs,
          maxDownloads,
        }),
      });
    }

    if (path.startsWith("/v1/drop/") && (request.method === "GET" || request.method === "DELETE")) {
      const id = path.slice("/v1/drop/".length);
      if (!HEX_ID_RE.test(id)) {
        return jsonResp(400, { error: "invalid_id" });
      }
      const token = (request.headers.get("x-relay-token") || "").trim();
      if (!token || token.length > 256) {
        return jsonResp(403, { error: "forbidden" });
      }
      const stub = env.RELAY_DROPS.get(env.RELAY_DROPS.idFromName(id));
      return stub.fetch("https://relay.internal/drop", {
        method: request.method,
        headers: { "x-relay-token": token },
      });
    }

    return jsonResp(404, { error: "not_found" });
  },
};
