#!/usr/bin/env python3
"""
Generate Day 1 passive component CSV with proper column alignment and validation.
"""

import csv
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class Part:
    lcsc_id: Optional[str]
    mpn: str
    manufacturer: str
    category: str  # Resistor, Capacitor, Inductor
    subcategory: str
    package: str
    voltage_v: Optional[float]
    capacitance_f: Optional[float]
    resistance_ohm: Optional[float]
    inductance_h: Optional[float]
    tolerance: str
    temp_coeff: str
    power_w: Optional[float]
    stock: Optional[int]
    price_cny: Optional[float]
    moq: Optional[int]
    description: str
    keywords: str

    def to_row(self):
        return [
            self.lcsc_id or "",
            self.mpn,
            self.manufacturer,
            self.category,
            self.subcategory,
            self.package,
            self.voltage_v if self.voltage_v is not None else "",
            self.capacitance_f if self.capacitance_f is not None else "",
            self.resistance_ohm if self.resistance_ohm is not None else "",
            self.inductance_h if self.inductance_h is not None else "",
            self.tolerance,
            self.temp_coeff,
            self.power_w if self.power_w is not None else "",
            self.stock if self.stock is not None else "",
            self.price_cny if self.price_cny is not None else "",
            self.moq if self.moq is not None else "",
            self.description,
            self.keywords,
        ]

HEADERS = [
    "lcsc_id", "mpn", "manufacturer", "category", "subcategory", "package",
    "voltage_v", "capacitance_f", "resistance_ohm", "inductance_h",
    "tolerance", "temp_coeff", "power_w", "stock", "price_cny", "moq",
    "description", "keywords"
]

def validate_parts(parts: List[Part]) -> bool:
    """Validate all parts have correct column mapping."""
    errors = []
    
    for i, p in enumerate(parts):
        # Check mpn and manufacturer are populated
        if not p.mpn:
            errors.append(f"Row {i}: Empty mpn")
        if not p.manufacturer:
            errors.append(f"Row {i}: Empty manufacturer")
        
        # Category-specific validation
        if p.category == "Resistor":
            if p.resistance_ohm is None:
                errors.append(f"Row {i} ({p.mpn}): Resistor missing resistance_ohm")
            if p.capacitance_f is not None:
                errors.append(f"Row {i} ({p.mpn}): Resistor should not have capacitance_f")
            if p.inductance_h is not None:
                errors.append(f"Row {i} ({p.mpn}): Resistor should not have inductance_h")
            if p.package not in ["0402", "0603", "0805", "1206"]:
                errors.append(f"Row {i} ({p.mpn}): Invalid package {p.package}")
        
        elif p.category == "Capacitor":
            if p.capacitance_f is None:
                errors.append(f"Row {i} ({p.mpn}): Capacitor missing capacitance_f")
            if p.resistance_ohm is not None:
                errors.append(f"Row {i} ({p.mpn}): Capacitor should not have resistance_ohm")
            if p.inductance_h is not None:
                errors.append(f"Row {i} ({p.mpn}): Capacitor should not have inductance_h")
            if p.package not in ["0402", "0603", "0805", "1206"]:
                errors.append(f"Row {i} ({p.mpn}): Invalid package {p.package}")
        
        elif p.category == "Inductor":
            if p.inductance_h is None:
                errors.append(f"Row {i} ({p.mpn}): Inductor missing inductance_h")
            if p.resistance_ohm is not None:
                errors.append(f"Row {i} ({p.mpn}): Inductor should not have resistance_ohm")
            if p.capacitance_f is not None:
                errors.append(f"Row {i} ({p.mpn}): Inductor should not have capacitance_f")
            if p.package not in ["0402", "0603", "0805", "1206"]:
                errors.append(f"Row {i} ({p.mpn}): Invalid package {p.package}")
    
    # Count by category
    resistors = [p for p in parts if p.category == "Resistor"]
    capacitors = [p for p in parts if p.category == "Capacitor"]
    inductors = [p for p in parts if p.category == "Inductor"]
    
    print(f"Resistors: {len(resistors)} (expected 180)")
    print(f"Capacitors: {len(capacitors)} (expected 250)")
    print(f"Inductors: {len(inductors)} (expected 70)")
    print(f"Total: {len(parts)} (expected 500)")
    
    if len(resistors) != 180:
        errors.append(f"Resistor count mismatch: {len(resistors)} != 180")
    if len(capacitors) != 250:
        errors.append(f"Capacitor count mismatch: {len(capacitors)} != 250")
    if len(inductors) != 70:
        errors.append(f"Inductor count mismatch: {len(inductors)} != 70")
    if len(parts) != 500:
        errors.append(f"Total count mismatch: {len(parts)} != 500")
    
    # Check for duplicate MPNs
    mpns = [p.mpn for p in parts]
    if len(mpns) != len(set(mpns)):
        dupes = {m for m in mpns if mpns.count(m) > 1}
        errors.append(f"Duplicate MPNs: {dupes}")
    
    if errors:
        print("\nVALIDATION ERRORS:")
        for e in errors:
            print(f"  - {e}")
        return False
    
    print("\nVALIDATION PASSED")
    return True

# ============================================================
# RESISTORS — 180 unique MPNs
# ============================================================

