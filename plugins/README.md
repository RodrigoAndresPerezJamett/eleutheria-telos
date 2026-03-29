# Eleutheria Telos — Plugin Developer Guide

Plugins are independent HTTP servers that run alongside the app. The host proxies requests through Axum, enforces route permissions, and injects sidebar entries automatically.

---

## Quick start

```
plugins/
  your-plugin/
    manifest.json   ← required
    main.py         ← or main.js, or a binary
```

Copy `hello-python/` or `hello-node/` as your starting point, then edit `manifest.json`.

---

## Manifest (`manifest.json`)

```json
{
  "id":              "my-plugin",
  "name":            "My Plugin",
  "version":         "0.1.0",
  "author":          "Your Name",
  "description":     "What this plugin does.",
  "runtime":         "python",
  "entry":           "main.py",
  "min_app_version": "0.1.0",
  "icon":            "🔌",
  "routes": [
    "/plugins/my-plugin"
  ],
  "sidebar": {
    "show":  true,
    "label": "My Plugin",
    "order": 200
  }
}
```

### Available permissions

| Permission | What it grants |
|-----------|---------------|
| `db.read` | Read your own rows in `plugin_data` |
| `db.write` | Write your own rows in `plugin_data` (50MB quota — see below) |
| `clipboard.read` | Read clipboard history |
| `clipboard.write` | Write to the system clipboard |
| `event_bus.subscribe` | Listen to app events (OCR completed, note created, etc.) |
| `event_bus.publish` | Publish events to the app event bus |
| `fs.user_dir` | Read/write inside `~/eleutheria/plugins/{id}/` only. Path traversal (`../`) is rejected. |
| `ocr.invoke` | Call the OCR tool programmatically |
| `tts.invoke` | Call Voice-to-text programmatically |
| `translate.invoke` | Call the translation tool programmatically |
| `notifications.show` | Show system notifications |
| `network.outbound` | Make outbound HTTP requests (declare allowed domains in `network_allowlist`) |

Undeclared permissions are enforced at the proxy layer — attempting an operation without the required permission returns:
```json
{ "error": "permission_denied", "required": "permission_name" }
```

#### Storage quota (`db.write`)
Plugin data is limited to **50MB per plugin**. Attempting to write beyond the limit returns:
```json
{ "error": "storage_quota_exceeded", "used_bytes": N, "limit_bytes": 52428800 }
```
Check your quota proactively: `GET /api/db/plugin/quota` → `{ "used_bytes": N, "limit_bytes": 52428800 }`. For large binary files, use `fs.user_dir` instead.

---

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Unique slug. Must match the folder name under `plugins/`. Used in all URLs. |
| `name` | yes | Human-readable name shown in sidebar and plugin list. |
| `version` | yes | Semver string. |
| `author` | yes | Author name or org. |
| `description` | yes | One-line description. |
| `runtime` | yes | `"python"`, `"node"`, or `"binary"`. |
| `entry` | yes | Entry point filename (relative to plugin directory). |
| `min_app_version` | no | Minimum app version required to run this plugin. |
| `icon` | no | Emoji or single character shown in sidebar. Default: `🔌`. |
| `routes` | no | List of URL prefixes the plugin is allowed to handle (see Permissions). |
| `sidebar` | no | If omitted or `show: false`, no sidebar entry is created. |

### `sidebar` object

| Field | Required | Description |
|-------|----------|-------------|
| `show` | yes | `true` to add a sidebar entry. |
| `label` | yes | Text shown next to the icon in the desktop sidebar. |
| `order` | no | Sort position (ascending). Plugins appear after built-in tools. Lower numbers appear first. Default: `4294967295` (last). |

---

## Runtimes

| `runtime` value | Command used to start the plugin |
|-----------------|----------------------------------|
| `"python"` | `python3 <entry>` |
| `"node"` | `node <entry>` |
| `"binary"` | `./<entry>` (executable in the plugin directory) |

The process is started with its **current directory set to the plugin folder** (`plugins/<id>/`).

