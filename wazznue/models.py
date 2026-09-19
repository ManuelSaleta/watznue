"""Data models for wazznue."""

from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional


class CategoryTier(str, Enum):
    SECURITY = "Security & Critical"
    CORE = "Core & System"
    DESKTOP = "Desktop & Experience"
    APPLICATIONS = "User Applications"
    LIBRARIES = "Libraries & Development"
    OTHER = "Other Updates"


class Severity(str, Enum):
    CRITICAL = "Critical"
    HIGH = "High"
    MODERATE = "Moderate"
    LOW = "Low"
    UNSPECIFIED = "Unspecified"

    @classmethod
    def from_str(cls, val: Optional[str]) -> "Severity":
        if not val:
            return cls.UNSPECIFIED
        v = val.strip().lower()
        if "crit" in v:
            return cls.CRITICAL
        elif "high" in v or "urg" in v or "imp" in v:
            return cls.HIGH
        elif "mod" in v or "med" in v:
            return cls.MODERATE
        elif "low" in v:
            return cls.LOW
        return cls.UNSPECIFIED


@dataclass
class BugReference:
    bug_id: str
    title: str = ""
    url: str = ""


@dataclass
class PendingPackage:
    name: str
    new_version: str
    installed_version: str = "installed"
    arch: str = "x86_64"
    repo: str = "updates"


@dataclass
class NormalizedChangeItem:
    package_name: str
    installed_version: str
    new_version: str
    category: CategoryTier = CategoryTier.OTHER
    severity: Severity = Severity.UNSPECIFIED
    advisory_id: Optional[str] = None
    advisory_type: Optional[str] = None
    cves: List[str] = field(default_factory=list)
    bugs: List[BugReference] = field(default_factory=list)
    bullets: List[str] = field(default_factory=list)
    source_name: str = "local"
