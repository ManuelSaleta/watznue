mod formatters;

use clap::Parser;
use formatters::{CliFormatter, JsonFormatter, MarkdownFormatter};
use watznue_core::detector::{get_demo_packages, PackageManagerDetector};
use watznue_core::models::CategoryTier;
use watznue_core::normalizer::ChangelogNormalizer;
use watznue_core::sources::SourceAggregator;
use watznue_core::spinner::start_loading_spinner;


#[derive(Parser, Debug)]
#[command(
    name = "watznue",
    about = "watznue — Modular, lean update digest & changelog inspector.",
    version
)]
struct Args {
    /// Optional package names to inspect. If omitted, checks system pending updates.
    #[arg(value_name = "PACKAGES")]
    packages: Vec<String>,

    /// Run against a demo set of packages to preview output even when system is up to date.
    #[arg(long)]
    demo: bool,

    /// Display only security-related updates and CVE fixes.
    #[arg(long)]
    security_only: bool,

    /// Filter updates by specific category (security, core, desktop, apps, libs).
    #[arg(long, value_name = "CATEGORY")]
    category: Option<String>,

    /// Output format (cli, markdown, json).
    #[arg(long, default_value = "cli", value_name = "FORMAT")]
    format: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 1. Detect pending packages & normalize changes in worker thread
    let demo = args.demo;
    let packages = args.packages.clone();
    let security_only = args.security_only;
    let category = args.category.clone();
    let handle = tokio::runtime::Handle::current();

    let pending_worker = std::thread::spawn(move || {
        let pending = if demo {
            get_demo_packages()
        } else {
            let detector = PackageManagerDetector::new();
            detector.get_pending_packages(&packages)
        };

        if pending.is_empty() && !demo {
            return (pending, Vec::new());
        }

        // 2. Aggregate sources & normalize
        let normalized_items = handle.block_on(async {
            let mut aggregator = SourceAggregator::new();
            let mut items = Vec::new();

            for pkg in &pending {
                let bodhi_data = aggregator.fetch_bodhi_update(&pkg.name).await;

                let local_changelog = if bodhi_data.as_ref().and_then(|b| b.notes.as_ref()).is_none() {
                    aggregator.fetch_rpm_changelog(&pkg.name, 25)
                } else {
                    None
                };

                let item = ChangelogNormalizer::normalize(pkg, bodhi_data.as_ref(), local_changelog.as_deref());

                // Apply filters
                if security_only && item.category != CategoryTier::Security {
                    continue;
                }

                if let Some(ref cat_filter) = category {
                    let target_cat = match cat_filter.to_lowercase().as_str() {
                        "security" => CategoryTier::Security,
                        "core" => CategoryTier::Core,
                        "desktop" => CategoryTier::Desktop,
                        "apps" => CategoryTier::Applications,
                        "libs" => CategoryTier::Libraries,
                        _ => CategoryTier::Other,
                    };
                    if item.category != target_cat {
                        continue;
                    }
                }

                items.push(item);
            }

            items
        });

        (pending, normalized_items)
    });

    // loading animation
    start_loading_spinner(|| !pending_worker.is_finished());

    let (pending, normalized_items) = pending_worker
        .join()
        .unwrap_or_else(|_| (Vec::new(), Vec::new()));

    // 3. Render output
    if pending.is_empty() && !args.demo {
        match args.format.as_str() {
            "json" => println!("{}", JsonFormatter::format_digest(&[])),
            "markdown" => println!("{}", MarkdownFormatter::format_digest(&[])),
            _ => CliFormatter::format_digest(&[]),
        }
        return;
    }

    match args.format.as_str() {
        "json" => println!("{}", JsonFormatter::format_digest(&normalized_items)),
        "markdown" => println!("{}", MarkdownFormatter::format_digest(&normalized_items)),
        _ => CliFormatter::format_digest(&normalized_items),
    }
}
