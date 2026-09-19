"""Package manager detection and pending updates discovery."""

import re
import shutil
import subprocess
from typing import List, Optional
from wazznue.models import PendingPackage


class PackageManagerDetector:
    """Detects installed package managers and queries pending package updates."""

    def __init__(self, binary_override: Optional[str] = None):
        self.pm_bin = binary_override or self._detect_binary()

    def _detect_binary(self) -> str:
        for candidate in ["dnf5", "dnf"]:
            path = shutil.which(candidate)
            if path:
                return candidate
        return "dnf"

    def get_pending_packages(self, filter_packages: Optional[List[str]] = None) -> List[PendingPackage]:
        """Runs check-update and returns pending packages."""
        if filter_packages:
            # If specific packages requested, return them as candidates
            return [
                PendingPackage(name=pkg, new_version="latest", installed_version="installed")
                for pkg in filter_packages
            ]

        cmd = [self.pm_bin, "check-update"]
        try:
            res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            # check-update returns:
            # 100: updates available
            # 0: no updates available
            # others: error
            return self._parse_check_update_output(res.stdout)
        except Exception as e:
            print(f"Warning: Failed to execute {self.pm_bin} check-update: {e}")
            return []

    def _parse_check_update_output(self, output: str) -> List[PendingPackage]:
        packages: List[PendingPackage] = []
        for line in output.splitlines():
            line = line.strip()
            if not line or line.startswith("Updating and loading") or line.startswith("Repositories loaded."):
                continue
            if line.startswith("Security:") or line.startswith("Last metadata"):
                continue

            # Format: name.arch  version-release  repo
            # Example: mesa-dri-drivers.x86_64 24.1.3-1.fc40 updates
            parts = line.split()
            if len(parts) >= 2:
                pkg_full = parts[0]
                new_ver = parts[1]
                repo = parts[2] if len(parts) > 2 else "updates"

                if "." in pkg_full:
                    name, arch = pkg_full.rsplit(".", 1)
                else:
                    name, arch = pkg_full, "x86_64"

                # Exclude header-like lines
                if name.lower() in ["name", "package", "obsoleting"]:
                    continue

                packages.append(PendingPackage(
                    name=name,
                    new_version=new_ver,
                    arch=arch,
                    repo=repo
                ))
        return packages


def get_demo_packages() -> List[PendingPackage]:
    """Provides a realistic set of sample packages for demonstration purposes."""
    return [
        PendingPackage(name="kernel", new_version="6.10.10-200.fc40", installed_version="6.10.9-200.fc40"),
        PendingPackage(name="mesa-dri-drivers", new_version="24.1.7-1.fc40", installed_version="24.1.6-1.fc40"),
        PendingPackage(name="pipewire", new_version="1.2.4-1.fc40", installed_version="1.2.3-1.fc40"),
        PendingPackage(name="mutter", new_version="46.5-1.fc40", installed_version="46.4-1.fc40"),
        PendingPackage(name="firefox", new_version="130.0.1-1.fc40", installed_version="130.0-1.fc40"),
        PendingPackage(name="gdk-pixbuf2", new_version="2.44.6-3.fc44", installed_version="2.44.4-1.fc44"),
    ]
