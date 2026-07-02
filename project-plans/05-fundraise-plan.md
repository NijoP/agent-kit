# Fundraise Plan — Pre-Seed

> Anchored to `00-overview.md` (the source of truth). Goal restated from there: a
> **fundable MVP + demo in ~3 months** that raises a **pre-seed**, funding ~12 months
> to build the real product. Team = solo technical founder + heavy AI-assisted dev.
> This doc is the go-to-market for the *raise*, not the product. Written 2026-07-02.
>
> Tone: honest and practical. A solo, pre-revenue, pre-user founder raising pre-seed
> is a **hard** raise. This plan is built to maximize odds, not to pretend they're high.

---

## 0. The one-sentence framing everything hangs on

**"AI-native hardware design needs a correctness runtime you can trust — I built the
runtime, here's the demo, fund the year that turns it into the product."**

Every artifact below is a projection of that sentence. If a slide, email, or demo beat
doesn't serve it, cut it.

---

## 1. The fundable artifact set

Five artifacts. Each has a **quality bar** — the line below which it actively hurts you.
Ship all five to the bar; do not ship a sixth thing instead.

### 1.1 The demo video (2–3 min) — *the single most important asset*
- **What:** screen recording of the hero demo (`00-overview.md` §4) inside the local IDE:
  one English sentence → requirements → architecture → BOM → schematic → assisted route →
  kernel review with explanations, traceability graph filling in live. End on the
  KiCad-import → AI-review beat (the bulletproof segment) so viewers see it generalizes.
- **Quality bar:** no dead air, no "let me refresh," no visible jank. Real product UI, not
  slides pretending to be a product. Voiceover is tight and technical, not salesy. First
  15 seconds must show the "whoa" (sentence in → checked board out) before any preamble —
  investors watch the first 20 seconds and decide whether to finish. Under 3:00. Captioned.
- **Why it's #1:** a solo founder's video *is* the first meeting. It gets forwarded between
  partners when you're not in the room. It must survive being watched at 2x with no audio.

### 1.2 The live demo (screen-shared, driven by you)
- **What:** the same flow, run live, plus 2–3 "off the rails" moments you can safely trigger
  (change one requirement, re-run; import a different real board; show a DRC catch). Have
  the curated example memorized cold.
