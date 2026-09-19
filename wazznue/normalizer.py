"""Data normalizer and changelog bullet extractor."""

import re
from typing import Any, Dict, List, Optional
from wazznue.models import (
    BugReference,
    CategoryTier,
    NormalizedChangeItem,
    PendingPackage,
    Severity,
)

CORE_PACKAGES = {
    "kernel", "kernel-core", "kernel-modules", "systemd", "systemd-libs",
    "pipewire", "wireplumber", "mesa-dri-drivers", "mesa-filesystem", "mesa",
    "glibc", "dracut", "grub2", "grub2-common", "linux-firmware"
}

DESKTOP_KEYWORDS = [
    "gnome", "mutter", "wayland", "gtk", "qt", "desktop", "sound",
    "audio", "session", "display", "shell", "compositor", "alacritty"
]

APP_KEYWORDS = [
    "firefox", "thunderbird", "libreoffice", "chromium", "vlc", "gimp",
    "inkscape", "obs-studio", "code", "vscodium", "discord"
]

DEV_KEYWORDS = [
    "python", "rust", "gcc", "llvm", "clang", "meson", "ninja",
    "golang", "git", "cmake"
]


class ChangelogNormalizer:
    """Cleans, categorizes, and normalizes unstructured advisory data into standard records."""

    @staticmethod
    def classify_category(package_name: str, advisory_type: Optional[str] = None, cves: Optional[List[str]] = None) -> CategoryTier:
        name = package_name.lower()

        # Security check first
        if (advisory_type and advisory_type.lower() == "security") or (cves and len(cves) > 0):
            return CategoryTier.SECURITY

        # Core system check
        if name in CORE_PACKAGES or name.startswith("kernel-"):
            return CategoryTier.CORE

        # Desktop check
        if any(kw in name for kw in DESKTOP_KEYWORDS):
            return CategoryTier.DESKTOP

        # User applications
        if any(kw in name for kw in APP_KEYWORDS):
            return CategoryTier.APPLICATIONS

        # Development & libraries
        if any(kw in name for kw in DEV_KEYWORDS) or name.startswith("lib"):
            return CategoryTier.LIBRARIES

        return CategoryTier.OTHER

    @staticmethod
    def extract_bullets_from_text(raw_text: str, max_bullets: int = 5) -> List[str]:
        """Cleans messy text into succinct, readable bullet points."""
        if not raw_text:
            return []

        bullets: List[str] = []
        lines = raw_text.splitlines()

        for line in lines:
            line = line.strip()
            if not line:
                continue

            # Strip RPM specfile date headers (e.g., * Fri Sep 19 2026 Jane Doe - 1.2.3)
            if line.startswith("* ") and ("@" in line or "-" in line):
                continue

            # Strip markdown header markers (e.g. ##### **Changelog**)
            if re.match(r"^#+\s*", line):
                continue

            # Strip common non-informative boilerplate
            lower = line.lower()
            boilerplate_patterns = [
                "rebuilt for",
                "automatic update for",
                "mass rebuild",
                "bump release",
                "rpmautospec",
                "changelog for",
            ]
            if any(p in lower for p in boilerplate_patterns):
                continue

            # Clean markdown bold/italic formatting wrappers and bullet markers
            cleaned = re.sub(r"^[-*•\s]+", "", line).strip()
            cleaned = re.sub(r"^\*\*(.+?)\*\*$", r"\1", cleaned).strip()

            if not cleaned or len(cleaned) < 4:
                continue

            # Deduplicate
            if cleaned not in bullets:
                bullets.append(cleaned)
                if len(bullets) >= max_bullets:
                    break

        return bullets

    @classmethod
    def normalize(
        cls,
        pkg: PendingPackage,
        bodhi_data: Optional[Dict[str, Any]] = None,
        raw_changelog: Optional[str] = None,
    ) -> NormalizedChangeItem:
        """Constructs a NormalizedChangeItem from multiple possible data sources."""
        cves: List[str] = []
        bugs: List[BugReference] = []
        bullets: List[str] = []
        advisory_id: Optional[str] = None
        advisory_type: Optional[str] = None
        severity = Severity.UNSPECIFIED
        source_name = "local"

        # Ingest Bodhi metadata if present
        if bodhi_data:
            source_name = "Fedora Bodhi"
            advisory_id = bodhi_data.get("alias") or bodhi_data.get("updateid")
            advisory_type = bodhi_data.get("type")
            severity = Severity.from_str(bodhi_data.get("severity"))

            # Extract bugs
            for b in bodhi_data.get("bugs", []):
                bid = str(b.get("bug_id", ""))
                title = b.get("title", "")
                if bid:
                    bugs.append(BugReference(bug_id=bid, title=title))

            # Extract CVEs from notes and comments
            notes = bodhi_data.get("notes", "") or ""
            cve_matches = re.findall(r"CVE-\d{4}-\d{4,7}", notes, re.IGNORECASE)
            for c in cve_matches:
                c_upper = c.upper()
                if c_upper not in cves:
                    cves.append(c_upper)

            # Extract bullets from notes
            bullets = cls.extract_bullets_from_text(notes)

        # Fallback to local changelog if bullets are sparse
        if len(bullets) < 2 and raw_changelog:
            rpm_bullets = cls.extract_bullets_from_text(raw_changelog)
            for b in rpm_bullets:
                if b not in bullets:
                    bullets.append(b)
                if len(bullets) >= 4:
                    break

        # Fallback if still empty
        if not bullets:
            bullets = [f"Update to {pkg.new_version}"]

        category = cls.classify_category(pkg.name, advisory_type, cves)

        return NormalizedChangeItem(
            package_name=pkg.name,
            installed_version=pkg.installed_version,
            new_version=pkg.new_version,
            category=category,
            severity=severity,
            advisory_id=advisory_id,
            advisory_type=advisory_type,
            cves=cves,
            bugs=bugs,
            bullets=bullets,
            source_name=source_name,
        )
