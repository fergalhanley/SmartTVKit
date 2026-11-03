#!/usr/bin/env python3
"""Synchronise the local Rust renderer A2UI fixture with Elsewhere TV API."""
import json
import os
from pathlib import Path
from urllib.parse import urlencode
from urllib.request import urlopen

root = Path(__file__).resolve().parents[2]
base = os.getenv("ELSEWHERE_TV_API", "http://localhost:8787").rstrip("/")
locale = os.getenv("ELSEWHERE_TV_LOCALE", "en-AU")
url = base + "/v1/a2ui/home?" + urlencode({"locale": locale})
with urlopen(url, timeout=20) as response:
    content = response.read().decode("utf-8")
messages = [json.loads(line) for line in content.splitlines() if line.strip()]
assert len(messages) >= 3 and messages[0].get("version") == "v0.9.1"
assert "createSurface" in messages[0]
assert "updateComponents" in messages[1]
assert "updateDataModel" in messages[2]
output = root / "app/elsewhere-tv.a2ui.jsonl"
output.write_text(content, encoding="utf-8")
print("Synced", url, "to", output)
print("Run: SMARTTVKIT_A2UI_FILE=app/elsewhere-tv.a2ui.jsonl cargo run")