- **Quality bar:** runs offline from a clean boot in <60s to first output. You can recover
  from any hiccup without breaking eye contact. Investors *will* ask "can you type something
  else?" — have 2 pre-vetted alternate prompts that also work flawlessly, and be honest that
  the surface is curated (that's the MVP scope, not a lie).

### 1.3 The deck (~10–12 slides) — see §2
- **Quality bar:** readable in 3 minutes with no narration (it gets forwarded). One idea per
  slide, big type, few words. The correctness-kernel/moat slide and the demo slide carry the
  deck; everything else is support. No fake TAM math, no fake logos.

### 1.4 Design-partner / waitlist signal — see §4
- **What:** a landing page + waitlist, plus a handful of real hardware engineers who said
  "I want this / I'll try it." Depth over vanity count.
- **Quality bar (be specific):** **150–400 waitlist emails** from *real hardware engineers*
  (not a Product Hunt spray of randoms), **plus 3–5 named design partners** who took a call
  and will let you quote them. Ten qualified, quotable believers beat 2,000 anonymous emails.

### 1.5 Data-room basics
- **What (minimum viable):** the deck (PDF), the demo video (unlisted link), a one-page
  technical brief on the kernel (deterministic, event-sourced, 185 tests, 15-phase pipeline,
  verification engine — pulled from `00-overview.md` §7), a one-pager on you (background +
  why you can build this), a clean cap table (you own 100%), and a short FAQ/objection doc.
- **Quality bar:** everything in one shared folder (Notion/Drive/DocSend), link-gated, no
  broken links, no half-finished docs. Legal can be light at pre-seed; **do not** over-lawyer
  before you have a lead. A SAFE template (YC standard) is enough.

**Explicitly NOT required at pre-seed:** revenue, a full product, a team, a patent, audited
financials, a 5-year model. Adding these is procrastination dressed as diligence.

---

## 2. Deck outline (11 slides)

Slide-by-slide. Each line is the *claim the slide must land*, not decoration.

1. **Title / one-liner.** "Cursor for hardware — an AI harness you can trust to design
   boards, because a deterministic kernel verifies everything it does." Name, contact, one
   clean product shot. (Set the frame in 5 seconds.)
2. **Problem.** Hardware design is slow, expert-gated, and error-prone; mistakes cost
   board respins (weeks + real money). Existing AI copilots *suggest* but can't be trusted
   to *drive* because nothing guarantees correctness — the human is still the safety net.
3. **Insight / why now.** LLMs are finally good enough to *drive* engineering work — but
   only if a deterministic system verifies them. "AI that suggests" is a toy; "AI you can
   trust to drive" needs a correctness runtime underneath. That runtime is the missing piece,
   and it's now buildable. (Why now = frontier models + the kernel both exist in 2026.)
4. **The wedge.** We don't rebuild KiCad. We narrow to one loop — English intent → generated,
   verified, fully-traceable board — reusing the canvas, owning the harness + kernel. Land
   with the intent→generate→review→explain loop; expand outward from there.
5. **Demo.** Embedded 2–3 min video / live walkthrough. This slide is mostly the demo itself.
   The kernel catching an error and the AI explaining it (traced to the original sentence) is
   the money shot. **Spend the most meeting-time here.**
6. **Moat — the correctness kernel.** The defensible core: deterministic, event-sourced kernel
   that *owns* engineering state; every AI action verified, traceable, replayable. Contrast:
   copilots bolt AI onto an editor (probabilistic assist); we built AI behind a strict kernel
   boundary (verified-by-construction). This is architecturally hard to add after the fact —
   that's the moat, not "we have AI too." (Proof points: 185 tests, 15-phase pipeline.)
7. **Market.** Bottom-up, not fantasy TAM: KiCad has millions of users; Altium/Cadence/etc. is
   a multi-billion-dollar EDA market; hardware startups + pro engineers are the beachhead.
   Frame as "EDA is a large, old, incumbent-heavy market ripe for an AI-native re-platforming,"
   and size the *wedge* (design partners → pro seats) honestly.
8. **Competition — esp. vs Flux.** 2x2 or table: editor-first + AI copilot (Flux) vs
   runtime-first + AI driver (us). Flux = AI that suggests, human catches mistakes. Us = AI
   you can trust to drive, verified + traceable + replayable. KiCad = powerful, no AI, no
   correctness runtime. Incumbents = slow, cloud-locked, not AI-native. Own the "trust" axis.
9. **Business model direction.** Not a pricing table — a *direction*: seat-based SaaS for pro
   engineers / teams (Cursor-style), local-first app, later team/collab + verified-output /
   compliance tiers as the enterprise wedge. Signal you know where revenue comes from without
   over-committing pre-product.
10. **Team / why me.** Solo *technical* founder who **already built the hard part** — a real,
    tested, deterministic engineering runtime (this is the differentiator; most "Cursor for X"
    founders have a wrapper, you have a kernel). Show depth: 185 tests, physics-grounded
    verification, the engineering-science layer. Address solo head-on: "solo today, hiring the
    first 2 engineers with this round." Turn the weakness into "I ship — here's proof."
11. **The ask + use of funds + 12-month plan.** The number (§5), the milestones it buys
    (design partners → private beta → first paying pilots), and the 12-month plan tied to
    `03-roadmap.md`. End on the milestone that de-risks the *next* round (seed).

*(Optional 12th "appendix" slide: architecture diagram / kernel deep-dive for technical
investors who ask — keep out of the main flow.)*

---

## 3. The narrative (the 3–4 sentences that tie it together)

> Hardware design is slow and expert-gated, and the new wave of AI copilots can only
> *suggest* — a human still has to catch every mistake, because nothing underneath guarantees
> the design is correct. I believe AI should be able to *drive* hardware design, and the only
> way to trust it is a deterministic correctness runtime that owns the engineering state and
> verifies every move — so I built that runtime first: an event-sourced Rust kernel with a
> 15-phase pipeline and a real verification engine, already tested and working. Here's the
> demo — one English sentence becomes a checked, explained, fully-traceable board inside a
> local IDE. I'm raising a pre-seed to spend the next year turning this runtime into the
> product: the AI-native EDA tool that engineers actually design on.

Memorize this. It's the answer to "so what do you do?" and the spine of every email.

---

## 4. Traction / signal plan (manufacturing credibility in 3 months)

You are pre-product for most of the 3 months. You cannot show usage. So you manufacture
**belief signal**: proof that real hardware engineers want this, and that you can build it.

### 4.1 Waitlist — target and mechanics
- **Target: 150–400 qualified emails** by pitch time. Qualified = identifiably a hardware/EE
  person (not a generic "cool" upvote). Quality of the list > size.
- **Landing page:** one page — the one-liner, a 60–90s cut of the demo video, 3 bullets on
  the moat, an email capture, one line on "design partner? book a call." Ship it in week 2–3
  so it accrues signal the whole time. (Reuse a template; this is not where build effort goes.)
- **Instrument it:** track source of every signup so you can tell investors "X% came from
  hardware communities, not press." Sourced, engaged signups are the credible ones.

### 4.2 Where the hardware engineers are (go here, in person/online)
- **Reddit:** r/PrintedCircuitBoard, r/AskElectronics, r/electronics, r/ECE, r/KiCad, r/embedded,
  r/hardware, r/FPGA. Post the demo as a "look what I built," not an ad. Value first.
- **X/Twitter:** hardware / EE / KiCad / embedded builders; reply-guy your way into the
  conversation for weeks before you launch; DM people who post board bring-ups.
- **Discord/Slack:** KiCad community, various embedded/hardware Discords, Hackaday-adjacent
  servers, maker/EE servers. Lurk, help, then share.
- **Forums / sites:** Hackaday, EEVblog forum, Hackster.io, r/hardware crossovers, Adafruit /
  SparkFun community edges, university EE / robotics / cubesat / Formula-SAE teams.
- **Events/hackathons:** hardware hackathons, maker faires, local EE/embedded meetups; any
  demo you can do in person converts far above online.

### 4.3 Design-partner outreach (the highest-value signal)
- **Goal: 3–5 named design partners.** A design partner = a real engineer/team who took a call,
  saw the demo, and agreed to try it / give feedback (and let you quote them to investors).
- **Who:** hardware startups (they feel the pain and move fast), indie hardware/embedded
  consultants, hardware-heavy university teams, small robotics/IoT/drone shops.
- **Method:** personal 1:1 outreach (not a blast). "I built X, you clearly care about Y, 20
  minutes?" Show the demo, ask about their workflow, ask if they'd design-partner. Log every
  call; the *quotes* ("this would save us a respin") are worth more than the count.

### 4.4 The public launch (convert attention → investor-relevant signal)
- **Show HN** ("Show HN: Cursor for hardware, with a deterministic correctness kernel") — HN
  respects a real technical artifact from a solo builder; the kernel is your credibility.
- **Cross-post** the demo to the communities in §4.2, timed together for a wave.
- **A written technical post** ("Why AI hardware design needs a correctness runtime") that
  explains the moat — this doubles as an investor artifact and a recruiting/credibility asset.
- **Convert:** every upvote/comment/signup is raw attention. Turn it into investor signal by
  (a) capturing emails, (b) booking design-partner calls off the interest, (c) screenshotting
  the *best* qualitative reactions from real engineers for the deck's "signal" slide. Investors
  don't fund traffic; they fund evidence that the *right* people want it. Curate accordingly.

---

## 5. Investor strategy

### 5.1 Who to target (in priority order for *this* profile)
1. **Accelerators — your best single shot as a solo, pre-network founder.**
   - **YC** — funds solo technical founders with real artifacts routinely; strong on
     "founder built the hard thing." Apply regardless of timing; it's a forcing function.
   - **HAX** (SOSV) — hardware-focused, deep-tech friendly; understands hardware timelines.
   - Other hardware/deep-tech / dev-tool programs (e.g. hardware-focused pre-seed programs,
     university/regional deep-tech accelerators). Accelerators solve *both* money and network.
2. **Technical / hardware-friendly angels.** People who ran EDA/hardware companies, ex-Altium/
   Cadence/Autodesk, hardware founders who exited, dev-tool angels (they get "Cursor for X").
   Angels write faster, need less consensus, and are reachable by a solo founder.
3. **Pre-seed / deep-tech micro-funds.** Small funds that lead $300k–$1M pre-seeds, explicitly
   deep-tech / dev-tools / "picks and shovels" / AI-infra theses. They can *lead* and set terms.
4. **Dev-tool & AI-infra seed funds** (as stretch / for later): they understand the "IDE +
   runtime" shape even if hardware is adjacent to their core thesis.

### 5.2 How many to approach
- **Build a pipeline of ~40–60 targets** (angels + micro-funds + 2–3 accelerators). Expect
  roughly: ~40–60 contacted → ~15–25 first meetings → ~5–10 second meetings → 1–3 checks.
  Pre-seed is a numbers game with a leaky funnel; a thin list is the #1 avoidable mistake.
- **Sequence it:** pitch 5–8 *lower-priority* names first as live practice, tighten the story,
  *then* hit your top targets once the pitch is sharp. Never burn your best intro on draft 1.

### 5.3 Warm-intro tactics for a solo founder with no network
- **The demo is your intro.** A great 2-min video is a warm intro that scales — it gets
  forwarded. Lead cold outreach *with the artifact*, not with a request for a call.
- **Manufacture warmth:** (a) do the public launch first, then reference it ("400 hardware
  engineers on the waitlist in 2 weeks"); (b) get design partners to intro you to *their*
  investors; (c) angels intro to other angels — always end a good meeting with "who else
  should I talk to?"; (d) use founder communities (YC alumni via the accelerator, indie
  hackers, on-deck-style groups) for intros.
- **Cold email that works:** 5 sentences — one-liner, the artifact link, the one proof point
  (kernel/tests), the one traction point (waitlist/partners), a specific small ask ("15 min?").
  Personalize the first line to *why them*. No 6-paragraph essays.

### 5.4 Round shape
- **Raise: $500k–$1.5M** on a **SAFE** (post-money cap; YC standard doc). For a solo pre-product
  founder, **~$750k** is a realistic, credible target; going out at $3M would signal naïveté.
- **What it funds (12 months):** you full-time + **1–2 engineers** (frontend/UI + one systems/
  kernel or AI eng), LLM/API + parts-API + tooling costs, a design-partner program, and runway
  to the *next* de-risking milestone.
- **Milestones it buys (what makes the seed raisable):** curated demo → real private beta in
  design partners' hands → the loop working on *more than one* curated example → first signs of
  usage/retention → ideally first paid pilots. The pre-seed's job is to convert "impressive
  demo" into "engineers are actually using this," which is exactly what a seed investor buys.
- **Keep dilution sane:** ~10–20% total on the SAFE(s). Don't give away control at pre-seed.

---

## 6. Timeline (raise overlaps the 3-month build)

Fundraising is a *relationship* process that starts early and *closes* late. Don't wait until
the demo is done to start; don't hard-pitch before it exists.

- **Weeks 1–4 (build heads-down + plant seeds):** build the demo. In parallel: ship the landing
  page + waitlist, start showing up in the communities (§4.2), start a lightweight investor list.
  **No pitching yet** — you have nothing to show. Begin *casual* "building this, would love to
  keep you posted" touches with 5–10 friendly angels (not asks — warmups).
- **Weeks 5–8 (demo takes shape + build the funnel):** get the hero demo to "rough but real."
  Do soft design-partner outreach; book calls off community interest. Expand the investor list
  to 40–60. Draft the deck + record a first demo video. Send "here's an early look" updates to
  your warmup angels (this is how you earn a real meeting later).
- **Weeks 9–10 (public launch + signal spike):** demo is flawless on the curated example.
  Launch (Show HN + community wave). Harvest waitlist + partner quotes. Finalize the polished
  demo video and deck. This launch is *timed* to give you a fresh traction story for the pitch.
- **Weeks 11–13 (formal raise):** run the pitch **in batches** (practice names first, then top
  targets) inside a compressed 2–4 week window to create urgency/competition. Data room ready.
  Push for a lead; once you have a lead + terms, fill the round with angels fast. Close on SAFEs.
- **Reality check:** a pre-seed often takes **2–4 months of active pitching** to close. Budget
  for the raise to extend past month 3. Keep ~6+ months of personal runway so you never pitch
  from desperation (investors smell it).

---

## 7. Objection handling (top 5 — crisp answers)

1. **"Solo founder — that's a risk."**
   *"Correct, and I'm de-risking it with this round: the first two hires are budgeted. But
   solo let me build the hard part — a real, tested deterministic kernel — with no committee.
   Most 'AI for X' founders show a wrapper; I'm showing a runtime. I ship."* (Then point at
   the 185 tests / working demo. Proof beats promises on this one.)

