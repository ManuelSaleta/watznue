use crate::models::{
    BugReference, CategoryTier, NormalizedChangeItem, PendingPackage, Severity,
};
use crate::sources::BodhiUpdate;
use regex::Regex;

const CORE_PACKAGES: &[&str] = &[
    "kernel",
    "kernel-core",
    "kernel-modules",
    "systemd",
    "systemd-libs",
    "pipewire",
    "wireplumber",
    "mesa-dri-drivers",
    "mesa-filesystem",
    "mesa",
    "glibc",
    "dracut",
    "grub2",
    "grub2-common",
    "linux-firmware",
];

const DESKTOP_KEYWORDS: &[&str] = &[
    "gnome", "mutter", "wayland", "gtk", "qt", "desktop", "sound",
    "audio", "session", "display", "shell", "compositor", "alacritty",
];

const APP_KEYWORDS: &[&str] = &[
    "firefox", "thunderbird", "libreoffice", "chromium", "vlc", "gimp",
    "inkscape", "obs-studio", "code", "vscodium", "discord",
];

const DEV_KEYWORDS: &[&str] = &[
    "python", "rust", "gcc", "llvm", "clang", "meson", "ninja",
    "golang", "git", "cmake",
];

pub struct ChangelogNormalizer;

impl ChangelogNormalizer {
    pub fn classify_category(
        package_name: &str,
        advisory_type: Option<&str>,
        cves: &[String],
    ) -> CategoryTier {
        let name = package_name.to_lowercase();

        if let Some(atype) = advisory_type {
            if atype.eq_ignore_ascii_case("security") {
                return CategoryTier::Security;
            }
        }
        if !cves.is_empty() {
            return CategoryTier::Security;
        }

        if CORE_PACKAGES.contains(&name.as_str()) || name.starts_with("kernel-") {
            return CategoryTier::Core;
        }

        if DESKTOP_KEYWORDS.iter().any(|&kw| name.contains(kw)) {
            return CategoryTier::Desktop;
        }

        if APP_KEYWORDS.iter().any(|&kw| name.contains(kw)) {
            return CategoryTier::Applications;
        }

        if DEV_KEYWORDS.iter().any(|&kw| name.contains(kw)) || name.starts_with("lib") {
            return CategoryTier::Libraries;
        }

        CategoryTier::Other
    }

    pub fn extract_bullets_from_text(raw_text: &str, max_bullets: usize) -> Vec<String> {
        let mut bullets = Vec::new();
        let boilerplate = [
            "rebuilt for",
            "automatic update for",
            "mass rebuild",
            "bump release",
            "rpmautospec",
            "changelog for",
        ];

        for line in raw_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Strip specfile changelog headers (* Fri Sep 19 2026 ...)
            if trimmed.starts_with("* ") && (trimmed.contains('@') || trimmed.contains('-')) {
                continue;
            }

            // Strip markdown headers
            if trimmed.starts_with('#') {
                continue;
            }

            let lower = trimmed.to_lowercase();
            if boilerplate.iter().any(|&bp| lower.contains(bp)) {
                continue;
            }

            // Strip leading bullet markers
            let mut cleaned = trimmed.trim_start_matches(['-', '*', '•', ' ']);

            // Strip markdown bold wrappers
            if cleaned.starts_with("**") && cleaned.ends_with("**") && cleaned.len() > 4 {
                cleaned = &cleaned[2..cleaned.len() - 2];
            }
            let cleaned = cleaned.trim();

            if cleaned.len() < 4 {
                continue;
            }

            let cleaned_str = cleaned.to_string();
            if !bullets.contains(&cleaned_str) {
                bullets.push(cleaned_str);
                if bullets.len() >= max_bullets {
                    break;
                }
            }
        }

        bullets
    }

    pub fn normalize(
        pkg: &PendingPackage,
        bodhi_data: Option<&BodhiUpdate>,
        raw_changelog: Option<&str>,
    ) -> NormalizedChangeItem {
        let mut cves = Vec::new();
        let mut bugs = Vec::new();
        let mut bullets = Vec::new();
        let mut advisory_id = None;
        let mut advisory_type = None;
        let mut severity = Severity::Unspecified;
        let mut source_name = "local".to_string();

        let cve_regex = Regex::new(r"(?i)\bCVE-\d{4}-\d{4,7}\b").unwrap();

        if let Some(bodhi) = bodhi_data {
            source_name = "Fedora Bodhi".to_string();
            advisory_id = bodhi.alias.clone().or_else(|| bodhi.updateid.clone());
            advisory_type = bodhi.update_type.clone();

            if let Some(ref sev) = bodhi.severity {
                severity = Severity::from_str(sev);
            }

            for bug in &bodhi.bugs {
                let id_str = match &bug.bug_id {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => String::new(),
                };
                if !id_str.is_empty() {
                    bugs.push(BugReference {
                        bug_id: id_str.clone(),
                        title: bug.title.clone().unwrap_or_default(),
                        url: format!("https://bugzilla.redhat.com/show_bug.cgi?id={}", id_str),
                    });
                }
            }

            if let Some(ref notes) = bodhi.notes {
                for cap in cve_regex.find_iter(notes) {
                    let cve_upper = cap.as_str().to_uppercase();
                    if !cves.contains(&cve_upper) {
                        cves.push(cve_upper);
                    }
                }
                bullets = Self::extract_bullets_from_text(notes, 5);
            }
        }

        if bullets.len() < 2 {
            if let Some(changelog) = raw_changelog {
                let rpm_bullets = Self::extract_bullets_from_text(changelog, 4);
                for b in rpm_bullets {
                    if !bullets.contains(&b) {
                        bullets.push(b);
                    }
                    if bullets.len() >= 4 {
                        break;
                    }
                }
            }
        }

        if bullets.is_empty() {
            bullets.push(format!("Update to {}", pkg.new_version));
        }

        let category = Self::classify_category(&pkg.name, advisory_type.as_deref(), &cves);

        NormalizedChangeItem {
            package_name: pkg.name.clone(),
            installed_version: pkg.installed_version.clone(),
            new_version: pkg.new_version.clone(),
            category,
            severity,
            advisory_id,
            advisory_type,
            cves,
            bugs,
            bullets,
            source_name,
        }
    }
}
