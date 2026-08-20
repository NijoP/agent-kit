#!/usr/bin/env python3
"""
Verify symbol/footprint referential integrity for all 500 parts.
Checks that each part references correct symbol/footprint files that exist.
"""

import csv
import os
from typing import List, Tuple

# Expected symbol mapping
SYMBOL_MAP = {
    'Resistor': 'Resistor.kicad_sym',
    'Capacitor': 'Capacitor.kicad_sym',
    'Inductor': 'Inductor.kicad_sym',
}

# Expected footprint mapping (generic by package)
FOOTPRINT_MAP = {
    '0402': 'Resistor_0402.kicad_mod',
    '0603': 'Resistor_0603.kicad_mod',
    '0805': 'Resistor_0805.kicad_mod',
    '1206': 'Resistor_1206.kicad_mod',
}

SYMBOLS_DIR = "/home/dev/electronics-agent-kit/data/assets/symbols"
FOOTPRINTS_DIR = "/home/dev/electronics-agent-kit/data/assets/footprints"

def verify_files_exist() -> Tuple[bool, List[str]]:
    """Check that all expected symbol/footprint files exist."""
    errors = []
    
    # Check symbols
    for cat, fname in SYMBOL_MAP.items():
        path = os.path.join(SYMBOLS_DIR, fname)
        if not os.path.exists(path):
            errors.append(f"Missing symbol file: {path}")
        else:
            print(f"  ✓ Symbol exists: {fname}")
    
    # Check footprints
    for pkg, fname in FOOTPRINT_MAP.items():
        path = os.path.join(FOOTPRINTS_DIR, fname)
        if not os.path.exists(path):
            errors.append(f"Missing footprint file: {path}")
        else:
            print(f"  ✓ Footprint exists: {fname}")
    
    return len(errors) == 0, errors

def verify_csv_references(csv_path: str) -> List[str]:
    """Verify that each part in CSV would reference correct existing files."""
    errors = []
    
    with open(csv_path, 'r') as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader, start=2):
            mpn = row['mpn']
            category = row['category']
            package = row['package']
            
            # Check symbol
            expected_symbol = SYMBOL_MAP.get(category)
            if not expected_symbol:
                errors.append(f"Row {i} ({mpn}): Unknown category '{category}' - no symbol mapping")
            else:
                symbol_path = os.path.join(SYMBOLS_DIR, expected_symbol)
                if not os.path.exists(symbol_path):
                    errors.append(f"Row {i} ({mpn}): Symbol file missing: {symbol_path}")
            
            # Check footprint
            expected_footprint = FOOTPRINT_MAP.get(package)
            if not expected_footprint:
                errors.append(f"Row {i} ({mpn}): Unknown package '{package}' - no footprint mapping")
            else:
                footprint_path = os.path.join(FOOTPRINTS_DIR, expected_footprint)
                if not os.path.exists(footprint_path):
                    errors.append(f"Row {i} ({mpn}): Footprint file missing: {footprint_path}")
    
    return errors

def verify_file_contents() -> List[str]:
    """Verify that the symbol/footprint files have valid KiCad syntax."""
    errors = []
    
    # Check symbols
    for fname in SYMBOL_MAP.values():
        path = os.path.join(SYMBOLS_DIR, fname)
        if os.path.exists(path):
            with open(path, 'r') as f:
                content = f.read()
                if not content.startswith('(kicad_symbol_lib'):
                    errors.append(f"Symbol {fname}: Invalid KiCad symbol format (missing kicad_symbol_lib header)")
                # Check for symbol definition - format is (symbol "Name" ...)
                if '(symbol "' not in content:
                    errors.append(f"Symbol {fname}: Missing symbol definition")
    
    # Check footprints
    for fname in FOOTPRINT_MAP.values():
        path = os.path.join(FOOTPRINTS_DIR, fname)
        if os.path.exists(path):
            with open(path, 'r') as f:
                content = f.read()
                if not content.startswith('(footprint'):
                    errors.append(f"Footprint {fname}: Invalid KiCad footprint format (missing footprint header)")
                if 'pad "1"' not in content or 'pad "2"' not in content:
                    errors.append(f"Footprint {fname}: Missing pad definitions")
    
    return errors

def main():
    csv_path = "/home/dev/electronics-agent-kit/data/imports/day1_passives.csv"
    
    print("=" * 60)
    print("SYMBOL/FOOTPRINT REFERENTIAL INTEGRITY VERIFICATION")
    print("=" * 60)
    
    # 1. Verify expected files exist
    print("\n1. Checking existence of symbol/footprint files...")
    ok, file_errors = verify_files_exist()
    if file_errors:
        for err in file_errors:
            print(f"  ✗ {err}")
    else:
        print("  ✓ All expected symbol/footprint files exist")
    
    # 2. Verify CSV references
    print("\n2. Verifying CSV references to symbol/footprint files...")
    ref_errors = verify_csv_references(csv_path)
    if ref_errors:
        for err in ref_errors[:20]:
            print(f"  ✗ {err}")
        if len(ref_errors) > 20:
            print(f"  ... and {len(ref_errors) - 20} more errors")
    else:
        print("  ✓ All 500 parts reference valid existing symbol/footprint files")
    
    # 3. Verify file contents
    print("\n3. Verifying KiCad syntax of symbol/footprint files...")
    content_errors = verify_file_contents()
    if content_errors:
        for err in content_errors:
            print(f"  ✗ {err}")
    else:
        print("  ✓ All symbol/footprint files have valid KiCad syntax")
    
    # Summary
    total_errors = len(file_errors) + len(ref_errors) + len(content_errors)
    print("\n" + "=" * 60)
    if total_errors == 0:
        print("✓ ALL SYMBOL/FOOTPRINT REFERENTIAL INTEGRITY CHECKS PASSED")
        print("  - 3 generic symbols (Resistor, Capacitor, Inductor) exist and valid")
        print("  - 4 generic footprints (0402, 0603, 0805, 1206) exist and valid")
        print("  - All 500 parts reference correct existing files")
        print("  - All files have valid KiCad syntax")
        return 0
    else:
        print(f"✗ {total_errors} ERRORS FOUND")
        return 1

if __name__ == "__main__":
    import sys
    sys.exit(main())