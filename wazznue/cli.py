"""CLI entry point for wazznue."""

import argparse
import sys
from typing import List

from wazznue.detectors import PackageManagerDetector, get_demo_packages
from wazznue.formatters import JsonFormatter, MarkdownFormatter, RichCliFormatter
from wazznue.models import CategoryTier, NormalizedChangeItem, PendingPackage
from wazznue.normalizer import ChangelogNormalizer
from wazznue.sources import SourceAggregator


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="wazznue",
        description="wazznue — Modular, lean update digest & changelog inspector.",
    )
    parser.add_argument(
        "packages",
        nargs="*",
        help="Optional package names to inspect. If omitted, checks system pending updates.",
    )
    parser.add_argument(
        "--demo",
        action="store_true",
        help="Run against a demo set of packages to preview output even when system is up to date.",
    )
    parser.add_argument(
        "--security-only",
        action="store_true",
        help="Display only security-related updates and CVE fixes.",
    )
    parser.add_argument(
        "--category",
        choices=["security", "core", "desktop", "apps", "libs"],
        help="Filter updates by specific category.",
    )
    parser.add_argument(
        "--format",
        choices=["cli", "markdown", "json"],
        default="cli",
        help="Output format (default: cli).",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    # 1. Discover packages
    detector = PackageManagerDetector()
    if args.demo:
        pending = get_demo_packages()
    elif args.packages:
        pending = detector.get_pending_packages(filter_packages=args.packages)
    else:
        pending = detector.get_pending_packages()

    if not pending and not args.demo:
        if args.format == "json":
            print(JsonFormatter.format_digest([]))
        elif args.format == "markdown":
            print(MarkdownFormatter.format_digest([]))
        else:
            RichCliFormatter().format_digest([])
        return

    # 2. Source aggregation & normalization
    aggregator = SourceAggregator()
    normalized_items: List[NormalizedChangeItem] = []

    for pkg in pending:
        # Query Bodhi
        bodhi_data = aggregator.fetch_bodhi_update(pkg.name)

        # Fallback to local changelog if needed
        local_changelog = None
        if not bodhi_data or not bodhi_data.get("notes"):
            local_changelog = aggregator.fetch_rpm_changelog(pkg.name)

        item = ChangelogNormalizer.normalize(
            pkg=pkg,
            bodhi_data=bodhi_data,
            raw_changelog=local_changelog,
        )

        # Apply filtering
        if args.security_only and item.category != CategoryTier.SECURITY:
            continue

        if args.category:
            cat_map = {
                "security": CategoryTier.SECURITY,
                "core": CategoryTier.CORE,
                "desktop": CategoryTier.DESKTOP,
                "apps": CategoryTier.APPLICATIONS,
                "libs": CategoryTier.LIBRARIES,
            }
            if item.category != cat_map.get(args.category):
                continue

        normalized_items.append(item)

    # 3. Format and output
    if args.format == "json":
        print(JsonFormatter.format_digest(normalized_items))
    elif args.format == "markdown":
        print(MarkdownFormatter.format_digest(normalized_items))
    else:
        RichCliFormatter().format_digest(normalized_items)


if __name__ == "__main__":
    main()
