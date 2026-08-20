#!/usr/bin/env python3
"""
Semantic validation of the Day 1 passive component CSV.
Checks for cross-column contamination and semantic correctness.
"""

import csv
import sys
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ValidationError:
    row: int
    mpn: str
    field: str
    expected: str
    actual: str
    message: str

def validate_csv(csv_path: str) -> List[ValidationError]:
    errors = []
    
    with open(csv_path, 'r') as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader, start=2):  # start=2 because header is row 1
            mpn = row['mpn']
            category = row['category']
            resistance = row['resistance_ohm']
            capacitance = row['capacitance_f']
            inductance = row['inductance_h']
            voltage = row['voltage_v']
            power = row['power_w']
            tolerance = row['tolerance']
            temp_coeff = row['temp_coeff']
            subcategory = row['subcategory']
            pkg = row['package']
            
            # Convert to float for numeric checks
            def to_float(v: str) -> Optional[float]:
                try:
                    return float(v) if v else None
                except ValueError:
                    return None
            
            r_val = to_float(resistance)
            c_val = to_float(capacitance)
            l_val = to_float(inductance)
            v_val = to_float(voltage)
            p_val = to_float(power)
            
            # Cross-column contamination checks
            if category == 'Resistor':
                if c_val is not None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='capacitance_f',
                        expected='NULL/empty', actual=str(c_val),
                        message=f"Resistor {mpn} has capacitance_f={c_val} (should be NULL)"
                    ))
                if l_val is not None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='inductance_h',
                        expected='NULL/empty', actual=str(l_val),
                        message=f"Resistor {mpn} has inductance_h={l_val} (should be NULL)"
                    ))
                # Resistor must have resistance
                if r_val is None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='resistance_ohm',
                        expected='numeric value', actual='NULL/empty',
                        message=f"Resistor {mpn} missing resistance_ohm"
                    ))
                # Resistor power should be reasonable
                if p_val is not None:
                    if p_val < 0.01 or p_val > 2.0:
                        errors.append(ValidationError(
                            row=i, mpn=mpn, field='power_w',
                            expected='0.01-2.0 W', actual=str(p_val),
                            message=f"Resistor {mpn} power_w={p_val} W seems unreasonable"
                        ))
            
            elif category == 'Capacitor':
                if r_val is not None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='resistance_ohm',
                        expected='NULL/empty', actual=str(r_val),
                        message=f"Capacitor {mpn} has resistance_ohm={r_val} (should be NULL)"
                    ))
                if l_val is not None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='inductance_h',
                        expected='NULL/empty', actual=str(l_val),
                        message=f"Capacitor {mpn} has inductance_h={l_val} (should be NULL)"
                    ))
                # Capacitor must have capacitance
                if c_val is None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='capacitance_f',
                        expected='numeric value', actual='NULL/empty',
                        message=f"Capacitor {mpn} missing capacitance_f"
                    ))
                # Capacitor should have voltage
                if v_val is None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='voltage_v',
                        expected='numeric value', actual='NULL/empty',
                        message=f"Capacitor {mpn} missing voltage_v"
                    ))
            
            elif category == 'Inductor':
                if c_val is not None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='capacitance_f',
                        expected='NULL/empty', actual=str(c_val),
                        message=f"Inductor {mpn} has capacitance_f={c_val} (should be NULL)"
                    ))
                if r_val is not None:
                    # DCR is acceptable for inductors, but let's flag if it looks like a resistance value
                    if r_val > 100:  # Arbitrary threshold - DCR should be low
                        errors.append(ValidationError(
                            row=i, mpn=mpn, field='resistance_ohm',
                            expected='DCR (typically < 100 ohm) or NULL', actual=str(r_val),
                            message=f"Inductor {mpn} has resistance_ohm={r_val} - may be cross-contamination"
                        ))
                # Inductor must have inductance
                if l_val is None:
                    errors.append(ValidationError(
                        row=i, mpn=mpn, field='inductance_h',
                        expected='numeric value', actual='NULL/empty',
                        message=f"Inductor {mpn} missing inductance_h"
                    ))
            
            # Package validation
            valid_packages = ['0402', '0603', '0805', '1206']
            if pkg and pkg not in valid_packages:
                errors.append(ValidationError(
                    row=i, mpn=mpn, field='package',
                    expected='one of ' + ', '.join(valid_packages), actual=pkg,
                    message=f"Part {mpn} has unknown package: {pkg}"
                ))
            
            # Tolerance validation
            if tolerance:
                if category in ['Resistor', 'Capacitor']:
                    if not any(t in tolerance for t in ['%', 'ppm']):
                        pass  # Could be more validation here
            
            # Temperature coefficient validation
            if temp_coeff and 'ppm' not in temp_coeff.lower() and '°c' not in temp_coeff.lower():
                errors.append(ValidationError(
                    row=i, mpn=mpn, field='temp_coeff',
                    expected='ppm/°C format', actual=temp_coeff,
                    message=f"Part {mpn} temp_coeff='{temp_coeff}' doesn't look like ppm/°C format"
                ))
    
    return errors

def main():
    csv_path = "/home/dev/electronics-agent-kit/data/imports/day1_passives.csv"
    
    print(f"Validating CSV: {csv_path}")
    errors = validate_csv(csv_path)
    
    if errors:
        print(f"\nFound {len(errors)} semantic validation errors:")
        for err in errors[:50]:  # Show first 50
            print(f"  Row {err.row}: {err.mpn} - {err.field} = {err.actual} (expected {err.expected})")
            print(f"    {err.message}")
        
        if len(errors) > 50:
            print(f"  ... and {len(errors) - 50} more errors")
        
        # Group by error type
        print("\nError summary by type:")
        error_types = {}
        for err in errors:
            error_types[err.field] = error_types.get(err.field, 0) + 1
        for field, count in sorted(error_types.items()):
            print(f"  {field}: {count}")
        
        return 1
    else:
        print("✓ No semantic validation errors found!")
        return 0

if __name__ == "__main__":
    sys.exit(main())