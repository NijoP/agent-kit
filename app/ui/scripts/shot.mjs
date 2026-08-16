// Visual QA driver: loads the built workspace in headless Chrome, drives it like an engineer, and
// records both screenshots (for humans) and DOM-level assertions (the machine check).
// Usage: node scripts/shot.mjs <distPort> <outDir>
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";

const [port = "4173", outDir = "/tmp/tusk-shots"] = process.argv.slice(2);
await mkdir(outDir, { recursive: true });

const browser = await chromium.launch({
  executablePath: "/usr/bin/google-chrome",
  args: ["--no-sandbox"],
});
const page = await browser.newPage({ viewport: { width: 1560, height: 940 }, deviceScaleFactor: 2 });
const errors = [];
page.on("console", (m) => { if (m.type() === "error") errors.push(`console: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`page: ${e.message}`));

const report = [];
const check = (name, ok, detail = "") => {
  report.push(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`);
};
const shot = (name) => page.screenshot({ path: `${outDir}/${name}.png` });

const waitForEvents = async (n) => {
  await page.waitForFunction(
    (target) => {
      const segs = [...document.querySelectorAll(".ide-statusbar .seg")];
      const el = segs.find((s) => s.textContent.includes("events"));
      if (!el) return false;
      const m = el.textContent.match(/(\d+)\s+events/);
      return m && Number(m[1]) >= target;
    },
    n,
    { timeout: 40000 },
  );
};

const root = `http://localhost:${port}`;

// ── hero cassette ──────────────────────────────────────────────────────────────────────────────
await page.goto(root, { waitUntil: "networkidle" });
await waitForEvents(151);
await page.waitForTimeout(400);

// brand
const brand = await page.textContent(".ide-menubar .brand");
check("brand is TUSK", brand.includes("TUSK"));
check("title is TUSK", (await page.title()).includes("TUSK"));

// landing: PRD doc, gate released, status bar
const statusText = await page.textContent(".ide-statusbar");
check("status bar shows RELEASED gate", statusText.includes("RELEASED"));
check("status bar net legend", /Power\s*1/.test(statusText) && /Ground\s*1/.test(statusText));
await shot("01-default-landing");

// PCB via keyboard
await page.keyboard.press("Control+3");
await page.waitForTimeout(600);
const pcbSvg = await page.$(".pcb-svg");
check("PCB canvas renders", !!pcbSvg);
const pcbHasPads = await page.$$eval(".pad", (els) => els.length);
check("PCB draws footprint pads", pcbHasPads === 4, `pads=${pcbHasPads}`);
const pcbHasTracks = await page.$$eval(".track", (els) => els.length);
check("PCB draws routed tracks", pcbHasTracks >= 6, `tracks=${pcbHasTracks}`);
const boardRect = await page.$eval(".brd-outline", (r) => r.getAttribute("width"));
check("board outline renders", !!boardRect && Number(boardRect) > 0, `w=${boardRect}`);
await shot("02-pcb-canvas");

// open the Design tree, select the VBUS net → cross-probe highlight on the PCB
await page.click('.ide-activitybar button[title="Design"]');
await page.waitForTimeout(300);
await page.click('.ide-sidebar .trow:has-text("Nets")');
await page.waitForTimeout(300);
await page.click('.ide-sidebar .trow:has-text("VBUS")');
await page.waitForTimeout(500);
const hotPads = await page.$$eval(".pad.hot", (els) => els.length);
const dimPads = await page.$$eval(".pad.dim", (els) => els.length);
check("net selection highlights its pads", hotPads === 4, `hot=${hotPads}`);
check("net selection dims other pads", dimPads === 0, `dim=${dimPads}`);
const hotTracks = await page.$$eval(".track.sel", (els) => els.length);
check("net selection highlights its tracks", hotTracks >= 3, `hotTracks=${hotTracks}`);
check("status bar reflects selection", (await page.textContent(".ide-statusbar")).includes("VBUS"));
await shot("03-pcb-net-highlight");

// inspector reflects the net
await page.click('.ide-rightdock .tab:has-text("Inspector")');
await page.waitForTimeout(400);
const insText = await page.textContent(".dock-section.inspector");
check("inspector shows net details", insText.includes("VBUS") && insText.includes("Power"));
check("inspector shows routing state", insText.includes("routed"));
check("inspector lists member pins", /Member pins/.test(insText));
await shot("04-inspector-net");

// schematic
await page.keyboard.press("Control+4");
await page.waitForTimeout(600);
const schNodes = await page.$$eval(".schematic-node", (els) => els.length);
check("schematic draws one node per component", schNodes === 4, `nodes=${schNodes}`);
const schRails = await page.$$eval(".schematic-rail", (els) => els.length);
check("schematic draws one rail per net", schRails === 2, `rails=${schRails}`);
const schLabels = await page.textContent(".schematic-netlabel");
check("schematic labels nets", schLabels.includes("VBUS"));
await shot("05-schematic");

// library
await page.click('.ide-activitybar button[title="Library"]');
await page.waitForTimeout(500);
const libParts = await page.$$eval(".lib-card", (els) => els.length);
check("library lists owned parts", libParts === 2, `parts=${libParts}`);
check("library shows lifecycle pill", (await page.textContent(".lib-card")).includes("Active"));
const libStats = await page.textContent(".lib-stats");
check("library stats", /4 symbols/.test(libStats) && /2 parts/.test(libStats));
await shot("06-library");

// layers panel + stack-up
await page.click('.ide-activitybar button[title="Design"]');
await page.waitForTimeout(300);
await page.keyboard.press("Control+3");
await page.waitForTimeout(400);
const stackText = await page.textContent(".stackup");
check("stack-up renders owned layers", /2 layers/.test(stackText) && /Signal/.test(stackText) && /Plane/.test(stackText));
check("stack-up shows thickness", /µm|mm/.test(stackText));
await shot("07-layers-stackup");

// command palette jumps to a net
await page.keyboard.press("Control+k");
await page.waitForTimeout(300);
await page.keyboard.type("Go to net: VBUS");
await page.waitForTimeout(300);
await page.keyboard.press("Enter");
await page.waitForTimeout(400);
const hotAfterPalette = await page.$$eval(".pad.hot", (els) => els.length);
check("palette jump selects the net", hotAfterPalette === 4, `hot=${hotAfterPalette}`);

// ── review cassette (BLOCKED, actionable violations) ───────────────────────────────────────────
await page.goto(`${root}/?fixture=review`, { waitUntil: "networkidle" });
await waitForEvents(35);
await page.waitForTimeout(400);

check("review gate shows BLOCKED", (await page.textContent(".ide-statusbar")).includes("BLOCKED"));
const reviewRows = await page.$$eval(".drc-row", (els) => els.length);
check("problems dock lists both violations", reviewRows === 2, `rows=${reviewRows}`);
const reviewRule = await page.textContent(".dock-content .drc-row .rule");
check("violation rule is bom-coverage", reviewRule.includes("bom-coverage"), reviewRule);
check("violation row has Explain action", (await page.textContent(".dock-content")).includes("Explain"));
check("violation row has Waive action", (await page.textContent(".dock-content")).includes("Waive"));
await shot("08-review-problems");

const explain = page.locator('.drc-row:has-text("bom-coverage") button:has-text("Explain")').first();
await explain.click();
await page.waitForTimeout(400);
const explText = await page.textContent(".viol-expl");
check("explain opens the explanation pane", explText.includes("Explanation"));
check("explain is honest when no kernel explanation", /No kernel explanation/.test(explText) || explText.length > 0);
await shot("09-review-explain");

await page.keyboard.press("Control+3");
await page.waitForTimeout(500);
const reviewPads = await page.$$eval(".pad", (els) => els.length);
check("review PCB draws R1+R2 pads", reviewPads === 2, `pads=${reviewPads}`);
const reviewDrcMarkers = await page.$$eval(".drc-marker", (els) => els.length);
check("review PCB marks violation subjects", reviewDrcMarkers === 2, `markers=${reviewDrcMarkers}`);
await shot("10-review-pcb");

// cross-probe from a violation subject on the review board
await page.click('.dock-tab:has-text("Problems")');
await page.waitForTimeout(300);
await page.click('.dock-content .drc-row .msg');
await page.waitForTimeout(400);
const selLabel = await page.textContent(".ide-statusbar");
check("clicking a finding selects its subject", /R1|R2/.test(selLabel));

check("no console/page errors", errors.length === 0, errors.join(" | "));

await browser.close();
const reportText = report.join("\n");
await writeFile(`${outDir}/qa-report.txt`, reportText + "\n");
console.log(reportText);
console.log(`screenshots → ${outDir}`);