---

## Environment variables

The host injects these env vars before starting your process:

| Variable | Description |
|----------|-------------|
| `ELEUTHERIA_APP_PORT` | Port the Axum HTTP server is listening on. Use this to call back into the host API. |
| `ELEUTHERIA_TOKEN` | Session token. Include as `Authorization: Bearer <token>` in all host API calls. |
| `ELEUTHERIA_PLUGIN_ID` | Your plugin's `id` from `manifest.json`. |
| `ELEUTHERIA_PLUGIN_PORT` | The port **your plugin must listen on**. Assigned by the host at startup. |

```python
# Python example
import os
APP_PORT   = int(os.environ.get("ELEUTHERIA_APP_PORT", "47821"))
TOKEN      = os.environ.get("ELEUTHERIA_TOKEN", "")
PLUGIN_ID  = os.environ.get("ELEUTHERIA_PLUGIN_ID", "my-plugin")
PLUGIN_PORT = int(os.environ.get("ELEUTHERIA_PLUGIN_PORT", "47901"))
```

```js
// Node.js example
const APP_PORT    = parseInt(process.env.ELEUTHERIA_APP_PORT ?? "47821", 10);
const TOKEN       = process.env.ELEUTHERIA_TOKEN ?? "";
const PLUGIN_ID   = process.env.ELEUTHERIA_PLUGIN_ID ?? "my-plugin";
const PLUGIN_PORT = parseInt(process.env.ELEUTHERIA_PLUGIN_PORT ?? "47901", 10);
```

---

## Routing and permissions

Your plugin must:
1. **Listen on `127.0.0.1:ELEUTHERIA_PLUGIN_PORT`** — the host proxies all plugin traffic here.
2. **Declare the routes it handles** in `manifest.json` under `routes`.

The proxy enforces permissions: if a request's path does not start with a declared route, the host returns `403 Forbidden` before forwarding anything to your plugin.

### How URLs work

```
Browser → Axum proxy                    → Your plugin
GET /plugins/my-plugin/api/data  →  GET /api/data
GET /plugins/my-plugin           →  GET /
```

The host strips the `/plugins/<id>` prefix before forwarding. So `GET /plugins/my-plugin/api/data` arrives at your server as `GET /api/data`.

Your plugin should handle both forms if it may be called directly during development:
```
GET /          or  GET /plugins/my-plugin
GET /api/data  or  GET /plugins/my-plugin/api/data
```

### Route declaration example

```json
"routes": [
  "/plugins/my-plugin"
]
```

This single entry covers all subpaths: `/plugins/my-plugin`, `/plugins/my-plugin/api/data`, etc.

---

## Calling the host API

Make standard HTTP requests to `http://127.0.0.1:<ELEUTHERIA_APP_PORT>` with the session token:

```python
import urllib.request
import json

def call_host(path):
    url = f"http://127.0.0.1:{APP_PORT}{path}"
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {TOKEN}"})
    with urllib.request.urlopen(req, timeout=3) as r:
        return json.loads(r.read())

# Health check (no auth required)
data = call_host("/health")
# → {"status": "ok", "message": "..."}

# List clipboard history
items = call_host("/api/clipboard")
```

### Available host endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check. Returns `{"status":"ok"}`. No auth required. |
| `GET` | `/api/clipboard` | List clipboard history. Query: `?q=search&limit=50`. |
| `GET` | `/api/notes` | List notes. Query: `?q=search`. |
| `POST` | `/api/notes` | Create note. Form fields: `title`, `content`, `tags`. |

All endpoints except `/health` require `Authorization: Bearer <ELEUTHERIA_TOKEN>`.

---

## UI: serving HTMX fragments

Your plugin's root route (`GET /`) should return an HTML fragment that the host swaps into `#tool-panel`. Do **not** return a full `<!DOCTYPE html>` page — just the content div.

