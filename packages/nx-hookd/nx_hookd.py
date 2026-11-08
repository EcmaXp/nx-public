"""Serve Claude Code telemetry hooks from one process instead of one Python start per event."""

import json
import os
import queue
import socketserver
import sys
import threading
from datetime import UTC, datetime
from http.server import BaseHTTPRequestHandler
from pathlib import Path

from sidepulse.hook import (
    routed_hook_payload,
    send_hook_event,
    write_hook_line,
    write_hook_status_audit,
)
from sidepulse.origin import annotate_payload_with_origin

ENV_HEADER = "x-hook-env"


def socket_path() -> Path:
    return Path(
        os.environ.get("NX_HOOKD_SOCKET") or Path.home() / ".local/state/nx/hookd.sock"
    )


def sidepulse_log_path(provider: str, env: dict[str, str]) -> Path:
    state = env.get("XDG_STATE_HOME") or str(Path.home() / ".local/state")
    return (
        Path(state).expanduser() / "sidepulse" / "agent-monitor" / f"{provider}.jsonl"
    )


def log_sidepulse(provider: str, payload: dict, env: dict[str, str]) -> None:
    payload = annotate_payload_with_origin(provider, payload, env=env)
    actual_provider, log_path, line = routed_hook_payload(
        provider,
        sidepulse_log_path(provider, env),
        json.dumps(payload, ensure_ascii=False),
    )
    for step in (
        lambda: write_hook_line(log_path, line),
        lambda: write_hook_status_audit(actual_provider, line),
        lambda: send_hook_event(actual_provider, line),
    ):
        try:
            step()
        except Exception as exc:  # noqa: BLE001
            print(f"sidepulse {provider}: {exc!r}", file=sys.stderr, flush=True)


jobs: queue.Queue = queue.Queue()


def worker() -> None:
    while True:
        provider, payload, env = jobs.get()
        log_sidepulse(provider, payload, env)


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_POST(self) -> None:
        parts = self.path.strip("/").split("/")
        body = self.rfile.read(int(self.headers.get("content-length") or 0))
        if len(parts) != 2 or parts[0] != "sidepulse":
            self.reply(404)
            return
        try:
            payload = json.loads(body or b"{}")
            env = json.loads(self.headers.get(ENV_HEADER) or "{}")
        except json.JSONDecodeError:
            self.reply(400)
            return
        if not isinstance(payload, dict) or not isinstance(env, dict):
            self.reply(400)
            return
        payload.setdefault(
            "logged_at", datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
        )
        jobs.put((parts[1], payload, {str(k): str(v) for k, v in env.items()}))
        self.reply(204)

    def reply(self, status: int) -> None:
        self.send_response(status)
        self.send_header("content-length", "0")
        self.end_headers()

    def address_string(self) -> str:
        return "uds"

    def log_message(self, format: str, *args) -> None:
        pass


class Server(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
    daemon_threads = True


def main() -> None:
    path = socket_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.unlink(missing_ok=True)
    threading.Thread(target=worker, daemon=True).start()
    os.umask(0o077)
    with Server(str(path), Handler) as server:
        server.serve_forever()


if __name__ == "__main__":
    main()
