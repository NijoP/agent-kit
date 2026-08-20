#!/usr/bin/env python3
"""
Generate KiCad symbol files for passive components.
Creates generic symbols that can be reused across MPNs.
"""

import os

# Generic resistor symbol (2 pins)
RESISTOR_SYMBOL = """(kicad_symbol_lib (version 20211014) (generator "EAK")
  (symbol "Resistor" (in_bom yes) (on_board yes)
    (property "Reference" "R" (at 0 -2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Value" "Resistor" (at 0 2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Footprint" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "MPN" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Manufacturer" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (pin passive line (at -2.54 0 180) (length 2.54) (name "1" (effects (font (size 1.27 1.27)))) (number "1"))
    (pin passive line (at 2.54 0 0) (length 2.54) (name "2" (effects (font (size 1.27 1.27)))) (number "2"))
    (rectangle (start -1.27 -1.27) (end 1.27 1.27) (stroke (width 0.254) (type default)) (fill (type none)))
  )
)
"""

# Generic capacitor symbol (2 pins)
CAPACITOR_SYMBOL = """(kicad_symbol_lib (version 20211014) (generator "EAK")
  (symbol "Capacitor" (in_bom yes) (on_board yes)
    (property "Reference" "C" (at 0 -2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Value" "Capacitor" (at 0 2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Footprint" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "MPN" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Manufacturer" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (pin passive line (at -2.54 0 180) (length 2.54) (name "1" (effects (font (size 1.27 1.27)))) (number "1"))
    (pin passive line (at 2.54 0 0) (length 2.54) (name "2" (effects (font (size 1.27 1.27)))) (number "2"))
    (polyline (pts (xy -1.27 -1.27) (xy -1.27 1.27)) (stroke (width 0.254) (type default)) (fill (type none)))
    (polyline (pts (xy 1.27 -1.27) (xy 1.27 1.27)) (stroke (width 0.254) (type default)) (fill (type none)))
  )
)
"""

# Generic inductor symbol (2 pins)
INDUCTOR_SYMBOL = """(kicad_symbol_lib (version 20211014) (generator "EAK")
  (symbol "Inductor" (in_bom yes) (on_board yes)
    (property "Reference" "L" (at 0 -2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Value" "Inductor" (at 0 2.54 90) (effects (font (size 1.27 1.27)) (justify left)))
    (property "Footprint" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Datasheet" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "MPN" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (property "Manufacturer" "" (at 0 0 90) (effects (font (size 1.27 1.27)) (hide yes)))
    (pin passive line (at -2.54 0 180) (length 2.54) (name "1" (effects (font (size 1.27 1.27)))) (number "1"))
    (pin passive line (at 2.54 0 0) (length 2.54) (name "2" (effects (font (size 1.27 1.27)))) (number "2"))
    (arc (start -1.27 -1.27) (mid -1.27 0) (end -1.27 1.27) (stroke (width 0.254) (type default)) (fill (type none)))
    (arc (start -0.42 -1.27) (mid -0.42 0) (end -0.42 1.27) (stroke (width 0.254) (type default)) (fill (type none)))
    (arc (start 0.42 -1.27) (mid 0.42 0) (end 0.42 1.27) (stroke (width 0.254) (type default)) (fill (type none)))
    (arc (start 1.27 -1.27) (mid 1.27 0) (end 1.27 1.27) (stroke (width 0.254) (type default)) (fill (type none)))
  )
)
"""

def main():
    symbols_dir = "/home/dev/electronics-agent-kit/data/assets/symbols"
    os.makedirs(symbols_dir, exist_ok=True)
    
    # Write generic symbols
    with open(os.path.join(symbols_dir, "Resistor.kicad_sym"), "w") as f:
        f.write(RESISTOR_SYMBOL)
    
    with open(os.path.join(symbols_dir, "Capacitor.kicad_sym"), "w") as f:
        f.write(CAPACITOR_SYMBOL)
    
    with open(os.path.join(symbols_dir, "Inductor.kicad_sym"), "w") as f:
        f.write(INDUCTOR_SYMBOL)
    
    print(f"Generated 3 generic symbols in {symbols_dir}")
    
    # Verify they're valid KiCad format
    for name in ["Resistor.kicad_sym", "Capacitor.kicad_sym", "Inductor.kicad_sym"]:
        path = os.path.join(symbols_dir, name)
        with open(path, "r") as f:
            content = f.read()
            if content.startswith("(kicad_symbol_lib") and "symbol" in content:
                print(f"  {name}: Valid KiCad symbol format")
            else:
                print(f"  {name}: INVALID format!")
                return 1
    
    return 0

if __name__ == "__main__":
    exit(main())