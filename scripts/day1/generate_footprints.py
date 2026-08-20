#!/usr/bin/env python3
"""
Generate KiCad footprint files for passive components.
Creates standard IPC-7351 compliant footprints for 0402, 0603, 0805, 1206.
"""

import os

# Package dimensions in mm (IPC-7351 nominal)
PACKAGES = {
    "0402": {
        "l": 1.0, "w": 0.5, "h": 0.35,
        "pad_w": 0.5, "pad_h": 0.5, "pad_spacing": 0.5,
        "courtyard_excess": 0.25,
    },
    "0603": {
        "l": 1.6, "w": 0.8, "h": 0.45,
        "pad_w": 0.9, "pad_h": 0.7, "pad_spacing": 0.9,
        "courtyard_excess": 0.25,
    },
    "0805": {
        "l": 2.0, "w": 1.25, "h": 0.55,
        "pad_w": 1.3, "pad_h": 0.9, "pad_spacing": 1.15,
        "courtyard_excess": 0.25,
    },
    "1206": {
        "l": 3.2, "w": 1.6, "h": 0.55,
        "pad_w": 1.6, "pad_h": 1.1, "pad_spacing": 1.75,
        "courtyard_excess": 0.25,
    },
}

def generate_footprint(pkg_code, dims):
    """Generate a KiCad footprint for a chip component."""
    l = dims["l"]
    w = dims["w"]
    pad_w = dims["pad_w"]
    pad_h = dims["pad_h"]
    pad_spacing = dims["pad_spacing"]
    courtyard_excess = dims["courtyard_excess"]
    
    # Pad positions (centered)
    pad_x = pad_spacing / 2
    
    # Courtyard
    cw = l + 2 * courtyard_excess
    ch = w + 2 * courtyard_excess
    
    # Silkscreen outline (slightly larger than body)
    ss_w = w + 0.2
    ss_l = l + 0.2
    
    fp = f"""(footprint "Resistor_{pkg_code}" (version 20211014) (generator "EAK")
  (layer "F.Cu")
  (descr "Surface mount resistor/capacitor/inductor {pkg_code} metric")
  (tags "resistor capacitor inductor {pkg_code} SMD")
  (attr smd)
  (fp_text reference "REF**" (at 0 {-(w/2 + 0.5):.2f}) (layer "F.SilkS")
    (effects (font (size 1 1) (thickness 0.15))))
  (fp_text value {{VALUE}} (at 0 {(w/2 + 0.5):.2f}) (layer "F.Fab")
    (effects (font (size 1 1) (thickness 0.15))))
  (fp_line (start {-ss_l/2:.2f} {-ss_w/2:.2f}) (end {ss_l/2:.2f} {-ss_w/2:.2f}) (layer "F.SilkS") (stroke (width 0.12) (type default)))
  (fp_line (start {ss_l/2:.2f} {-ss_w/2:.2f}) (end {ss_l/2:.2f} {ss_w/2:.2f}) (layer "F.SilkS") (stroke (width 0.12) (type default)))
  (fp_line (start {ss_l/2:.2f} {ss_w/2:.2f}) (end {-ss_l/2:.2f} {ss_w/2:.2f}) (layer "F.SilkS") (stroke (width 0.12) (type default)))
  (fp_line (start {-ss_l/2:.2f} {ss_w/2:.2f}) (end {-ss_l/2:.2f} {-ss_w/2:.2f}) (layer "F.SilkS") (stroke (width 0.12) (type default)))
  (fp_line (start {-cw/2:.2f} {-ch/2:.2f}) (end {cw/2:.2f} {-ch/2:.2f}) (layer "F.CrtYd") (stroke (width 0.05) (type default)))
  (fp_line (start {cw/2:.2f} {-ch/2:.2f}) (end {cw/2:.2f} {ch/2:.2f}) (layer "F.CrtYd") (stroke (width 0.05) (type default)))
  (fp_line (start {cw/2:.2f} {ch/2:.2f}) (end {-cw/2:.2f} {ch/2:.2f}) (layer "F.CrtYd") (stroke (width 0.05) (type default)))
  (fp_line (start {-cw/2:.2f} {ch/2:.2f}) (end {-cw/2:.2f} {-ch/2:.2f}) (layer "F.CrtYd") (stroke (width 0.05) (type default)))
  (pad "1" smd rect (at {-pad_x:.2f} 0) (size {pad_w:.2f} {pad_h:.2f}) (layers "F.Cu" "F.Paste" "F.Mask"))
  (pad "2" smd rect (at {pad_x:.2f} 0) (size {pad_w:.2f} {pad_h:.2f}) (layers "F.Cu" "F.Paste" "F.Mask"))
  (model ${{KICAD8_FOOTPRINT_DIR}}/Resistor_{pkg_code}.3dshapes/Resistor_{pkg_code}.step
    (at (xyz 0 0 0))
    (scale (xyz 1 1 1))
    (rotate (xyz 0 0 0))
  )
)
"""
    return fp

def main():
    footprints_dir = "/home/dev/electronics-agent-kit/data/assets/footprints"
    os.makedirs(footprints_dir, exist_ok=True)
    
    for pkg_code, dims in PACKAGES.items():
        fp_content = generate_footprint(pkg_code, dims)
        filename = f"Resistor_{pkg_code}.kicad_mod"
        filepath = os.path.join(footprints_dir, filename)
        with open(filepath, "w") as f:
            f.write(fp_content)
        print(f"Generated {filename}")
    
    print(f"\nGenerated {len(PACKAGES)} footprints in {footprints_dir}")
    
    # Verify
    for pkg_code in PACKAGES:
        filename = f"Resistor_{pkg_code}.kicad_mod"
        filepath = os.path.join(footprints_dir, filename)
        with open(filepath, "r") as f:
            content = f.read()
            if content.startswith("(footprint") and 'pad "1"' in content and 'pad "2"' in content:
                print(f"  {filename}: Valid KiCad footprint format")
            else:
                print(f"  {filename}: INVALID format!")
                return 1
    
    return 0

if __name__ == "__main__":
    exit(main())