def gen_resistors() -> List[Part]:
    parts = []
    
    # Yageo RC0402 (0402, 1/16W = 0.0625W, 1%)
    values_402 = [
        ("RC0402FR-071RL", 1), ("RC0402FR-071R5L", 1.5), ("RC0402FR-072R2L", 2.2),
        ("RC0402FR-073R3L", 3.3), ("RC0402FR-074R7L", 4.7), ("RC0402FR-076R8L", 6.8),
        ("RC0402FR-0710RL", 10), ("RC0402FR-0715RL", 15), ("RC0402FR-0722RL", 22),
        ("RC0402FR-0733RL", 33), ("RC0402FR-0747RL", 47), ("RC0402FR-0768RL", 68),
        ("RC0402FR-07100L", 100), ("RC0402FR-07150L", 150), ("RC0402FR-07220L", 220),
        ("RC0402FR-07330L", 330), ("RC0402FR-07470L", 470), ("RC0402FR-07680L", 680),
        ("RC0402FR-071KL", 1000), ("RC0402FR-071K5L", 1500), ("RC0402FR-072K2L", 2200),
        ("RC0402FR-073K3L", 3300), ("RC0402FR-074K7L", 4700), ("RC0402FR-076K8L", 6800),
        ("RC0402FR-0710KL", 10000), ("RC0402FR-0715KL", 15000), ("RC0402FR-0722KL", 22000),
        ("RC0402FR-0733KL", 33000), ("RC0402FR-0747KL", 47000), ("RC0402FR-0768KL", 68000),
        ("RC0402FR-07100KL", 100000), ("RC0402FR-07150KL", 150000), ("RC0402FR-07220KL", 220000),
        ("RC0402FR-07330KL", 330000), ("RC0402FR-07470KL", 470000), ("RC0402FR-07680KL", 680000),
        ("RC0402FR-071ML", 1000000),
    ]
    for mpn, val in values_402:
        parts.append(Part(None, mpn, "Yageo", "Resistor", "Thin Film", "0402",
                         None, None, float(val), None, "±1%", "±100ppm/°C", 0.0625,
                         None, None, None,
                         f"{val}Ω 0402 1% 1/16W", f"resistor 0402 1% thin-film"))
    
    # Yageo RC0603 (0603, 1/10W = 0.1W, 1%)
    values_603 = [
        ("RC0603FR-071RL", 1), ("RC0603FR-071R5L", 1.5), ("RC0603FR-072R2L", 2.2),
        ("RC0603FR-073R3L", 3.3), ("RC0603FR-074R7L", 4.7), ("RC0603FR-076R8L", 6.8),
        ("RC0603FR-0710RL", 10), ("RC0603FR-0715RL", 15), ("RC0603FR-0722RL", 22),
        ("RC0603FR-0733RL", 33), ("RC0603FR-0747RL", 47), ("RC0603FR-0768RL", 68),
        ("RC0603FR-07100L", 100), ("RC0603FR-07150L", 150), ("RC0603FR-07220L", 220),
        ("RC0603FR-07330L", 330), ("RC0603FR-07470L", 470), ("RC0603FR-07680L", 680),
        ("RC0603FR-071KL", 1000), ("RC0603FR-071K5L", 1500), ("RC0603FR-072K2L", 2200),
        ("RC0603FR-073K3L", 3300), ("RC0603FR-074K7L", 4700), ("RC0603FR-076K8L", 6800),
        ("RC0603FR-0710KL", 10000), ("RC0603FR-0715KL", 15000), ("RC0603FR-0722KL", 22000),
        ("RC0603FR-0733KL", 33000), ("RC0603FR-0747KL", 47000), ("RC0603FR-0768KL", 68000),
        ("RC0603FR-07100KL", 100000), ("RC0603FR-07150KL", 150000), ("RC0603FR-07220KL", 220000),
        ("RC0603FR-07330KL", 330000), ("RC0603FR-07470KL", 470000), ("RC0603FR-07680KL", 680000),
        ("RC0603FR-071ML", 1000000),
    ]
    for mpn, val in values_603:
        parts.append(Part(None, mpn, "Yageo", "Resistor", "Thin Film", "0603",
                         None, None, float(val), None, "±1%", "±100ppm/°C", 0.1,
                         None, None, None,
                         f"{val}Ω 0603 1% 1/10W", f"resistor 0603 1% thin-film"))
    
    # Yageo RC0805 (0805, 1/8W = 0.125W, 1%)
    values_805 = [
        ("RC0805FR-071RL", 1), ("RC0805FR-071R5L", 1.5), ("RC0805FR-072R2L", 2.2),
        ("RC0805FR-073R3L", 3.3), ("RC0805FR-074R7L", 4.7), ("RC0805FR-076R8L", 6.8),
        ("RC0805FR-0710RL", 10), ("RC0805FR-0715RL", 15), ("RC0805FR-0722RL", 22),
        ("RC0805FR-0733RL", 33), ("RC0805FR-0747RL", 47), ("RC0805FR-0768RL", 68),
        ("RC0805FR-07100L", 100), ("RC0805FR-07150L", 150), ("RC0805FR-07220L", 220),
        ("RC0805FR-07330L", 330), ("RC0805FR-07470L", 470), ("RC0805FR-07680L", 680),
        ("RC0805FR-071KL", 1000), ("RC0805FR-071K5L", 1500), ("RC0805FR-072K2L", 2200),
        ("RC0805FR-073K3L", 3300), ("RC0805FR-074K7L", 4700), ("RC0805FR-076K8L", 6800),
        ("RC0805FR-0710KL", 10000), ("RC0805FR-0715KL", 15000), ("RC0805FR-0722KL", 22000),
        ("RC0805FR-0733KL", 33000), ("RC0805FR-0747KL", 47000), ("RC0805FR-0768KL", 68000),
        ("RC0805FR-07100KL", 100000), ("RC0805FR-07150KL", 150000), ("RC0805FR-07220KL", 220000),
        ("RC0805FR-07330KL", 330000), ("RC0805FR-07470KL", 470000), ("RC0805FR-07680KL", 680000),
        ("RC0805FR-071ML", 1000000),
    ]
    for mpn, val in values_805:
        parts.append(Part(None, mpn, "Yageo", "Resistor", "Thin Film", "0805",
                         None, None, float(val), None, "±1%", "±100ppm/°C", 0.125,
                         None, None, None,
                         f"{val}Ω 0805 1% 1/8W", f"resistor 0805 1% thin-film"))
    
    # Vishay CRCW0402 (0402, 1/16W, 1%)
    values_vishay = [
        ("CRCW04021R00FKED", 1), ("CRCW04021R50FKED", 1.5), ("CRCW04022R20FKED", 2.2),
        ("CRCW04023R30FKED", 3.3), ("CRCW04024R70FKED", 4.7), ("CRCW04026R80FKED", 6.8),
        ("CRCW040210R0FKED", 10), ("CRCW040215R0FKED", 15), ("CRCW040222R0FKED", 22),
        ("CRCW040233R0FKED", 33), ("CRCW040247R0FKED", 47), ("CRCW040268R0FKED", 68),
        ("CRCW0402100RFKED", 100), ("CRCW0402150RFKED", 150), ("CRCW0402220RFKED", 220),
        ("CRCW0402330RFKED", 330), ("CRCW0402470RFKED", 470), ("CRCW0402680RFKED", 680),
        ("CRCW04021K00FKED", 1000), ("CRCW04021K50FKED", 1500), ("CRCW04022K20FKED", 2200),
        ("CRCW04023K30FKED", 3300), ("CRCW04024K70FKED", 4700), ("CRCW04026K80FKED", 6800),
        ("CRCW040210K0FKED", 10000), ("CRCW040215K0FKED", 15000), ("CRCW040222K0FKED", 22000),
        ("CRCW040233K0FKED", 33000), ("CRCW040247K0FKED", 47000), ("CRCW040268K0FKED", 68000),
        ("CRCW0402100KFKED", 100000), ("CRCW0402150KFKED", 150000), ("CRCW0402220KFKED", 220000),
        ("CRCW0402330KFKED", 330000), ("CRCW0402470KFKED", 470000), ("CRCW0402680KFKED", 680000),
        ("CRCW04021M00FKED", 1000000),
    ]
    for mpn, val in values_vishay:
        parts.append(Part(None, mpn, "Vishay", "Resistor", "Thin Film", "0402",
                         None, None, float(val), None, "±1%", "±100ppm/°C", 0.0625,
                         None, None, None,
                         f"{val}Ω 0402 1% 1/16W", f"resistor 0402 1% thin-film"))
    
    # Panasonic ERJ-2GE (0402, 1/16W, 1%)
    values_pan = [
        ("ERJ-2GEJ1R0X", 1), ("ERJ-2GEJ1R5X", 1.5), ("ERJ-2GEJ2R2X", 2.2),
        ("ERJ-2GEJ3R3X", 3.3), ("ERJ-2GEJ4R7X", 4.7), ("ERJ-2GEJ6R8X", 6.8),
        ("ERJ-2GEJ100X", 10), ("ERJ-2GEJ150X", 15), ("ERJ-2GEJ220X", 22),
        ("ERJ-2GEJ330X", 33), ("ERJ-2GEJ470X", 47), ("ERJ-2GEJ680X", 68),
        ("ERJ-2GEJ101X", 100), ("ERJ-2GEJ151X", 150), ("ERJ-2GEJ221X", 220),
        ("ERJ-2GEJ331X", 330), ("ERJ-2GEJ471X", 470), ("ERJ-2GEJ681X", 680),
        ("ERJ-2GEJ102X", 1000), ("ERJ-2GEJ152X", 1500), ("ERJ-2GEJ222X", 2200),
        ("ERJ-2GEJ332X", 3300), ("ERJ-2GEJ472X", 4700), ("ERJ-2GEJ682X", 6800),
        ("ERJ-2GEJ103X", 10000), ("ERJ-2GEJ153X", 15000), ("ERJ-2GEJ223X", 22000),
        ("ERJ-2GEJ333X", 33000), ("ERJ-2GEJ473X", 47000), ("ERJ-2GEJ683X", 68000),
        ("ERJ-2GEJ104X", 100000), ("ERJ-2GEJ154X", 150000), ("ERJ-2GEJ224X", 220000),
        ("ERJ-2GEJ334X", 330000), ("ERJ-2GEJ474X", 470000), ("ERJ-2GEJ684X", 680000),
        ("ERJ-2GEJ105X", 1000000),
    ]
    for mpn, val in values_pan:
        parts.append(Part(None, mpn, "Panasonic", "Resistor", "Thick Film", "0402",
                         None, None, float(val), None, "±1%", "±100ppm/°C", 0.0625,
                         None, None, None,
                         f"{val}Ω 0402 1% 1/16W", f"resistor 0402 1% thick-film"))
    
    return parts[:180]

