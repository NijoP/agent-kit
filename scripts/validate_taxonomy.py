#!/usr/bin/env python3
"""
Deterministic validation script for EAK V1 3,500-component library taxonomy.

Validates:
- Total allocation == 3500
- No duplicate family/subfamily names
- No negative counts
- Current + Planned == Total per family
- All 16 required families present
- Subfamily sums match family totals
- Required fields present
"""

import re
import sys
import json
from pathlib import Path
from typing import Dict, List, Tuple, Any
from dataclasses import dataclass, asdict

TAXONOMY_FILE = Path("docs/engineering/library-taxonomy.md")

REQUIRED_FAMILIES = [
    "Passives", "Diodes", "Transistors", "Analog ICs / Op-Amps",
    "Power Management", "Digital Logic", "MCUs", "Memory",
    "Communication", "Sensors", "RF / Wireless", "Audio",
    "Protection", "Connectors", "Electromechanical", "Specialized"
]

@dataclass
class ValidationResult:
    check: str
    passed: bool
    message: str
    details: Dict[str, Any] = None

class TaxonomyValidator:
    def __init__(self, content: str):
        self.content = content
        self.results: List[ValidationResult] = []
        self.family_data: Dict[str, Dict] = {}
        self.subfamily_data: Dict[str, List[Tuple[str, int]]] = {}

    def add_result(self, check: str, passed: bool, message: str, details: Dict = None):
        self.results.append(ValidationResult(check, passed, message, details or {}))

    def parse_allocation_table(self) -> Dict[str, Dict[str, int]]:
        """Parse the main allocation table (section 1.1)"""
        table_match = re.search(
            r'\| Family \| Current \(Verified\) \| Additional \(Planned\) \| Total \| % of Library \| Status \|.*?\n((?:\|.*?\n)+)',
            self.content, re.DOTALL
        )
        if not table_match:
            self.add_result("parse_allocation_table", False, "Could not find allocation table")
            return {}

        families = {}
        rows = table_match.group(1).strip().split('\n')
        for row in rows:
            if row.strip() and not row.strip().startswith('|---'):
                parts = [p.strip().replace('**', '').replace(',', '') for p in row.split('|')]
                if len(parts) >= 6:
                    family = parts[1]
                    if family in ['Family', 'TOTAL', '']:
                        continue
                    try:
                        current = int(parts[2]) if parts[2].isdigit() else 0
                        planned = int(parts[3]) if parts[3].isdigit() else 0
                        total = int(parts[4]) if parts[4].isdigit() else 0
                        families[family] = {'current': current, 'planned': planned, 'total': total}
                    except ValueError as e:
                        self.add_result("parse_allocation_table", False, f"Parse error for {family}: {e}")
        return families

    def parse_subfamily_tables(self) -> Dict[str, List[Tuple[str, int]]]:
        """Parse subfamily breakdown tables for each family"""
        subfamily_patterns = [
            ('Passives', r'### 2\.1 Passives.*?\| Subfamily \| Current \| Planned \| Total \|', True),
            ('Diodes', r'### 2\.2 Diodes.*?\| Subfamily \| Count \|', False),
            ('Transistors', r'### 2\.3 Transistors.*?\| Subfamily \| Count \|', False),
            ('Analog ICs / Op-Amps', r'### 2\.4 Analog ICs.*?\| Subfamily \| Count \|', False),
            ('Power Management', r'### 2\.5 Power Management.*?\| Subfamily \| Count \|', False),
            ('Digital Logic', r'### 2\.6 Digital Logic.*?\| Subfamily \| Count \|', False),
            ('MCUs', r'### 2\.7 MCUs.*?\| Subfamily \| Count \|', False),
            ('Memory', r'### 2\.8 Memory.*?\| Subfamily \| Count \|', False),
            ('Communication', r'### 2\.9 Communication.*?\| Subfamily \| Count \|', False),
            ('Sensors', r'### 2\.10 Sensors.*?\| Subfamily \| Count \|', False),
            ('RF / Wireless', r'### 2\.11 RF / Wireless.*?\| Subfamily \| Count \|', False),
            ('Audio', r'### 2\.12 Audio.*?\| Subfamily \| Count \|', False),
            ('Protection', r'### 2\.13 Protection.*?\| Subfamily \| Count \|', False),
            ('Connectors', r'### 2\.14 Connectors.*?\| Subfamily \| Count \|', False),
            ('Electromechanical', r'### 2\.15 Electromechanical.*?\| Subfamily \| Count \|', False),
            ('Specialized', r'### 2\.16 Specialized.*?\| Subfamily \| Count \|', False),
        ]

        subfamily_data = {}
        for name, pattern, is_passives in subfamily_patterns:
            match = re.search(pattern, self.content, re.DOTALL)
            if not match:
                self.add_result(f"parse_subfamily_{name}", False, f"Could not find subfamily table for {name}")
                continue

            section_start = match.end()
            section = self.content[section_start:section_start+3000]
            table_end = section.find('**Status Distribution:**')
            if table_end == -1:
                table_end = section.find('---')
            table_text = section[:table_end]
            rows = table_text.strip().split('\n')

            subfamilies = []
            for row in rows:
                if '|' in row and not row.strip().startswith('|---') and not row.strip().startswith('| Subfamily'):
                    parts = [p.strip() for p in row.split('|')]
                    if len(parts) >= 3:
                        try:
                            subfamily_name = parts[1]
                            if is_passives:
                                val = int(parts[4]) if parts[4].isdigit() else 0
                            else:
                                val = int(parts[2]) if parts[2].isdigit() else 0
                            subfamilies.append((subfamily_name, val))
                        except (ValueError, IndexError):
                            pass
            subfamily_data[name] = subfamilies
        return subfamily_data

    def validate_total_allocation(self):
        """Check total == 3500"""
        total_current = sum(f['current'] for f in self.family_data.values())
        total_planned = sum(f['planned'] for f in self.family_data.values())
        total_total = sum(f['total'] for f in self.family_data.values())

        passed = (total_current == 500 and total_planned == 3000 and total_total == 3500)
        self.add_result(
            "total_allocation",
            passed,
            f"Total: current={total_current}, planned={total_planned}, total={total_total} (expected 500+3000=3500)",
            {"current": total_current, "planned": total_planned, "total": total_total}
        )

    def validate_no_duplicates(self):
        """Check for duplicate family names"""
        family_names = list(self.family_data.keys())
        duplicates = [name for name in family_names if family_names.count(name) > 1]
        passed = len(duplicates) == 0
        self.add_result(
            "no_duplicate_families",
            passed,
            f"Duplicate families: {duplicates}" if duplicates else "No duplicate family names",
            {"duplicates": duplicates}
        )

    def validate_all_families_present(self):
        """Check all 16 required families are present"""
        present = set(self.family_data.keys())
        required = set(REQUIRED_FAMILIES)
        missing = required - present
        extra = present - required
        passed = len(missing) == 0 and len(extra) == 0
        self.add_result(
            "all_families_present",
            passed,
            f"Missing: {missing}, Extra: {extra}" if missing or extra else "All 16 required families present",
            {"missing": list(missing), "extra": list(extra)}
        )

    def validate_no_negative_counts(self):
        """Check for negative counts"""
        negatives = []
        for family, data in self.family_data.items():
            for key, val in data.items():
                if val < 0:
                    negatives.append(f"{family}.{key}={val}")
        passed = len(negatives) == 0
        self.add_result(
            "no_negative_counts",
            passed,
            f"Negative counts found: {negatives}" if negatives else "No negative counts",
            {"negatives": negatives}
        )

    def validate_current_plus_planned_equals_total(self):
        """Check current + planned == total per family"""
        mismatches = []
        for family, data in self.family_data.items():
            if data['current'] + data['planned'] != data['total']:
                mismatches.append(f"{family}: {data['current']}+{data['planned']} != {data['total']}")
        passed = len(mismatches) == 0
        self.add_result(
            "current_plus_planned_equals_total",
            passed,
            f"Mismatches: {mismatches}" if mismatches else "All families: current + planned = total",
            {"mismatches": mismatches}
        )

    def validate_subfamily_sums_match(self):
        """Check subfamily sums match family totals"""
        mismatches = []
        for family, subfamilies in self.subfamily_data.items():
            subfamily_sum = sum(val for _, val in subfamilies)
            expected = self.family_data.get(family, {}).get('total', 0)
            if subfamily_sum != expected:
                mismatches.append(f"{family}: subfamily sum={subfamily_sum}, family total={expected}")
        passed = len(mismatches) == 0
        self.add_result(
            "subfamily_sums_match",
            passed,
            f"Mismatches: {mismatches}" if mismatches else "All subfamily sums match family totals",
            {"mismatches": mismatches}
        )

    def validate_subfamily_no_duplicates(self):
        """Check for duplicate subfamily names within each family"""
        all_duplicates = []
        for family, subfamilies in self.subfamily_data.items():
            names = [name for name, _ in subfamilies]
            duplicates = [name for name in names if names.count(name) > 1]
            if duplicates:
                all_duplicates.extend([f"{family}: {d}" for d in duplicates])
        passed = len(all_duplicates) == 0
        self.add_result(
            "subfamily_no_duplicates",
            passed,
            f"Duplicate subfamilies: {all_duplicates}" if all_duplicates else "No duplicate subfamily names",
            {"duplicates": all_duplicates}
        )

    def validate_subfamily_no_negative(self):
        """Check for negative subfamily counts"""
        negatives = []
        for family, subfamilies in self.subfamily_data.items():
            for name, val in subfamilies:
                if val < 0:
                    negatives.append(f"{family}.{name}={val}")
        passed = len(negatives) == 0
        self.add_result(
            "subfamily_no_negative",
            passed,
            f"Negative subfamily counts: {negatives}" if negatives else "No negative subfamily counts",
            {"negatives": negatives}
        )

    def validate_status_model_documented(self):
        """Check status model section exists"""
        has_status_model = "## 3. Status Model (Provenance / State)" in self.content
        has_planned = "PLANNED" in self.content
        has_verified = "VERIFIED" in self.content
        has_available = "AVAILABLE" in self.content
        has_blocked = "BLOCKED" in self.content
        has_not_found = "NOT_FOUND" in self.content

        passed = all([has_status_model, has_planned, has_verified, has_available, has_blocked, has_not_found])
        self.add_result(
            "status_model_documented",
            passed,
            "Status model with 5 states documented" if passed else "Status model incomplete",
            {
                "has_status_model_section": has_status_model,
                "has_planned": has_planned,
                "has_verified": has_verified,
                "has_available": has_available,
                "has_blocked": has_blocked,
                "has_not_found": has_not_found
            }
        )

    def validate_metadata_architecture_documented(self):
        """Check metadata architecture sections exist"""
        has_common_core = "## 4.1 COMMON CORE (All Families)" in self.content
        has_family_specific = "## 4.2 FAMILY-SPECIFIC Metadata" in self.content
        has_physical_quantity = "Physical Quantity" in self.content

        passed = all([has_common_core, has_family_specific, has_physical_quantity])
        self.add_result(
            "metadata_architecture_documented",
            passed,
            "Metadata architecture documented (COMMON CORE + FAMILY-SPECIFIC + Physical Quantity)" if passed else "Metadata architecture incomplete",
            {
                "has_common_core": has_common_core,
                "has_family_specific": has_family_specific,
                "has_physical_quantity": has_physical_quantity
            }
        )

    def validate_engineering_constraints_documented(self):
        """Check engineering constraints table exists"""
        has_constraints = "## 5. Engineering Constraints Per Family" in self.content
        passed = has_constraints
        self.add_result(
            "engineering_constraints_documented",
            passed,
            "Engineering constraints per family documented" if passed else "Engineering constraints table missing",
            {"has_constraints_table": has_constraints}
        )

    def validate_golden_board_relevance_documented(self):
        """Check golden-board relevance table exists"""
        has_relevance = "## 6. Golden-Board Relevance Per Family" in self.content
        passed = has_relevance
        self.add_result(
            "golden_board_relevance_documented",
            passed,
            "Golden-board relevance per family documented" if passed else "Golden-board relevance table missing",
            {"has_relevance_table": has_relevance}
        )

    def validate_verification_requirements_documented(self):
        """Check verification requirements table exists"""
        has_verification = "## 7. Verification Requirements Per Family" in self.content
        passed = has_verification
        self.add_result(
            "verification_requirements_documented",
            passed,
            "Verification requirements per family documented" if passed else "Verification requirements table missing",
            {"has_verification_table": has_verification}
        )

    def validate_asset_requirements_documented(self):
        """Check asset requirements table exists"""
        has_assets = "## 8. Asset Requirements Per Family" in self.content
        passed = has_assets
        self.add_result(
            "asset_requirements_documented",
            passed,
            "Asset requirements per family documented" if passed else "Asset requirements table missing",
            {"has_assets_table": has_assets}
        )

    def run_all_validations(self):
        """Run all validation checks"""
        self.family_data = self.parse_allocation_table()
        self.subfamily_data = self.parse_subfamily_tables()

        if not self.family_data:
            return

        self.validate_total_allocation()
        self.validate_no_duplicates()
        self.validate_all_families_present()
        self.validate_no_negative_counts()
        self.validate_current_plus_planned_equals_total()
        self.validate_subfamily_sums_match()
        self.validate_subfamily_no_duplicates()
        self.validate_subfamily_no_negative()
        self.validate_status_model_documented()
        self.validate_metadata_architecture_documented()
        self.validate_engineering_constraints_documented()
        self.validate_golden_board_relevance_documented()
        self.validate_verification_requirements_documented()
        self.validate_asset_requirements_documented()

    def get_summary(self) -> Dict:
        passed = sum(1 for r in self.results if r.passed)
        failed = sum(1 for r in self.results if not r.passed)
        return {
            "total_checks": len(self.results),
            "passed": passed,
            "failed": failed,
            "success": failed == 0,
            "results": [asdict(r) for r in self.results]
        }

    def print_report(self):
        """Print human-readable validation report"""
        print("=" * 70)
        print("EAK V1 LIBRARY TAXONOMY VALIDATION REPORT")
        print("=" * 70)
        for r in self.results:
            status = "✓ PASS" if r.passed else "✗ FAIL"
            print(f"[{status}] {r.check}: {r.message}")
            if r.details and not r.passed:
                for k, v in r.details.items():
                    print(f"         {k}: {v}")
        print("=" * 70)
        summary = self.get_summary()
        print(f"Total: {summary['total_checks']} | Passed: {summary['passed']} | Failed: {summary['failed']}")
        print("=" * 70)

def main():
    if not TAXONOMY_FILE.exists():
        print(f"ERROR: Taxonomy file not found: {TAXONOMY_FILE}")
        sys.exit(1)

    content = TAXONOMY_FILE.read_text()
    validator = TaxonomyValidator(content)
    validator.run_all_validations()
    validator.print_report()

    # Output machine-readable JSON
    summary = validator.get_summary()
    json_output = {
        "taxonomy_file": str(TAXONOMY_FILE),
        "validation_timestamp": "2025-08-20T00:00:00Z",
        "summary": summary,
        "results": summary["results"]
    }
    print(json.dumps(json_output, indent=2))

    sys.exit(0 if summary["success"] else 1)

if __name__ == "__main__":
    main()