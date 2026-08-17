# TUSK — the AI-native electronics engineering IDE

The **TUSK workspace**: a projection of the kernel's committed-event stream. React + TypeScript +
Vite. This is the concrete implementation of the six-region layout and the *Graphite + Signal* CMF
spec described in [`../../docs/ui/USER-MANUAL.md`](../../docs/ui/USER-MANUAL.md).

> The kernel owns the truth; this UI only projects it. Every panel is the fold of an event log —
> close a panel and nothing is lost.

The EDA surfaces are pure projections of owned facts: the schematic is drawn from committed pins and
nets, the PCB from placements and tracks, the layers panel from the board stack-up, the library from
sourced parts, and the findings dock from raised violations. A surface can never claim a connection
the kernel didn't commit.

## Run it (no Tauri / webkit needed)

```bash
cd app/ui
npm install
npm run dev
```

In the browser it plays a **captured `eak-events.jsonl`** — real output from `eak run` over the hero
cassette (*"USB-C powered I²C temperature sensor, < 1 W"*), 151 committed events, replayed through
the same fold the packaged app drives live. That is why the status bar reads **CASSETTE**: a recorded
run replayed deterministically. No webkit2gtk is required until the app is finally packaged.

A second captured run is available for the review workflow: append `?fixture=review` to load the
BLOCKED import-review cassette (a real `eak` import + verify of a demo board — 2 open
`bom-coverage` findings, gate blocked).

```bash
npm test           # fold-parity + EDA selector tests over both cassettes
npm run build      # typecheck + production build
node scripts/shot.mjs <previewPort> <outDir>   # headless-Chrome visual QA (DOM assertions + shots)
```

## How it maps to the kernel

| Concern | File | Mirrors (kernel) |
|---|---|---|
| Event / entity contract | `src/contract/v1.ts` | `eak-ports::Event`, `eak-domain` entities |
| Offline replay source | `src/events/FixturePlayer.ts` | a captured `EventLog` |
| Live desktop source | `src/events/TauriBridge.ts` | `EventSink` → `eak://event` (see `../src-tauri/src/main.rs`) |
| The fold | `src/store/fold.ts` | `eak-runtime::EngineeringState::apply` |
| EDA projections (pads, ratlines, net map, schematic graph, cross-probe) | `src/store/selectors.ts` | `eak-runtime::EngineeringState` queries |
| Store + UI state | `src/store/useWorkspaceStore.ts` | — |
| Design tokens (CMF + ECAD palette) | `src/styles/tokens.css` | USER-MANUAL §11 |

## Refresh the fixtures from the kernel

```bash
cd ../../eak
# hero (RELEASED generated design)
cargo run -p eak-cli -- run \
  --intent "USB-C powered I2C temperature sensor, < 1 W" \
  --reasoning fixture --cassette crates/eak-cli/fixtures/hero_cassette.json \
  --log ../app/ui/public/fixtures/hero.jsonl --deterministic --seed 1
# review (BLOCKED import review — reaches BOM gate, never fabricates a BOM)
cargo run -p eak-cli --example emit_review_cassette \
  -- crates/eak-cli/fixtures/review_board.kicad_pcb \
  > ../app/ui/public/fixtures/review.jsonl
```

The streams are stored verbatim (real kernel bytes); the loader quotes the u128 `EntityId`s at parse
time so JS never rounds them — see the note in `src/contract/v1.ts`.

## Known contract gap (surfaced, not hidden)

`EntityId` is a `u128` serialized as a bare JSON number (up to 39 digits), which exceeds JS
`Number` precision. The loader works around it by string-quoting long integer runs before parse.
The **proper fix lives in the kernel**: serialize `EntityId` as a string across the sink/IPC
boundary so both this fixture path and the live `TauriBridge` are precise by construction. Tracked
against `contract v1`.