```html
<div class="p-6 max-w-xl">
  <h2 class="text-xl font-semibold text-white mb-4">🔌 My Plugin</h2>

  <form hx-post="/plugins/my-plugin/api/action"
        hx-target="#result"
        hx-swap="innerHTML">
    <input name="value" type="text"
           class="bg-gray-700 text-white rounded px-3 py-2 text-sm" />
    <button type="submit"
            class="bg-blue-600 hover:bg-blue-500 text-white text-sm px-4 py-2 rounded">
      Submit
    </button>
  </form>

  <div id="result" class="mt-4 text-sm text-gray-300"></div>
</div>
```

### HTMX conventions

- Always use **absolute paths starting with `/plugins/<your-id>/`** in `hx-get`, `hx-post`, etc. The HTMX URL rewriter in the shell will route them through the proxy correctly.
- Always set `hx-target` and `hx-swap` explicitly — never rely on defaults.
- Use Tailwind CSS classes for styling — Tailwind CDN is loaded by the shell.
- **Do not** load HTMX, Alpine.js, or Tailwind yourself — they are already available in the WebView.

### Alpine.js

Alpine.js is available. You can use `x-data`, `x-show`, `x-on`, etc. in your fragment. Since the fragment is injected via HTMX swap, Alpine automatically initializes new elements.

```html
<div x-data="{ open: false }">
  <button @click="open = !open">Toggle</button>
  <p x-show="open">Hello from Alpine</p>
</div>
```

---

## HTML escaping

Always escape user-supplied or dynamic data before inserting into HTML:

```python
import html
safe = html.escape(user_input)
```

```js
function htmlEscape(str) {
  return str.replace(/&/g,"&amp;").replace(/</g,"&lt;")
            .replace(/>/g,"&gt;").replace(/"/g,"&quot;");
}
```

---

## Graceful shutdown

The host sends `SIGTERM` when it exits. Handle it cleanly so port-in-use errors don't block the next startup:

```python
import signal, sys

def shutdown(sig, frame):
    server.shutdown()
    sys.exit(0)

signal.signal(signal.SIGTERM, shutdown)
```

```js
process.on("SIGTERM", () => {
  server.close(() => process.exit(0));
});
```

---

## Local development (without the full app)

You can run your plugin standalone to test routes:

```bash
# Set the env vars manually
export ELEUTHERIA_APP_PORT=47821
export ELEUTHERIA_TOKEN=dev-token
export ELEUTHERIA_PLUGIN_ID=my-plugin
export ELEUTHERIA_PLUGIN_PORT=47901

cd plugins/my-plugin
python3 main.py       # or: node main.js
```

Then test with curl:

```bash
# UI fragment
curl http://127.0.0.1:47901/

# API endpoint
curl http://127.0.0.1:47901/api/hello

# POST form
curl -X POST http://127.0.0.1:47901/api/echo \
  -d "message=hello+world"
```

`host_reachable` will be `false` in standalone mode (no Axum server running) — that is expected.

---

## Reference implementations

| Plugin | Runtime | Location |
|--------|---------|----------|
| Hello Python | Python 3 (stdlib only) | `plugins/hello-python/` |
| Hello Node | Node.js 22 (stdlib only) | `plugins/hello-node/` |

Both implement the same surface:
- `GET /` — HTMX UI fragment with echo form
- `GET /api/hello` — JSON info (id, port, runtime version, `host_reachable`)
- `POST /api/echo` — echoes `message` form field as HTML

---

## Checklist for a new plugin

- [ ] Create `plugins/<id>/` directory
- [ ] Write `manifest.json` with correct `id`, `runtime`, `entry`, and `routes`
- [ ] Start HTTP server on `ELEUTHERIA_PLUGIN_PORT` bound to `127.0.0.1`
- [ ] Return an HTMX fragment from `GET /`
- [ ] Use `/plugins/<id>/...` paths in all `hx-*` attributes
- [ ] Handle `SIGTERM` for clean shutdown
- [ ] Escape all dynamic content in HTML responses
- [ ] Test standalone with env vars set manually
- [ ] Restart the app — your plugin appears in the sidebar automatically
