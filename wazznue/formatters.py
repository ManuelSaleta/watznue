"""Output formatters for CLI (Rich), Markdown, and JSON."""

import json
from typing import Dict, List
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from wazznue.models import CategoryTier, NormalizedChangeItem, Severity


class RichCliFormatter:
    """Renders high-contrast, beautiful terminal output using Rich."""

    def __init__(self, console: Console = None):
        self.console = console or Console()

    def format_digest(self, items: List[NormalizedChangeItem]) -> None:
        if not items:
            self.console.print("\n[bold green]✓ System is fully up to date![/bold green] No pending updates found.\n")
            return

        # Header Banner
        self.console.print()
        header = Text(" ✨ wazznue — Pending Update Digest ", style="bold white on blue")
        self.console.print(header)
        self.console.print(f"[dim]Found {len(items)} package(s) with pending updates.[/dim]\n")

        # Group items by CategoryTier
        grouped: Dict[CategoryTier, List[NormalizedChangeItem]] = {}
        for item in items:
            grouped.setdefault(item.category, []).append(item)

        # Print in priority order
        order = [
            CategoryTier.SECURITY,
            CategoryTier.CORE,
            CategoryTier.DESKTOP,
            CategoryTier.APPLICATIONS,
            CategoryTier.LIBRARIES,
            CategoryTier.OTHER,
        ]

        for cat in order:
            cat_items = grouped.get(cat, [])
            if not cat_items:
                continue

            # Category Header Badge
            if cat == CategoryTier.SECURITY:
                cat_badge = "[bold white on red] 🚨 SECURITY & CRITICAL [/bold white on red]"
            elif cat == CategoryTier.CORE:
                cat_badge = "[bold white on dark_blue] ⚙️  CORE & SYSTEM [/bold white on dark_blue]"
            elif cat == CategoryTier.DESKTOP:
                cat_badge = "[bold black on cyan] 🖥️  DESKTOP & UX [/bold black on cyan]"
            elif cat == CategoryTier.APPLICATIONS:
                cat_badge = "[bold white on magenta] 📦 USER APPLICATIONS [/bold white on magenta]"
            else:
                cat_badge = f"[bold white on grey37] 📚 {cat.value.upper()} [/bold white on grey37]"

            self.console.print(cat_badge)

            for item in cat_items:
                self._render_package_item(item)
            self.console.print()

    def _render_package_item(self, item: NormalizedChangeItem) -> None:
        # Title line: package name + version diff
        pkg_text = Text()
        pkg_text.append(f"• {item.package_name} ", style="bold yellow")
        pkg_text.append(f"({item.installed_version} -> {item.new_version})", style="dim")

        # Severity Pill
        if item.severity == Severity.CRITICAL:
            pkg_text.append(" [CRITICAL]", style="bold red")
        elif item.severity == Severity.HIGH:
            pkg_text.append(" [HIGH]", style="bold red")
        elif item.severity == Severity.MODERATE:
            pkg_text.append(" [MODERATE]", style="bold dark_orange")

        # CVE pills
        for cve in item.cves:
            pkg_text.append(f" [{cve}]", style="bold white on dark_red")

        self.console.print(pkg_text)

        # Bullets
        for bullet in item.bullets:
            self.console.print(f"    - {bullet}", style="bright_white")

        # Bug references
        if item.bugs:
            bug_links = ", ".join([f"RHBZ#{b.bug_id}" for b in item.bugs[:3]])
            self.console.print(f"    [dim]Bugs: {bug_links}[/dim]")


class MarkdownFormatter:
    """Formats digest as clean GitHub Flavored Markdown."""

    @staticmethod
    def format_digest(items: List[NormalizedChangeItem]) -> str:
        lines: List[str] = [
            "# System Update Digest (`wazznue`)",
            f"*Pending updates: {len(items)} package(s)*\n",
        ]

        grouped: Dict[CategoryTier, List[NormalizedChangeItem]] = {}
        for item in items:
            grouped.setdefault(item.category, []).append(item)

        for cat, cat_items in grouped.items():
            lines.append(f"## {cat.value}\n")
            for item in cat_items:
                sev_str = f" `[{item.severity.value}]`" if item.severity != Severity.UNSPECIFIED else ""
                cve_str = " " + " ".join([f"`{c}`" for c in item.cves]) if item.cves else ""
                lines.append(f"### {item.package_name} (`{item.installed_version}` -> `{item.new_version}`){sev_str}{cve_str}")
                for b in item.bullets:
                    lines.append(f"- {b}")
                if item.bugs:
                    bugs_md = ", ".join([f"[RHBZ#{b.bug_id}](https://bugzilla.redhat.com/{b.bug_id})" for b in item.bugs])
                    lines.append(f"- **Resolved Issues:** {bugs_md}")
                lines.append("")

        return "\n".join(lines)


class JsonFormatter:
    """Formats digest as structured JSON."""

    @staticmethod
    def format_digest(items: List[NormalizedChangeItem]) -> str:
        raw_list = []
        for item in items:
            raw_list.append({
                "package": item.package_name,
                "installed_version": item.installed_version,
                "new_version": item.new_version,
                "category": item.category.value,
                "severity": item.severity.value,
                "advisory_id": item.advisory_id,
                "cves": item.cves,
                "bugs": [{"id": b.bug_id, "title": b.title} for b in item.bugs],
                "bullets": item.bullets,
                "source": item.source_name,
            })
        return json.dumps({"updates": raw_list}, indent=2)
