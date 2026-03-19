#!/usr/bin/env python3
"""
hello-python — Reference Eleutheria Telos plugin (Python).

This plugin demonstrates the minimum structure required to build a plugin:
  - Reads env vars injected by the host app
  - Starts an HTTP server on the assigned port
  - Serves an HTMX UI fragment at GET /
  - Exposes a JSON API at GET /api/hello and POST /api/echo
  - Optionally calls back into the host app via ELEUTHERIA_APP_PORT + ELEUTHERIA_TOKEN

Requires only Python 3 stdlib — no third-party packages needed.
"""

import json
import logging
import os
import urllib.request
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs, urlparse

# ── Configuration (injected by host) ──────────────────────────────────────────

APP_PORT = int(os.environ.get("ELEUTHERIA_APP_PORT", "47821"))
TOKEN = os.environ.get("ELEUTHERIA_TOKEN", "")
PLUGIN_ID = os.environ.get("ELEUTHERIA_PLUGIN_ID", "hello-python")
PLUGIN_PORT = int(os.environ.get("ELEUTHERIA_PLUGIN_PORT", "47900"))

logging.basicConfig(
    level=logging.INFO,
    format=f"[{PLUGIN_ID}] %(levelname)s %(message)s",
)
log = logging.getLogger(__name__)

# ── Helpers ────────────────────────────────────────────────────────────────────

UI_HTML = """\
<div id="tool-panel" class="p-6 max-w-xl">
  <h2 class="text-xl font-semibold text-white mb-1">🐍 Hello Python</h2>
  <p class="text-gray-400 text-sm mb-6">
    Reference plugin — demonstrates the Eleutheria Telos plugin API.
  </p>

  <!-- Echo form -->
  <form class="space-y-3"
        hx-post="/plugins/hello-python/api/echo"
        hx-target="#echo-result"
        hx-swap="innerHTML">
    <label class="block text-sm text-gray-300">Send a message to the plugin:</label>
    <div class="flex gap-2">
      <input name="message"
             type="text"
             placeholder="Type something…"
             class="flex-1 bg-gray-700 text-white rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500" />
      <button type="submit"
              class="bg-blue-600 hover:bg-blue-500 text-white text-sm px-4 py-2 rounded transition-colors">
        Echo
      </button>
    </div>
  </form>

  <div id="echo-result" class="mt-4 text-sm text-gray-300"></div>

  <!-- Info from plugin JSON API -->
  <div class="mt-8 border-t border-gray-700 pt-4">
    <button class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
            hx-get="/plugins/hello-python/api/hello"
            hx-target="#hello-result"
            hx-swap="innerHTML">
      Fetch plugin info
    </button>
    <div id="hello-result" class="mt-2 text-xs text-gray-400 font-mono"></div>
  </div>
</div>
"""


def json_response(data: dict) -> bytes:
    return json.dumps(data).encode()


def call_host_api(path: str) -> dict | None:
    """Make an authenticated GET request to the host app's API."""
    url = f"http://127.0.0.1:{APP_PORT}{path}"
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {TOKEN}"})
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            return json.loads(resp.read())
    except Exception as exc:
        log.warning("Host API call failed (%s): %s", path, exc)
        return None


# ── Request handler ────────────────────────────────────────────────────────────

class PluginHandler(BaseHTTPRequestHandler):
    """Routes incoming requests to the appropriate handler."""

    def log_message(self, fmt, *args):  # silence default access log
        log.debug(fmt, *args)

    # Helpers ──────────────────────────────────────────────────────────────────

    def _send(self, status: int, content_type: str, body: bytes):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _read_body(self) -> bytes:
        length = int(self.headers.get("Content-Length", 0))
        return self.rfile.read(length) if length > 0 else b""

    def _parse_form(self) -> dict[str, str]:
        raw = self._read_body().decode()
        parsed = parse_qs(raw, keep_blank_values=True)
        return {k: v[0] for k, v in parsed.items()}

    # Routes ───────────────────────────────────────────────────────────────────

    def do_GET(self):
        path = urlparse(self.path).path.rstrip("/") or "/"

        if path in ("/", "/plugins/hello-python"):
            # Serve the HTMX UI fragment
            self._send(200, "text/html; charset=utf-8", UI_HTML.encode())

        elif path in ("/api/hello", "/plugins/hello-python/api/hello"):
            # Return plugin info as JSON
            data = {
                "plugin_id": PLUGIN_ID,
                "plugin_port": PLUGIN_PORT,
                "app_port": APP_PORT,
                "python_version": __import__("sys").version,
            }
            # Verify host connectivity via /health (returns JSON, no auth needed)
            host_data = call_host_api("/health")
            if host_data is not None:
                data["host_reachable"] = True
            else:
                data["host_reachable"] = False

            body = json_response(data)
            # Return as formatted HTML for HTMX target
            html = f"<pre>{json.dumps(data, indent=2)}</pre>"
            self._send(200, "text/html; charset=utf-8", html.encode())

        else:
            self._send(404, "application/json", json_response({"error": "not found"}))

    def do_POST(self):
        path = urlparse(self.path).path.rstrip("/")

        if path in ("/api/echo", "/plugins/hello-python/api/echo"):
            form = self._parse_form()
            message = form.get("message", "").strip()

            if not message:
                html = '<p class="text-gray-500">Nothing to echo.</p>'
            else:
                escaped = (
                    message
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;")
                    .replace('"', "&quot;")
                )
                html = (
                    f'<p class="text-green-400">Plugin echoes: '
                    f'<span class="font-mono">{escaped}</span></p>'
                )

            self._send(200, "text/html; charset=utf-8", html.encode())

        else:
            self._send(404, "application/json", json_response({"error": "not found"}))


# ── Entry point ────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", PLUGIN_PORT), PluginHandler)
    log.info("Listening on http://127.0.0.1:%d", PLUGIN_PORT)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        log.info("Shutting down")
        server.server_close()
