use colored::Colorize;
use std::collections::HashMap;
use watznue_core::models::{CategoryTier, NormalizedChangeItem, Severity};

pub struct CliFormatter;

impl CliFormatter {
    pub fn format_digest(items: &[NormalizedChangeItem]) {
        if items.is_empty() {
            println!("\n{} No pending updates found.\n", "System is up to date.".green().bold());
            return;
        }

        println!();
        println!(
            "{}",
            " watznue — Pending Update Digest "
                .on_blue()
                .white()
                .bold()
        );
        println!(
            "{}",
            format!("Found {} package(s) with pending updates.\n", items.len()).dimmed()
        );

        let mut grouped: HashMap<CategoryTier, Vec<&NormalizedChangeItem>> = HashMap::new();
        for item in items {
            grouped.entry(item.category).or_default().push(item);
        }

        let order = [
            CategoryTier::Security,
            CategoryTier::Core,
            CategoryTier::Desktop,
            CategoryTier::Applications,
            CategoryTier::Libraries,
            CategoryTier::Other,
        ];

        for cat in order {
            if let Some(cat_items) = grouped.get(&cat) {
                let badge = match cat {
                    CategoryTier::Security => "[SECURITY & CRITICAL]".on_red().white().bold(),
                    CategoryTier::Core => "[CORE & SYSTEM]".on_blue().white().bold(),
                    CategoryTier::Desktop => "[DESKTOP & UX]".on_cyan().black().bold(),
                    CategoryTier::Applications => "[APPLICATIONS]".on_magenta().white().bold(),
                    _ => cat.badge_label().on_bright_black().white().bold(),
                };

                println!("{}", badge);

                for item in cat_items {
                    Self::render_package_item(item);
                }
                println!();
            }
        }
    }

    fn render_package_item(item: &NormalizedChangeItem) {
        let mut title = format!(
            "• {} ({})",
            item.package_name.yellow().bold(),
            format!("{} -> {}", item.installed_version, item.new_version).dimmed()
        );

        match item.severity {
            Severity::Critical => {
                title.push_str(&format!(" {}", "[CRITICAL]".red().bold()));
            }
            Severity::High => {
                title.push_str(&format!(" {}", "[HIGH]".red().bold()));
            }
            Severity::Moderate => {
                title.push_str(&format!(" {}", "[MODERATE]".yellow().bold()));
            }
            _ => {}
        }

        for cve in &item.cves {
            title.push_str(&format!(" {}", format!("[{}]", cve).on_red().white().bold()));
        }

        println!("{}", title);

        for bullet in &item.bullets {
            println!("    - {}", bullet);
        }

        if !item.bugs.is_empty() {
            let bug_links: Vec<String> = item
                .bugs
                .iter()
                .take(3)
                .map(|b| format!("RHBZ#{}", b.bug_id))
                .collect();
            println!("    {}", format!("Bugs: {}", bug_links.join(", ")).dimmed());
        }
    }
}

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format_digest(items: &[NormalizedChangeItem]) -> String {
        let mut lines = vec![
            "# System Update Digest (`watznue`)".to_string(),
            format!("*Pending updates: {} package(s)*\n", items.len()),
        ];

        let mut grouped: HashMap<CategoryTier, Vec<&NormalizedChangeItem>> = HashMap::new();
        for item in items {
            grouped.entry(item.category).or_default().push(item);
        }

        let order = [
            CategoryTier::Security,
            CategoryTier::Core,
            CategoryTier::Desktop,
            CategoryTier::Applications,
            CategoryTier::Libraries,
            CategoryTier::Other,
        ];

        for cat in order {
            if let Some(cat_items) = grouped.get(&cat) {
                lines.push(format!("## {}\n", cat.as_str()));

                for item in cat_items {
                    let sev_str = if item.severity != Severity::Unspecified {
                        format!(" `[{}]`", item.severity.as_str())
                    } else {
                        String::new()
                    };

                    let cve_str = if !item.cves.is_empty() {
                        let cve_list: Vec<String> = item.cves.iter().map(|c| format!("`{}`", c)).collect();
                        format!(" {}", cve_list.join(" "))
                    } else {
                        String::new()
                    };

                    lines.push(format!(
                        "### {} (`{}` -> `{}`){}{}",
                        item.package_name, item.installed_version, item.new_version, sev_str, cve_str
                    ));

                    for bullet in &item.bullets {
                        lines.push(format!("- {}", bullet));
                    }

                    if !item.bugs.is_empty() {
                        let bug_links: Vec<String> = item
                            .bugs
                            .iter()
                            .map(|b| format!("[RHBZ#{}](https://bugzilla.redhat.com/{})", b.bug_id, b.bug_id))
                            .collect();
                        lines.push(format!("- **Resolved Issues:** {}", bug_links.join(", ")));
                    }
                    lines.push(String::new());
                }
            }
        }

        lines.join("\n")
    }
}

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format_digest(items: &[NormalizedChangeItem]) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "updates": items
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}
