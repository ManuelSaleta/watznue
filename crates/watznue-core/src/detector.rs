use crate::models::PendingPackage;
use std::process::Command;

pub struct PackageManagerDetector {
    pm_bin: String,
}

impl Default for PackageManagerDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManagerDetector {
    pub fn new() -> Self {
        let pm_bin = Self::detect_binary();
        Self { pm_bin }
    }

    fn detect_binary() -> String {
        for candidate in &["dnf5", "dnf"] {
            if let Ok(status) = Command::new("which").arg(candidate).output() {
                if status.status.success() {
                    return candidate.to_string();
                }
            }
        }
        "dnf".to_string()
    }

    pub fn get_pending_packages(&self, filter_packages: &[String]) -> Vec<PendingPackage> {
        if !filter_packages.is_empty() {
            return filter_packages
                .iter()
                .map(|name| PendingPackage::new(name, "latest"))
                .collect();
        }

        let output = match Command::new(&self.pm_bin).arg("check-update").output() {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_check_update(&stdout)
    }

    fn parse_check_update(&self, output: &str) -> Vec<PendingPackage> {
        let mut packages = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("Updating and loading")
                || trimmed.starts_with("Repositories loaded.")
                || trimmed.starts_with("Security:")
                || trimmed.starts_with("Last metadata")
            {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let pkg_full = parts[0];
                let new_ver = parts[1];
                let repo = if parts.len() > 2 { parts[2] } else { "updates" };

                let (name, arch) = if let Some(idx) = pkg_full.rfind('.') {
                    (&pkg_full[..idx], &pkg_full[idx + 1..])
                } else {
                    (pkg_full, "x86_64")
                };

                let name_lower = name.to_lowercase();
                if name_lower == "name" || name_lower == "package" || name_lower == "obsoleting" {
                    continue;
                }

                packages.push(PendingPackage {
                    name: name.to_string(),
                    new_version: new_ver.to_string(),
                    installed_version: "installed".to_string(),
                    arch: arch.to_string(),
                    repo: repo.to_string(),
                });
            }
        }

        packages
    }
}

pub fn get_demo_packages() -> Vec<PendingPackage> {
    vec![
        PendingPackage {
            name: "kernel".to_string(),
            new_version: "6.10.10-200.fc40".to_string(),
            installed_version: "6.10.9-200.fc40".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
        PendingPackage {
            name: "mesa-dri-drivers".to_string(),
            new_version: "24.1.7-1.fc40".to_string(),
            installed_version: "24.1.6-1.fc40".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
        PendingPackage {
            name: "pipewire".to_string(),
            new_version: "1.2.4-1.fc40".to_string(),
            installed_version: "1.2.3-1.fc40".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
        PendingPackage {
            name: "mutter".to_string(),
            new_version: "46.5-1.fc40".to_string(),
            installed_version: "46.4-1.fc40".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
        PendingPackage {
            name: "firefox".to_string(),
            new_version: "130.0.1-1.fc40".to_string(),
            installed_version: "130.0-1.fc40".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
        PendingPackage {
            name: "gdk-pixbuf2".to_string(),
            new_version: "2.44.6-3.fc44".to_string(),
            installed_version: "2.44.4-1.fc44".to_string(),
            arch: "x86_64".to_string(),
            repo: "updates".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_check_update_output() {
        let raw_output = r#"
Updating and loading repositories:
Repositories loaded.
Security: kernel-core-6.10.10-200.fc40.x86_64 is a security update
mesa-dri-drivers.x86_64               24.1.7-1.fc40                   updates
libheif.x86_64                        1.23.4-6.fc44                   updates
libheif.i686                          1.23.4-6.fc44                   updates
        "#;

        let detector = PackageManagerDetector {
            pm_bin: "dnf5".to_string(),
        };

        let packages = detector.parse_check_update(raw_output);
        assert_eq!(packages.len(), 3);

        assert_eq!(packages[0].name, "mesa-dri-drivers");
        assert_eq!(packages[0].arch, "x86_64");
        assert_eq!(packages[0].new_version, "24.1.7-1.fc40");

        assert_eq!(packages[1].name, "libheif");
        assert_eq!(packages[1].arch, "x86_64");

        assert_eq!(packages[2].name, "libheif");
        assert_eq!(packages[2].arch, "i686");
    }
}
