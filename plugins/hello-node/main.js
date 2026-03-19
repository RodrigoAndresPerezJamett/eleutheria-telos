#!/usr/bin/env node
/**
 * hello-node — Reference Eleutheria Telos plugin (Node.js).
 *
 * Demonstrates the minimum structure required to build a Node plugin:
 *   - Reads env vars injected by the host app
 *   - Starts an HTTP server on the assigned port
 *   - Serves an HTMX UI fragment at GET /
 *   - Exposes a JSON API at GET /api/hello and POST /api/echo
 *   - Optionally calls back into the host app via ELEUTHERIA_APP_PORT + ELEUTHERIA_TOKEN
 *
 * Requires only Node.js stdlib — no npm packages needed.
 */

"use strict";

const http = require("node:http");
const { URL } = require("node:url");
const querystring = require("node:querystring");

// ── Configuration (injected by host) ──────────────────────────────────────────

const APP_PORT   = parseInt(process.env.ELEUTHERIA_APP_PORT   ?? "47821", 10);
const TOKEN      = process.env.ELEUTHERIA_TOKEN               ?? "";
const PLUGIN_ID  = process.env.ELEUTHERIA_PLUGIN_ID           ?? "hello-node";
const PLUGIN_PORT = parseInt(process.env.ELEUTHERIA_PLUGIN_PORT ?? "47901", 10);

const log = {
  info:  (...a) => console.log(`[${PLUGIN_ID}] INFO`,  ...a),
  warn:  (...a) => console.warn(`[${PLUGIN_ID}] WARN`,  ...a),
  debug: (...a) => {},  // suppress debug in production
};

// ── UI fragment ────────────────────────────────────────────────────────────────

const UI_HTML = `\
<div id="tool-panel" class="p-6 max-w-xl">
  <h2 class="text-xl font-semibold text-white mb-1">🟩 Hello Node</h2>
  <p class="text-gray-400 text-sm mb-6">
    Reference plugin — demonstrates the Eleutheria Telos plugin API.
  </p>

  <!-- Echo form -->
  <form class="space-y-3"
        hx-post="/plugins/hello-node/api/echo"
        hx-target="#echo-result"
        hx-swap="innerHTML">
    <label class="block text-sm text-gray-300">Send a message to the plugin:</label>
    <div class="flex gap-2">
      <input name="message"
             type="text"
             placeholder="Type something…"
             class="flex-1 bg-gray-700 text-white rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-green-500" />
      <button type="submit"
              class="bg-green-600 hover:bg-green-500 text-white text-sm px-4 py-2 rounded transition-colors">
        Echo
      </button>
    </div>
  </form>

  <div id="echo-result" class="mt-4 text-sm text-gray-300"></div>

  <!-- Info from plugin JSON API -->
  <div class="mt-8 border-t border-gray-700 pt-4">
    <button class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
            hx-get="/plugins/hello-node/api/hello"
            hx-target="#hello-result"
            hx-swap="innerHTML">
      Fetch plugin info
    </button>
    <div id="hello-result" class="mt-2 text-xs text-gray-400 font-mono"></div>
  </div>
</div>
`;

// ── Helpers ────────────────────────────────────────────────────────────────────

function htmlEscape(str) {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function send(res, status, contentType, body) {
  const buf = Buffer.from(body, "utf8");
  res.writeHead(status, {
    "Content-Type": contentType,
    "Content-Length": buf.length,
  });
  res.end(buf);
}

function readBody(req) {
  return new Promise((resolve) => {
    const chunks = [];
    req.on("data", (c) => chunks.push(c));
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
  });
}

/** GET request to the host app with Bearer auth. Returns parsed JSON or null. */
function callHostApi(path) {
  return new Promise((resolve) => {
    const options = {
      hostname: "127.0.0.1",
      port: APP_PORT,
      path,
      method: "GET",
      headers: { Authorization: `Bearer ${TOKEN}` },
    };
    const req = http.request(options, (res) => {
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => {
        try {
          resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
        } catch {
          resolve(null);
        }
      });
    });
    req.on("error", (e) => {
      log.warn(`Host API call failed (${path}):`, e.message);
      resolve(null);
    });
    req.setTimeout(3000, () => { req.destroy(); resolve(null); });
    req.end();
  });
}

// ── Request router ─────────────────────────────────────────────────────────────

async function handleRequest(req, res) {
  const parsedUrl = new URL(req.url, `http://127.0.0.1:${PLUGIN_PORT}`);
  const path = parsedUrl.pathname.replace(/\/$/, "") || "/";

  // GET /  or  GET /plugins/hello-node
  if (req.method === "GET" && (path === "/" || path === "/plugins/hello-node")) {
    send(res, 200, "text/html; charset=utf-8", UI_HTML);
    return;
  }

  // GET /api/hello  or  GET /plugins/hello-node/api/hello
  if (req.method === "GET" && (path === "/api/hello" || path === "/plugins/hello-node/api/hello")) {
    const hostData = await callHostApi("/api/clipboard?limit=1");
    const data = {
      plugin_id:    PLUGIN_ID,
      plugin_port:  PLUGIN_PORT,
      app_port:     APP_PORT,
      node_version: process.version,
      host_reachable: hostData !== null,
    };
    const html = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
    send(res, 200, "text/html; charset=utf-8", html);
    return;
  }

  // POST /api/echo  or  POST /plugins/hello-node/api/echo
  if (req.method === "POST" && (path === "/api/echo" || path === "/plugins/hello-node/api/echo")) {
    const raw = await readBody(req);
    const form = querystring.parse(raw);
    const message = (form.message ?? "").trim();

    const html = message
      ? `<p class="text-green-400">Plugin echoes: <span class="font-mono">${htmlEscape(message)}</span></p>`
      : `<p class="text-gray-500">Nothing to echo.</p>`;
    send(res, 200, "text/html; charset=utf-8", html);
    return;
  }

  send(res, 404, "application/json", JSON.stringify({ error: "not found" }));
}

// ── Entry point ────────────────────────────────────────────────────────────────

const server = http.createServer((req, res) => {
  handleRequest(req, res).catch((err) => {
    log.warn("Unhandled error:", err);
    send(res, 500, "application/json", JSON.stringify({ error: "internal error" }));
  });
});

server.listen(PLUGIN_PORT, "127.0.0.1", () => {
  log.info(`Listening on http://127.0.0.1:${PLUGIN_PORT}`);
});

process.on("SIGTERM", () => {
  log.info("Shutting down");
  server.close(() => process.exit(0));
});
