import type { EntityId, PhysicalQuantity } from "../contract/v1";

/** Symbol form of a unit for compact display. */
const UNIT_SYMBOL: Record<string, string> = {
  Metre: "m",
  Millimetre: "mm",
  Micrometre: "µm",
  Centimetre: "cm",
  Watt: "W",
  Milliwatt: "mW",
  Ampere: "A",
  Milliampere: "mA",
  Volt: "V",
  Ohm: "Ω",
  Kilohm: "kΩ",
  Hertz: "Hz",
  Farad: "F",
  Celsius: "°C",
};

export function unitSymbol(unit: string): string {
  return UNIT_SYMBOL[unit] ?? unit;
}

/** Trim float noise (e.g. 6.500000000000001 → 6.5) for display. */
function tidy(n: number): string {
  const r = Math.round(n * 1000) / 1000;
  return String(r);
}

export function quantity(pq: PhysicalQuantity | null | undefined): string {
  if (!pq) return "—";
  return `${tidy(pq.magnitude)} ${unitSymbol(pq.unit)}`;
}

/** The kernel's short 8-hex id form — matches EntityId::short() for human-facing traces. */
export function shortId(id: EntityId | undefined): string {
  if (!id) return "—";
  try {
    const hex = BigInt(id).toString(16).padStart(8, "0");
    return hex.slice(-8);
  } catch {
    return String(id).slice(0, 8);
  }
}

export function titleCase(s: string): string {
  return s.replace(/([a-z])([A-Z])/g, "$1 $2");
}
