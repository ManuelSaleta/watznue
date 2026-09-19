"""Source aggregators for Bodhi REST API and local updateinfo."""

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any, Dict, List, Optional
import httpx


class SourceAggregator:
    """Manages queries to remote and local update metadata sources with disk caching."""

    def __init__(self, cache_dir: Optional[Path] = None, cache_ttl_seconds: int = 43200):
        self.cache_dir = cache_dir or Path(os.path.expanduser("~/.cache/wazznue"))
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.cache_file = self.cache_dir / "bodhi_cache.json"
        self.cache_ttl = cache_ttl_seconds
        self._cache: Dict[str, Any] = self._load_cache()

    def _load_cache(self) -> Dict[str, Any]:
        if self.cache_file.exists():
            try:
                with open(self.cache_file, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                return {}
        return {}

    def _save_cache(self) -> None:
        try:
            with open(self.cache_file, "w", encoding="utf-8") as f:
                json.dump(self._cache, f, indent=2)
        except Exception:
            pass

    def get_source_package_name(self, pkg_name: str) -> str:
        """Attempts to find the base/source RPM name for subpackages."""
        # Common subpackage heuristics
        if pkg_name.startswith("mesa-"):
            return "mesa"
        if pkg_name.startswith("kernel-"):
            return "kernel"
        if pkg_name.startswith("systemd-"):
            return "systemd"
        return pkg_name

    def fetch_bodhi_update(self, package_name: str) -> Optional[Dict[str, Any]]:
        """Queries Bodhi for the most recent advisory for package_name with cache check."""
        now = time.time()
        lookup_name = self.get_source_package_name(package_name)

        cached = self._cache.get(lookup_name)
        if cached and (now - cached.get("timestamp", 0) < self.cache_ttl):
            return cached.get("data")

        url = "https://bodhi.fedoraproject.org/updates/"
        params = {"packages": lookup_name, "rows_per_page": 1}
        try:
            with httpx.Client(timeout=3.0) as client:
                resp = client.get(url, params=params)
                if resp.status_code == 200:
                    data = resp.json()
                    updates = data.get("updates", [])
                    item = updates[0] if updates else None
                    # Cache result (including None / negative cache)
                    self._cache[lookup_name] = {"timestamp": now, "data": item}
                    self._save_cache()
                    return item
        except Exception:
            # On network failure or timeout, fall through to None
            pass

        # Negative cache fallback to prevent hammering on timeout
        self._cache[lookup_name] = {"timestamp": now, "data": None}
        self._save_cache()
        return None

    def fetch_local_advisory(self, package_name: str) -> Optional[str]:
        """Queries local dnf5 advisory info if available."""
        for pm in ["dnf5", "dnf"]:
            cmd = [pm, "advisory", "info", package_name]
            try:
                res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=2.0)
                if res.returncode == 0 and res.stdout.strip():
                    return res.stdout
            except Exception:
                continue
        return None

    def fetch_rpm_changelog(self, package_name: str, max_lines: int = 25) -> Optional[str]:
        """Queries local RPM changelog for installed version."""
        try:
            cmd = ["rpm", "-q", "--changelog", package_name]
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=1.0)
            if res.returncode == 0 and res.stdout.strip():
                lines = res.stdout.splitlines()[:max_lines]
                return "\n".join(lines)
        except Exception:
            pass
        return None