# ============================================================
# CAPACITORS — 250 unique MPNs
# ============================================================

def gen_capacitors() -> List[Part]:
    parts = []
    
    # Murata GRM155 (0402) C0G/NP0 50V
    cog_155 = [
        ("GRM1555C1H1R0CA01", 1e-12), ("GRM1555C1H1R5CA01", 1.5e-12), ("GRM1555C1H2R2CA01", 2.2e-12),
        ("GRM1555C1H3R3CA01", 3.3e-12), ("GRM1555C1H4R7CA01", 4.7e-12), ("GRM1555C1H6R8DA01", 6.8e-12),
        ("GRM1555C1H100JA01", 10e-12), ("GRM1555C1H150JA01", 15e-12), ("GRM1555C1H220JA01", 22e-12),
        ("GRM1555C1H330JA01", 33e-12), ("GRM1555C1H470JA01", 47e-12), ("GRM1555C1H680JA01", 68e-12),
        ("GRM1555C1H101JA01", 100e-12), ("GRM1555C1H151JA01", 150e-12), ("GRM1555C1H221JA01", 220e-12),
        ("GRM1555C1H331JA01", 330e-12), ("GRM1555C1H471JA01", 470e-12), ("GRM1555C1H681JA01", 680e-12),
        ("GRM1555C1H102JA01", 1e-9), ("GRM1555C1H152JA01", 1.5e-9), ("GRM1555C1H222JA01", 2.2e-9),
        ("GRM1555C1H332JA01", 3.3e-9), ("GRM1555C1H472JA01", 4.7e-9), ("GRM1555C1H682JA01", 6.8e-9),
        ("GRM1555C1H103JA01", 10e-9),
    ]
    for mpn, val in cog_155:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "C0G/NP0", "0402",
                         50.0, val, None, None, "±5%", "±30ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}F 0402 C0G 50V", f"capacitor 0402 C0G 50V"))
    
    # Murata GRM155 (0402) X7R 16V/25V/50V
    x7r_155 = [
        ("GRM155R71C103KA01", 10e-9, 16), ("GRM155R71C153KA01", 15e-9, 16), ("GRM155R71C223KA01", 22e-9, 16),
        ("GRM155R71C333KA01", 33e-9, 16), ("GRM155R71C473KA01", 47e-9, 16), ("GRM155R71C683KA01", 68e-9, 16),
        ("GRM155R71C104KA01", 100e-9, 16), ("GRM155R71C154KA01", 150e-9, 16), ("GRM155R71C224KA01", 220e-9, 16),
        ("GRM155R71E103KA01", 10e-9, 25), ("GRM155R71E153KA01", 15e-9, 25), ("GRM155R71E223KA01", 22e-9, 25),
        ("GRM155R71E333KA01", 33e-9, 25), ("GRM155R71E473KA01", 47e-9, 25), ("GRM155R71E683KA01", 68e-9, 25),
        ("GRM155R71E104KA01", 100e-9, 25), ("GRM155R71E154KA01", 150e-9, 25), ("GRM155R71E224KA01", 220e-9, 25),
        ("GRM155R71H103KA01", 10e-9, 50), ("GRM155R71H153KA01", 15e-9, 50), ("GRM155R71H223KA01", 22e-9, 50),
        ("GRM155R71H333KA01", 33e-9, 50), ("GRM155R71H473KA01", 47e-9, 50), ("GRM155R71H683KA01", 68e-9, 50),
        ("GRM155R71H104KA01", 100e-9, 50),
    ]
    for mpn, val, volt in x7r_155:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "X7R", "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 X7R {volt}V", f"capacitor 0402 X7R {volt}V"))
    
    # Murata GRM188 (0603) C0G/NP0 50V
    cog_188 = [
        ("GRM1885C1H1R0CA01", 1e-12), ("GRM1885C1H1R5CA01", 1.5e-12), ("GRM1885C1H2R2CA01", 2.2e-12),
        ("GRM1885C1H3R3CA01", 3.3e-12), ("GRM1885C1H4R7CA01", 4.7e-12), ("GRM1885C1H6R8DA01", 6.8e-12),
        ("GRM1885C1H100JA01", 10e-12), ("GRM1885C1H150JA01", 15e-12), ("GRM1885C1H220JA01", 22e-12),
        ("GRM1885C1H330JA01", 33e-12), ("GRM1885C1H470JA01", 47e-12), ("GRM1885C1H680JA01", 68e-12),
        ("GRM1885C1H101JA01", 100e-12), ("GRM1885C1H151JA01", 150e-12), ("GRM1885C1H221JA01", 220e-12),
        ("GRM1885C1H331JA01", 330e-12), ("GRM1885C1H471JA01", 470e-12), ("GRM1885C1H681JA01", 680e-12),
        ("GRM1885C1H102JA01", 1e-9), ("GRM1885C1H152JA01", 1.5e-9), ("GRM1885C1H222JA01", 2.2e-9),
        ("GRM1885C1H332JA01", 3.3e-9), ("GRM1885C1H472JA01", 4.7e-9), ("GRM1885C1H682JA01", 6.8e-9),
        ("GRM1885C1H103JA01", 10e-9), ("GRM1885C1H153JA01", 15e-9), ("GRM1885C1H223JA01", 22e-9),
        ("GRM1885C1H333JA01", 33e-9), ("GRM1885C1H473JA01", 47e-9), ("GRM1885C1H683JA01", 68e-9),
        ("GRM1885C1H104JA01", 100e-9),
    ]
    for mpn, val in cog_188:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "C0G/NP0", "0603",
                         50.0, val, None, None, "±5%", "±30ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}F 0603 C0G 50V", f"capacitor 0603 C0G 50V"))
    
    # Murata GRM188 (0603) X7R 16V/25V/50V
    x7r_188 = [
        ("GRM188R71C103KA01", 10e-9, 16), ("GRM188R71C153KA01", 15e-9, 16), ("GRM188R71C223KA01", 22e-9, 16),
        ("GRM188R71C333KA01", 33e-9, 16), ("GRM188R71C473KA01", 47e-9, 16), ("GRM188R71C683KA01", 68e-9, 16),
        ("GRM188R71C104KA01", 100e-9, 16), ("GRM188R71C154KA01", 150e-9, 16), ("GRM188R71C224KA01", 220e-9, 16),
        ("GRM188R71C334KA01", 330e-9, 16), ("GRM188R71C474KA01", 470e-9, 16), ("GRM188R71C105KA01", 1e-6, 16),
        ("GRM188R71E103KA01", 10e-9, 25), ("GRM188R71E153KA01", 15e-9, 25), ("GRM188R71E223KA01", 22e-9, 25),
        ("GRM188R71E333KA01", 33e-9, 25), ("GRM188R71E473KA01", 47e-9, 25), ("GRM188R71E683KA01", 68e-9, 25),
        ("GRM188R71E104KA01", 100e-9, 25), ("GRM188R71E154KA01", 150e-9, 25), ("GRM188R71E224KA01", 220e-9, 25),
        ("GRM188R71E334KA01", 330e-9, 25), ("GRM188R71E474KA01", 470e-9, 25), ("GRM188R71E105KA01", 1e-6, 25),
        ("GRM188R71H103KA01", 10e-9, 50), ("GRM188R71H153KA01", 15e-9, 50), ("GRM188R71H223KA01", 22e-9, 50),
        ("GRM188R71H333KA01", 33e-9, 50), ("GRM188R71H473KA01", 47e-9, 50), ("GRM188R71H683KA01", 68e-9, 50),
        ("GRM188R71H104KA01", 100e-9, 50),
    ]
    for mpn, val, volt in x7r_188:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "X7R", "0603",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0603 X7R {volt}V", f"capacitor 0603 X7R {volt}V"))
    
    # Murata GRM21 (0805) X7R 16V/25V/50V
    x7r_21 = [
        ("GRM21BR71C105KA01", 1e-6, 16), ("GRM21BR71C225KA01", 2.2e-6, 16), ("GRM21BR71C475KA01", 4.7e-6, 16),
        ("GRM21BR71E105KA01", 1e-6, 25), ("GRM21BR71E225KA01", 2.2e-6, 25), ("GRM21BR71E475KA01", 4.7e-6, 25),
        ("GRM21BR71H104KA01", 100e-9, 50), ("GRM21BR71H224KA01", 220e-9, 50), ("GRM21BR71H474KA01", 470e-9, 50),
        ("GRM21BR71H105KA01", 1e-6, 50),
    ]
    for mpn, val, volt in x7r_21:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "X7R", "0805",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0805 X7R {volt}V", f"capacitor 0805 X7R {volt}V"))
    
    # Murata GRM31 (1206) X7R 16V/25V/50V
    x7r_31 = [
        ("GRM31CR71C106KA01", 10e-6, 16), ("GRM31CR71C226KA01", 22e-6, 16),
        ("GRM31CR71E106KA01", 10e-6, 25), ("GRM31CR71E226KA01", 22e-6, 25),
        ("GRM31CR71H105KA01", 1e-6, 50), ("GRM31CR71H225KA01", 2.2e-6, 50), ("GRM31CR71H475KA01", 4.7e-6, 50),
    ]
    for mpn, val, volt in x7r_31:
        parts.append(Part(None, mpn, "Murata", "Capacitor", "X7R", "1206",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 1206 X7R {volt}V", f"capacitor 1206 X7R {volt}V"))
    
    # Taiyo Yuden UMK105 (0402) C0G 50V
    cog_ty_105 = [
        ("UMK105CG1R0CW-F", 1e-12), ("UMK105CG1R5CW-F", 1.5e-12), ("UMK105CG2R2CW-F", 2.2e-12),
        ("UMK105CG3R3CW-F", 3.3e-12), ("UMK105CG4R7CW-F", 4.7e-12), ("UMK105CG6R8DW-F", 6.8e-12),
        ("UMK105CG100JW-F", 10e-12), ("UMK105CG150JW-F", 15e-12), ("UMK105CG220JW-F", 22e-12),
        ("UMK105CG330JW-F", 33e-12), ("UMK105CG470JW-F", 47e-12), ("UMK105CG680JW-F", 68e-12),
        ("UMK105CG101JW-F", 100e-12), ("UMK105CG151JW-F", 150e-12), ("UMK105CG221JW-F", 220e-12),
        ("UMK105CG331JW-F", 330e-12), ("UMK105CG471JW-F", 470e-12), ("UMK105CG681JW-F", 680e-12),
        ("UMK105CG102JW-F", 1e-9), ("UMK105CG152JW-F", 1.5e-9), ("UMK105CG222JW-F", 2.2e-9),
        ("UMK105CG332JW-F", 3.3e-9), ("UMK105CG472JW-F", 4.7e-9), ("UMK105CG682JW-F", 6.8e-9),
        ("UMK105CG103JW-F", 10e-9),
    ]
    for mpn, val in cog_ty_105:
        parts.append(Part(None, mpn, "Taiyo Yuden", "Capacitor", "C0G/NP0", "0402",
                         50.0, val, None, None, "±5%", "±30ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}F 0402 C0G 50V", f"capacitor 0402 C0G 50V"))
    
    # Taiyo Yuden UMK105 (0402) X7R 16V/25V/50V
    x7r_ty_105 = [
        ("UMK105B7103KW-F", 10e-9, 16), ("UMK105B7153KW-F", 15e-9, 16), ("UMK105B7223KW-F", 22e-9, 16),
        ("UMK105B7333KW-F", 33e-9, 16), ("UMK105B7473KW-F", 47e-9, 16), ("UMK105B7683KW-F", 68e-9, 16),
        ("UMK105B7104KW-F", 100e-9, 16), ("UMK105B7154KW-F", 150e-9, 16), ("UMK105B7224KW-F", 220e-9, 16),
        ("UMK105B7103KV-F", 10e-9, 25), ("UMK105B7223KV-F", 22e-9, 25), ("UMK105B7473KV-F", 47e-9, 25),
        ("UMK105B7104KV-F", 100e-9, 25), ("UMK105B7103KJ-F", 10e-9, 50), ("UMK105B7223KJ-F", 22e-9, 50),
        ("UMK105B7473KJ-F", 47e-9, 50), ("UMK105B7104KJ-F", 100e-9, 50),
    ]
    for mpn, val, volt in x7r_ty_105:
        parts.append(Part(None, mpn, "Taiyo Yuden", "Capacitor", "X7R", "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 X7R {volt}V", f"capacitor 0402 X7R {volt}V"))
    
    # Taiyo Yuden EMK107 (0603) X7R 16V/25V/50V
    x7r_ty_107 = [
        ("EMK107B7103KW-F", 10e-9, 16), ("EMK107B7153KW-F", 15e-9, 16), ("EMK107B7223KW-F", 22e-9, 16),
        ("EMK107B7333KW-F", 33e-9, 16), ("EMK107B7473KW-F", 47e-9, 16), ("EMK107B7683KW-F", 68e-9, 16),
        ("EMK107B7104KW-F", 100e-9, 16), ("EMK107B7154KW-F", 150e-9, 16), ("EMK107B7224KW-F", 220e-9, 16),
        ("EMK107B7334KW-F", 330e-9, 16), ("EMK107B7474KW-F", 470e-9, 16), ("EMK107B7105KW-F", 1e-6, 16),
        ("EMK107B7103KV-F", 10e-9, 25), ("EMK107B7223KV-F", 22e-9, 25), ("EMK107B7473KV-F", 47e-9, 25),
        ("EMK107B7104KV-F", 100e-9, 25), ("EMK107B7224KV-F", 220e-9, 25), ("EMK107B7474KV-F", 470e-9, 25),
        ("EMK107B7103KJ-F", 10e-9, 50), ("EMK107B7223KJ-F", 22e-9, 50), ("EMK107B7473KJ-F", 47e-9, 50),
        ("EMK107B7104KJ-F", 100e-9, 50), ("EMK107B7224KJ-F", 220e-9, 50), ("EMK107B7474KJ-F", 470e-9, 50),
    ]
    for mpn, val, volt in x7r_ty_107:
        parts.append(Part(None, mpn, "Taiyo Yuden", "Capacitor", "X7R", "0603",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0603 X7R {volt}V", f"capacitor 0603 X7R {volt}V"))
    
    # Taiyo Yuden GMK212 (0805) X7R 16V/25V/50V
    x7r_ty_212 = [
        ("GMK212B7104KG-F", 100e-9, 16), ("GMK212B7224KG-F", 220e-9, 16), ("GMK212B7474KG-F", 470e-9, 16),
        ("GMK212B7105KG-F", 1e-6, 16), ("GMK212B7225KG-F", 2.2e-6, 16), ("GMK212B7475KG-F", 4.7e-6, 16),
        ("GMK212B7104KD-F", 100e-9, 25), ("GMK212B7224KD-F", 220e-9, 25), ("GMK212B7474KD-F", 470e-9, 25),
        ("GMK212B7105KD-F", 1e-6, 25), ("GMK212B7225KD-F", 2.2e-6, 25),
        ("GMK212B7104KB-F", 100e-9, 50), ("GMK212B7224KB-F", 220e-9, 50), ("GMK212B7474KB-F", 470e-9, 50),
    ]
    for mpn, val, volt in x7r_ty_212:
        parts.append(Part(None, mpn, "Taiyo Yuden", "Capacitor", "X7R", "0805",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0805 X7R {volt}V", f"capacitor 0805 X7R {volt}V"))
    
    # Samsung CL05 (0402) X7R/X5R
    sams_cl05 = [
        ("CL05A103KB5NNNC", 10e-9, 50), ("CL05A153KB5NNNC", 15e-9, 50), ("CL05A223KB5NNNC", 22e-9, 50),
        ("CL05A333KB5NNNC", 33e-9, 50), ("CL05A473KB5NNNC", 47e-9, 50), ("CL05A683KB5NNNC", 68e-9, 50),
        ("CL05A104KB5NNNC", 100e-9, 50), ("CL05A154KB5NNNC", 150e-9, 50), ("CL05A224KB5NNNC", 220e-9, 50),
        ("CL05A103KO5NNNC", 10e-9, 16), ("CL05A223KO5NNNC", 22e-9, 16), ("CL05A473KO5NNNC", 47e-9, 16),
        ("CL05A104KO5NNNC", 100e-9, 16), ("CL05A103KQ5NNNC", 10e-9, 6.3), ("CL05A223KQ5NNNC", 22e-9, 6.3),
        ("CL05A473KQ5NNNC", 47e-9, 6.3), ("CL05A104KQ5NNNC", 100e-9, 6.3), ("CL05A104MP5NNNC", 100e-9, 10),
        ("CL05A224MP5NNNC", 220e-9, 10), ("CL05A474MP5NNNC", 470e-9, 10), ("CL05A105MP5NNNC", 1e-6, 10),
        ("CL05A225MP5NNNC", 2.2e-6, 10), ("CL05A475MP5NNNC", 4.7e-6, 10), ("CL05A105MQ5NNNC", 1e-6, 6.3),
        ("CL05A225MQ5NNNC", 2.2e-6, 6.3),
    ]
    for mpn, val, volt in sams_cl05:
        dielectric = "X7R" if "B" in mpn or "A" in mpn and "KB" in mpn or "KO" in mpn or "KQ" in mpn else "X5R"
        if "MP" in mpn or "MQ" in mpn:
            dielectric = "X5R"
        parts.append(Part(None, mpn, "Samsung Electro-Mechanics", "Capacitor", dielectric, "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)" if dielectric == "X7R" else "±15% (-55 to +85°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 {dielectric} {volt}V", f"capacitor 0402 {dielectric} {volt}V"))
    
    # Samsung CL10 (0603) X7R/X5R
    sams_cl10 = [
        ("CL10B103KB8NNNC", 10e-9, 50), ("CL10B153KB8NNNC", 15e-9, 50), ("CL10B223KB8NNNC", 22e-9, 50),
        ("CL10B333KB8NNNC", 33e-9, 50), ("CL10B473KB8NNNC", 47e-9, 50), ("CL10B683KB8NNNC", 68e-9, 50),
        ("CL10B104KB8NNNC", 100e-9, 50), ("CL10B154KB8NNNC", 150e-9, 50), ("CL10B224KB8NNNC", 220e-9, 50),
        ("CL10B334KB8NNNC", 330e-9, 50), ("CL10B474KB8NNNC", 470e-9, 50), ("CL10B105KB8NNNC", 1e-6, 50),
        ("CL10B103KO8NNNC", 10e-9, 16), ("CL10B223KO8NNNC", 22e-9, 16), ("CL10B473KO8NNNC", 47e-9, 16),
        ("CL10B104KO8NNNC", 100e-9, 16), ("CL10B224KO8NNNC", 220e-9, 16), ("CL10B474KO8NNNC", 470e-9, 16),
        ("CL10B105KO8NNNC", 1e-6, 16), ("CL10B103KQ8NNNC", 10e-9, 6.3), ("CL10B223KQ8NNNC", 22e-9, 6.3),
        ("CL10B473KQ8NNNC", 47e-9, 6.3), ("CL10B104KQ8NNNC", 100e-9, 6.3), ("CL10B224KQ8NNNC", 220e-9, 6.3),
        ("CL10B474KQ8NNNC", 470e-9, 6.3), ("CL10A105MP8NNNC", 1e-6, 10), ("CL10A225MP8NNNC", 2.2e-6, 10),
        ("CL10A475MP8NNNC", 4.7e-6, 10), ("CL10A106MP8NNNC", 10e-6, 10), ("CL10A226MP8NNNC", 22e-6, 10),
    ]
    for mpn, val, volt in sams_cl10:
        dielectric = "X7R" if mpn.startswith("CL10B") else "X5R"
        parts.append(Part(None, mpn, "Samsung Electro-Mechanics", "Capacitor", dielectric, "0603",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)" if dielectric == "X7R" else "±15% (-55 to +85°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0603 {dielectric} {volt}V", f"capacitor 0603 {dielectric} {volt}V"))
    
    # TDK C1005 (0402) C0G/X7R
    tdk_c1005_cog = [
        ("C1005C0G1H1R0C050BA", 1e-12), ("C1005C0G1H1R5C050BA", 1.5e-12), ("C1005C0G1H2R2C050BA", 2.2e-12),
        ("C1005C0G1H3R3C050BA", 3.3e-12), ("C1005C0G1H4R7C050BA", 4.7e-12), ("C1005C0G1H6R8D050BA", 6.8e-12),
        ("C1005C0G1H100J050BA", 10e-12), ("C1005C0G1H150J050BA", 15e-12), ("C1005C0G1H220J050BA", 22e-12),
        ("C1005C0G1H330J050BA", 33e-12), ("C1005C0G1H470J050BA", 47e-12), ("C1005C0G1H680J050BA", 68e-12),
        ("C1005C0G1H101J050BA", 100e-12), ("C1005C0G1H151J050BA", 150e-12), ("C1005C0G1H221J050BA", 220e-12),
        ("C1005C0G1H331J050BA", 330e-12), ("C1005C0G1H471J050BA", 470e-12), ("C1005C0G1H681J050BA", 680e-12),
        ("C1005C0G1H102J050BA", 1e-9), ("C1005C0G1H152J050BA", 1.5e-9), ("C1005C0G1H222J050BA", 2.2e-9),
        ("C1005C0G1H332J050BA", 3.3e-9), ("C1005C0G1H472J050BA", 4.7e-9), ("C1005C0G1H682J050BA", 6.8e-9),
        ("C1005C0G1H103J050BA", 10e-9),
    ]
    for mpn, val in tdk_c1005_cog:
        parts.append(Part(None, mpn, "TDK", "Capacitor", "C0G/NP0", "0402",
                         50.0, val, None, None, "±5%", "±30ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}F 0402 C0G 50V", f"capacitor 0402 C0G 50V"))
    
    tdk_c1005_x7r = [
        ("C1005X7R1C103K050BA", 10e-9, 16), ("C1005X7R1C153K050BA", 15e-9, 16), ("C1005X7R1C223K050BA", 22e-9, 16),
        ("C1005X7R1C333K050BA", 33e-9, 16), ("C1005X7R1C473K050BA", 47e-9, 16), ("C1005X7R1C683K050BA", 68e-9, 16),
        ("C1005X7R1C104K050BA", 100e-9, 16), ("C1005X7R1C154K050BA", 150e-9, 16), ("C1005X7R1C224K050BA", 220e-9, 16),
        ("C1005X7R1E103K050BA", 10e-9, 25), ("C1005X7R1E223K050BA", 22e-9, 25), ("C1005X7R1E473K050BA", 47e-9, 25),
        ("C1005X7R1E104K050BA", 100e-9, 25), ("C1005X7R1H103K050BA", 10e-9, 50), ("C1005X7R1H223K050BA", 22e-9, 50),
        ("C1005X7R1H473K050BA", 47e-9, 50), ("C1005X7R1H104K050BA", 100e-9, 50),
    ]
    for mpn, val, volt in tdk_c1005_x7r:
        parts.append(Part(None, mpn, "TDK", "Capacitor", "X7R", "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 X7R {volt}V", f"capacitor 0402 X7R {volt}V"))
    
    # TDK C1608 (0603) X7R
    tdk_c1608 = [
        ("C1608X7R1C103K080AC", 10e-9, 16), ("C1608X7R1C153K080AC", 15e-9, 16), ("C1608X7R1C223K080AC", 22e-9, 16),
        ("C1608X7R1C333K080AC", 33e-9, 16), ("C1608X7R1C473K080AC", 47e-9, 16), ("C1608X7R1C683K080AC", 68e-9, 16),
        ("C1608X7R1C104K080AC", 100e-9, 16), ("C1608X7R1C154K080AC", 150e-9, 16), ("C1608X7R1C224K080AC", 220e-9, 16),
        ("C1608X7R1C334K080AC", 330e-9, 16), ("C1608X7R1C474K080AC", 470e-9, 16), ("C1608X7R1C105K080AC", 1e-6, 16),
        ("C1608X7R1E103K080AC", 10e-9, 25), ("C1608X7R1E223K080AC", 22e-9, 25), ("C1608X7R1E473K080AC", 47e-9, 25),
        ("C1608X7R1E104K080AC", 100e-9, 25), ("C1608X7R1E224K080AC", 220e-9, 25), ("C1608X7R1E474K080AC", 470e-9, 25),
        ("C1608X7R1E105K080AC", 1e-6, 25), ("C1608X7R1H103K080AC", 10e-9, 50), ("C1608X7R1H223K080AC", 22e-9, 50),
        ("C1608X7R1H473K080AC", 47e-9, 50), ("C1608X7R1H104K080AC", 100e-9, 50),
    ]
    for mpn, val, volt in tdk_c1608:
        parts.append(Part(None, mpn, "TDK", "Capacitor", "X7R", "0603",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0603 X7R {volt}V", f"capacitor 0603 X7R {volt}V"))
    
    # Vishay VJ0402 (0402) X7R
    vj0402 = [
        ("VJ0402Y103KXAMT", 10e-9, 25), ("VJ0402Y153KXAMT", 15e-9, 25), ("VJ0402Y223KXAMT", 22e-9, 25),
        ("VJ0402Y333KXAMT", 33e-9, 25), ("VJ0402Y473KXAMT", 47e-9, 25), ("VJ0402Y683KXAMT", 68e-9, 25),
        ("VJ0402Y104KXAMT", 100e-9, 25), ("VJ0402Y154KXAMT", 150e-9, 25), ("VJ0402Y224KXAMT", 220e-9, 25),
        ("VJ0402Y103KXBMT", 10e-9, 50), ("VJ0402Y223KXBMT", 22e-9, 50), ("VJ0402Y473KXBMT", 47e-9, 50),
        ("VJ0402Y104KXBMT", 100e-9, 50), ("VJ0402Y103KXQMT", 10e-9, 16), ("VJ0402Y223KXQMT", 22e-9, 16),
        ("VJ0402Y473KXQMT", 47e-9, 16), ("VJ0402Y104KXQMT", 100e-9, 16),
    ]
    for mpn, val, volt in vj0402:
        parts.append(Part(None, mpn, "Vishay", "Capacitor", "X7R", "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 X7R {volt}V", f"capacitor 0402 X7R {volt}V"))
    
    # Panasonic ECJ-0EB (0402) X7R
    ecj_eb = [
        ("ECJ-0EB1C103K", 10e-9, 16), ("ECJ-0EB1C153K", 15e-9, 16), ("ECJ-0EB1C223K", 22e-9, 16),
        ("ECJ-0EB1C333K", 33e-9, 16), ("ECJ-0EB1C473K", 47e-9, 16), ("ECJ-0EB1C683K", 68e-9, 16),
        ("ECJ-0EB1C104K", 100e-9, 16), ("ECJ-0EB1C154K", 150e-9, 16), ("ECJ-0EB1C224K", 220e-9, 16),
        ("ECJ-0EB1E103K", 10e-9, 25), ("ECJ-0EB1E223K", 22e-9, 25), ("ECJ-0EB1E473K", 47e-9, 25),
        ("ECJ-0EB1E104K", 100e-9, 25), ("ECJ-0EB1E224K", 220e-9, 25), ("ECJ-0EB1E474K", 470e-9, 25),
        ("ECJ-0EB1H103K", 10e-9, 50), ("ECJ-0EB1H223K", 22e-9, 50), ("ECJ-0EB1H473K", 47e-9, 50),
        ("ECJ-0EB1H104K", 100e-9, 50),
    ]
    for mpn, val, volt in ecj_eb:
        parts.append(Part(None, mpn, "Panasonic", "Capacitor", "X7R", "0402",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0402 X7R {volt}V", f"capacitor 0402 X7R {volt}V"))
    
    # Panasonic ECJ-1VB (0603) X7R
    ecj_vb = [
        ("ECJ-1VB1C103K", 10e-9, 16), ("ECJ-1VB1C153K", 15e-9, 16), ("ECJ-1VB1C223K", 22e-9, 16),
        ("ECJ-1VB1C333K", 33e-9, 16), ("ECJ-1VB1C473K", 47e-9, 16), ("ECJ-1VB1C683K", 68e-9, 16),
        ("ECJ-1VB1C104K", 100e-9, 16), ("ECJ-1VB1C154K", 150e-9, 16), ("ECJ-1VB1C224K", 220e-9, 16),
        ("ECJ-1VB1C334K", 330e-9, 16), ("ECJ-1VB1C474K", 470e-9, 16), ("ECJ-1VB1C105K", 1e-6, 16),
        ("ECJ-1VB1E103K", 10e-9, 25), ("ECJ-1VB1E223K", 22e-9, 25), ("ECJ-1VB1E473K", 47e-9, 25),
        ("ECJ-1VB1E104K", 100e-9, 25), ("ECJ-1VB1E224K", 220e-9, 25), ("ECJ-1VB1E474K", 470e-9, 25),
        ("ECJ-1VB1E105K", 1e-6, 25), ("ECJ-1VB1H103K", 10e-9, 50), ("ECJ-1VB1H223K", 22e-9, 50),
        ("ECJ-1VB1H473K", 47e-9, 50), ("ECJ-1VB1H104K", 100e-9, 50),
    ]
    for mpn, val, volt in ecj_vb:
        parts.append(Part(None, mpn, "Panasonic", "Capacitor", "X7R", "0603",
                         float(volt), val, None, None, "±10%", "±15% (-55 to +125°C)", None,
                         None, None, None,
                         f"{val:.0e}F 0603 X7R {volt}V", f"capacitor 0603 X7R {volt}V"))
    
    return parts[:250]

# ============================================================
# INDUCTORS — 70 unique MPNs
# ============================================================

def gen_inductors() -> List[Part]:
    parts = []
    
    # Murata LQG15H (0402) High-frequency
    lqg15h = [
        ("LQG15HN1N0S02", 1e-9), ("LQG15HN1N2S02", 1.2e-9), ("LQG15HN1N5S02", 1.5e-9),
        ("LQG15HN1N8S02", 1.8e-9), ("LQG15HN2N2S02", 2.2e-9), ("LQG15HN2N7S02", 2.7e-9),
        ("LQG15HN3N3S02", 3.3e-9), ("LQG15HN3N9S02", 3.9e-9), ("LQG15HN4N7S02", 4.7e-9),
        ("LQG15HN5N6S02", 5.6e-9), ("LQG15HN6N8S02", 6.8e-9), ("LQG15HN8N2S02", 8.2e-9),
        ("LQG15HN10NJ02", 10e-9), ("LQG15HN12NJ02", 12e-9), ("LQG15HN15NJ02", 15e-9),
        ("LQG15HN18NJ02", 18e-9), ("LQG15HN22NJ02", 22e-9), ("LQG15HN27NJ02", 27e-9),
        ("LQG15HN33NJ02", 33e-9), ("LQG15HN39NJ02", 39e-9), ("LQG15HN47NJ02", 47e-9),
        ("LQG15HN56NJ02", 56e-9), ("LQG15HN68NJ02", 68e-9), ("LQG15HN82NJ02", 82e-9),
        ("LQG15HN100J02", 100e-9), ("LQG15HN120J02", 120e-9), ("LQG15HN150J02", 150e-9),
        ("LQG15HN180J02", 180e-9), ("LQG15HN220J02", 220e-9), ("LQG15HN270J02", 270e-9),
        ("LQG15HN330J02", 330e-9), ("LQG15HN390J02", 390e-9), ("LQG15HN470J02", 470e-9),
    ]
    for mpn, val in lqg15h:
        parts.append(Part(None, mpn, "Murata", "Inductor", "High-Frequency", "0402",
                         None, None, None, val, "±5%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0402 HF ±5%", f"inductor 0402 high-frequency"))
    
    # Murata LQM18P (0603) Power inductors
    lqm18p = [
        ("LQM18PN1R0M00", 1e-6), ("LQM18PN1R5M00", 1.5e-6), ("LQM18PN2R2M00", 2.2e-6),
        ("LQM18PN3R3M00", 3.3e-6), ("LQM18PN4R7M00", 4.7e-6), ("LQM18PN6R8M00", 6.8e-6),
        ("LQM18PN100M00", 10e-6), ("LQM18PN150M00", 15e-6), ("LQM18PN220M00", 22e-6),
        ("LQM18PN330M00", 33e-6), ("LQM18PN470M00", 47e-6), ("LQM18PN680M00", 68e-6),
        ("LQM18PN101M00", 100e-6), ("LQM18PN151M00", 150e-6), ("LQM18PN221M00", 220e-6),
    ]
    for mpn, val in lqm18p:
        parts.append(Part(None, mpn, "Murata", "Inductor", "Power", "0603",
                         None, None, None, val, "±20%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0603 Power ±20%", f"inductor 0603 power"))
    
    # TDK MLZ1608 (0603) Power inductors
    mlz1608 = [
        ("MLZ1608M1R0WT000", 1e-6), ("MLZ1608M1R5WT000", 1.5e-6), ("MLZ1608M2R2WT000", 2.2e-6),
        ("MLZ1608M3R3WT000", 3.3e-6), ("MLZ1608M4R7WT000", 4.7e-6), ("MLZ1608M6R8WT000", 6.8e-6),
        ("MLZ1608M100WT000", 10e-6), ("MLZ1608M150WT000", 15e-6), ("MLZ1608M220WT000", 22e-6),
        ("MLZ1608M330WT000", 33e-6), ("MLZ1608M470WT000", 47e-6), ("MLZ1608M680WT000", 68e-6),
        ("MLZ1608M101WT000", 100e-6),
    ]
    for mpn, val in mlz1608:
        parts.append(Part(None, mpn, "TDK", "Inductor", "Power", "0603",
                         None, None, None, val, "±20%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0603 Power ±20%", f"inductor 0603 power"))
    
    # Bourns SRN2009 (0805) Power inductors
    srn2009 = [
        ("SRN2009-1R0M", 1e-6), ("SRN2009-1R5M", 1.5e-6), ("SRN2009-2R2M", 2.2e-6),
        ("SRN2009-3R3M", 3.3e-6), ("SRN2009-4R7M", 4.7e-6), ("SRN2009-6R8M", 6.8e-6),
        ("SRN2009-100M", 10e-6), ("SRN2009-150M", 15e-6), ("SRN2009-220M", 22e-6),
        ("SRN2009-330M", 33e-6), ("SRN2009-470M", 47e-6), ("SRN2009-680M", 68e-6),
        ("SRN2009-101M", 100e-6),
    ]
    for mpn, val in srn2009:
        parts.append(Part(None, mpn, "Bourns", "Inductor", "Power", "0805",
                         None, None, None, val, "±20%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0805 Power ±20%", f"inductor 0805 power"))
    
    # Würth Elektronik WE-MCA (0805) Power inductors
    we_mca = [
        ("7447920010", 1e-6), ("7447920015", 1.5e-6), ("7447920022", 2.2e-6),
        ("7447920033", 3.3e-6), ("7447920047", 4.7e-6), ("7447920068", 6.8e-6),
        ("7447920100", 10e-6), ("7447920150", 15e-6), ("7447920220", 22e-6),
        ("7447920330", 33e-6), ("7447920470", 47e-6), ("7447920680", 68e-6),
        ("7447921000", 100e-6),
    ]
    for mpn, val in we_mca:
        parts.append(Part(None, mpn, "Würth Elektronik", "Inductor", "Power", "0805",
                         None, None, None, val, "±20%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0805 Power ±20%", f"inductor 0805 power"))
    
    # Vishay IHLP1616 (0603) Power inductors
    ihlp1616 = [
        ("IHLP1616ABER1R0M01", 1e-6), ("IHLP1616ABER1R5M01", 1.5e-6), ("IHLP1616ABER2R2M01", 2.2e-6),
        ("IHLP1616ABER3R3M01", 3.3e-6), ("IHLP1616ABER4R7M01", 4.7e-6), ("IHLP1616ABER6R8M01", 6.8e-6),
        ("IHLP1616ABER100M01", 10e-6), ("IHLP1616ABER150M01", 15e-6), ("IHLP1616ABER220M01", 22e-6),
        ("IHLP1616ABER330M01", 33e-6), ("IHLP1616ABER470M01", 47e-6), ("IHLP1616ABER680M01", 68e-6),
        ("IHLP1616ABER101M01", 100e-6),
    ]
    for mpn, val in ihlp1616:
        parts.append(Part(None, mpn, "Vishay", "Inductor", "Power", "0603",
                         None, None, None, val, "±20%", "±25ppm/°C", None,
                         None, None, None,
                         f"{val:.0e}H 0603 Power ±20%", f"inductor 0603 power"))
    
    return parts[:70]

# ============================================================
# MAIN
# ============================================================

def main():
    resistors = gen_resistors()
    capacitors = gen_capacitors()
    inductors = gen_inductors()
    
    all_parts = resistors + capacitors + inductors
    
    print(f"Generated: {len(resistors)} resistors, {len(capacitors)} capacitors, {len(inductors)} inductors")
    print(f"Total: {len(all_parts)} parts")
    
    # Validate
    if not validate_parts(all_parts):
        print("VALIDATION FAILED - NOT WRITING CSV")
        return 1
    
    # Write CSV
    output_path = "/home/dev/electronics-agent-kit/data/imports/day1_passives.csv"
    with open(output_path, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(HEADERS)
        for p in all_parts:
            writer.writerow(p.to_row())
    
    print(f"\nCSV written to {output_path}")
    return 0

if __name__ == "__main__":
    exit(main())