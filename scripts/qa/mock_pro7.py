"""Minimal ProPresenter 7 REST mock on 127.0.0.1:18025. Logs every path.
GET /_log returns the log; POST /_select?name=X sets the selected presentation."""
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import parse_qs, urlparse

LOG = []
STATE = {"name": "Amazing Grace", "slide": 0}


class H(BaseHTTPRequestHandler):
    def _json(self, obj, code=200):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        u = urlparse(self.path)
        if u.path == "/_log":
            return self._json(LOG)
        LOG.append(u.path)
        if u.path == "/v1/version":
            return self._json({"name": "ProPresenter", "version": "7.16"})
        if u.path == "/v1/presentation/slide_index":
            return self._json({
                "slide_index": STATE["slide"],
                "presentation_index": {"presentation_id": {"name": STATE["name"]}, "index": STATE["slide"]},
            })
        if u.path.startswith("/v1/presentation/active/") and u.path.endswith("/trigger"):
            STATE["slide"] = int(u.path.split("/")[4])
        self.send_response(204)
        self.end_headers()

    def do_POST(self):
        u = urlparse(self.path)
        if u.path == "/_select":
            STATE["name"] = parse_qs(u.query)["name"][0]
            LOG.clear()
            return self._json({"ok": True})
        if u.path == "/_clear":
            LOG.clear()
            return self._json({"ok": True})
        self.send_response(404)
        self.end_headers()

    def log_message(self, *a):
        pass


HTTPServer(("127.0.0.1", int(sys.argv[1]) if len(sys.argv) > 1 else 18025), H).serve_forever()