2. **"How is this different from Flux?"**
   *"Flux is editor-first with an AI copilot: the human designs, AI suggests, the human catches
   mistakes. We're runtime-first: a deterministic kernel owns the engineering state and verifies
   every AI action — so the AI can *drive*, not just suggest, and every result is traceable and
   replayable. Flux = AI that suggests; us = AI you can trust. That's an architectural choice
   that's very hard to bolt onto an editor after the fact."*

3. **"Can't incumbents (or Flux, or Cadence) just copy this?"**
   *"They can add an AI copilot — everyone will. What's hard to copy is the correctness runtime
   *underneath*: event-sourced state, verification-by-construction, full traceability + replay.
   That's a ground-up architectural commitment, not a feature. Editor-first products would have
   to rebuild their core to get it — and I already built mine."*

4. **"Hardware is hard, slow, and a brutal market."**
   *"Which is exactly why AI-native tooling is valuable here — respins cost weeks and real money,
   so trustworthy automation has huge ROI. And I'm not manufacturing hardware or fighting a
   hardware sales cycle; I'm selling *software to the engineers who design hardware* — a
   Cursor-style seat business on top of a large, old, incumbent-heavy EDA market ripe for
   re-platforming."*

5. **"Where are the users? This is a demo, not a product."**
   *"True — that's what the pre-seed is for. Today I have a working demo, [N] hardware engineers
   on the waitlist, and [3–5] design partners who want it. The round funds turning the demo into
   a private beta in their hands and getting to real usage. I'm raising to buy exactly the
   evidence a seed investor will want."* (Honesty here builds trust; over-claiming loses it.)

