use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CategoryTier {
    Security,
    Core,
    Desktop,
    Applications,
    Libraries,
    Other,
}

impl CategoryTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Security => "Security & Critical",
            Self::Core => "Core & System",
            Self::Desktop => "Desktop & Experience",
            Self::Applications => "User Applications",
            Self::Libraries => "Libraries & Development",
            Self::Other => "Other Updates",
        }
    }

    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::Security => "[SECURITY & CRITICAL]",
            Self::Core => "[CORE & SYSTEM]",
            Self::Desktop => "[DESKTOP & UX]",
            Self::Applications => "[APPLICATIONS]",
            Self::Libraries => "[LIBRARIES & DEVELOPMENT]",
            Self::Other => "[OTHER UPDATES]",
        }
    }
}

impl fmt::Display for CategoryTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Moderate,
    Low,
    Unspecified,
}

impl Severity {
    pub fn from_str(val: &str) -> Self {
        let v = val.trim().to_lowercase();
        if v.contains("crit") {
            Self::Critical
        } else if v.contains("high") || v.contains("urg") || v.contains("imp") {
            Self::High
        } else if v.contains("mod") || v.contains("med") {
            Self::Moderate
        } else if v.contains("low") {
            Self::Low
        } else {
            Self::Unspecified
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Moderate => "Moderate",
            Self::Low => "Low",
            Self::Unspecified => "Unspecified",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReference {
    pub bug_id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPackage {
    pub name: String,
    pub new_version: String,
    pub installed_version: String,
    pub arch: String,
    pub repo: String,
}

impl PendingPackage {
    pub fn new(name: impl Into<String>, new_version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            new_version: new_version.into(),
            installed_version: "installed".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedChangeItem {
    pub package_name: String,
    pub installed_version: String,
    pub new_version: String,
    pub category: CategoryTier,
    pub severity: Severity,
    pub advisory_id: Option<String>,
    pub advisory_type: Option<String>,
    pub cves: Vec<String>,
    pub bugs: Vec<BugReference>,
    pub bullets: Vec<String>,
    pub source_name: String,
}