---

## 8. Realistic odds & alternatives

### 8.1 Honest odds
A **solo, pre-revenue, pre-user** founder raising a **priced/SAFE pre-seed** in hardware-adjacent
dev tools is a **hard raise** — realistically **low, maybe ~10–20%** of landing a "clean" pre-seed
purely on demo + signal in one 3-month cycle, *even with a great demo*. The two biggest odds-movers
you control: **(a)** a genuinely jaw-dropping, flawless demo, and **(b)** real design-partner
belief (quotes + a few pilots). Nail both and the odds rise materially; miss either and it stalls.
Plan as if the priced pre-seed might not land on the first pass — and have backups pre-wired so a
"no" is a fork, not a dead end.

### 8.2 Backup paths (pre-wire these — don't scramble later)
- **Accelerator-first (recommended hedge).** Treat YC / HAX / a deep-tech program as a *primary*
  path, not a fallback. It solves money **and** the network gap **and** gives you a credibility
  stamp that makes the pre-seed easier. Apply on the next cycle regardless of the direct raise.
- **Angels-only rolling SAFE.** If no fund will *lead*, skip the "priced round" framing and raise
  **$150k–$400k from a handful of angels** on SAFEs, one check at a time. Lower bar, no lead
  needed, extends runway to a stronger seed. Often the realistic actual outcome for solo founders.
- **Grants / non-dilutive.** Deep-tech / hardware / research grants, cloud + LLM credits
  (startup credit programs from the major clouds and model providers), pitch-competition prizes.
  Slow and bureaucratic, but non-dilutive and a credibility signal. Stack these under any path.
- **Revenue-first / bootstrap-to-raise.** If capital won't come, get 2–3 design partners to
  **pay** (even a small pilot fee) for the AI-review-on-import capability (the bulletproof
  segment already works). Revenue — even tiny — flips the story from "trust me" to "people pay
  for this," and makes the *next* raise dramatically easier. This is the strongest fallback.
- **Extend and re-cut.** If nothing lands in month 3, the kernel + demo don't expire. Keep
  shipping to a *second* working example and more usage, then re-approach in 3–6 months with a
  materially stronger story. A "not yet" is often "not enough traction yet," which is fixable.

### 8.3 The one thing that matters most
Investors at pre-seed buy **conviction in the founder + evidence the wedge is real.** For you,
that reduces to: **a demo that makes engineers say "I need this," and a few who prove it by
signing up, showing up, or paying.** Build those two things and the rest of this plan is
execution. Everything else — deck polish, investor lists, SAFE mechanics — is secondary to the
demo and the design partners. Spend your scarce hours accordingly.
