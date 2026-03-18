# TennoHelios — Phase 1 MVP Task Breakdown

Tasks are ordered by dependency. Nothing in a later milestone requires something
from a later milestone. Each task is scoped to one focused session.

**Platform markers:**
- macOS ✅ — can be fully built and tested on macOS
- macOS ⚠️ — can be written on macOS but only partially tested (needs real data or workarounds)
- Linux 🐧 — requires Bazzite / Steam Deck to verify end-to-end

---

## Milestone 1 — EE.log Watcher ✅ DONE

- [x] Task 1.1 — Scaffold Tauri + React + TypeScript + Tailwind | macOS ✅
  - Set up pnpm project, vite.config.ts, tsconfig.json, postcss, tailwind config
  - Done: `pnpm tauri dev` compiles and opens a window
  - Note: Icon files required at compile time by `tauri::generate_context!()`; create
    placeholder RGBA PNGs if real icons not yet designed

- [x] Task 1.2 — `log_watcher.rs`: file watching + trigger detection | macOS ✅
  - Tail EE.log from EOF; scan new lines for `ProjectionRewardChoice.swf`
  - Emit `RewardScreenEvent` through `mpsc` channel; handle log truncation
  - Done: `cargo test --lib -- log_watcher` → 7/7 pass
  - Ref: wfinfo-ng trigger patterns; note FSEvents unreliable in macOS test contexts
    so integration test uses a polling loop

- [x] Task 1.3 — Wire log_watcher into Tauri (`lib.rs`) | macOS ✅
  - Spawn watcher thread on app start; forward events via `app.emit()`
  - `App.tsx` listens for `"reward-screen-detected"` and shows it in UI
  - Done: changing EE.log content while app is running updates the UI status text

---

## Milestone 2 — Screenshot Capture

- [ ] Task 2.1 — `screenshot.rs`: X11 capture via `xcap` | macOS ✅
  - Add `xcap` crate; implement `capture_screen() -> Result<DynamicImage>`
  - On X11 / XWayland: capture primary monitor; return `image::DynamicImage`
  - Done: calling the function saves a PNG to `/tmp/tennohelios_test.png`
    that contains the actual screen contents
  - Ref: wfinfo-ng uses `xcap 0.0.4`; check for newer version
  - Note: test on macOS to verify basic capture works; Wayland path comes in 2.2

- [ ] Task 2.2 — `screenshot.rs`: Wayland capture path | Linux 🐧
  - Detect if running under Wayland (`WAYLAND_DISPLAY` env var)
  - Use `grim` subprocess (already present on Bazzite / SteamOS) or
    `libwaylandclient` + `wlr-screencopy` protocol for native capture
  - Fallback: if grim not found, fall back to X11/XWayland path automatically
  - Done: screenshot captures the Warframe window correctly inside a Gamescope session
  - Gotcha: Gamescope runs Wayland inside a nested compositor; the outer desktop
    screenshot APIs will capture the Gamescope window, not the game — verify
    that the captured image actually shows the reward screen

- [ ] Task 2.3 — Wire screenshot into detection pipeline | macOS ✅
  - In `lib.rs`: when `RewardScreenEvent` arrives, call `screenshot::capture_screen()`
  - Add configurable delay (default 0 ms — detection is already after screen appears)
  - Emit `"screenshot-taken"` Tauri event with image path so frontend can show it
  - Done: after triggering via a fake EE.log append, a screenshot file appears on disk
    and frontend receives the event

---

## Milestone 3 — OCR Pipeline

- [ ] Task 3.1 — Tesseract setup + `ocr.rs` scaffold | macOS ✅
  - Add `tesseract` crate to Cargo.toml
  - Install system deps: `tesseract-ocr` + `libtesseract-dev` (Linux); `tesseract` via
    Homebrew (macOS for dev)
  - Implement `extract_text(img: &DynamicImage) -> Result<String>` — raw OCR output
  - Done: passing any screenshot image returns a non-empty string without panicking
  - Ref: wfinfo-ng `ocr.rs` for initialisation flags and language data path

- [ ] Task 3.2 — Reward screen crop + pre-processing | macOS ✅
  - The 4 item name boxes occupy a predictable region of the reward screen
  - Implement `crop_reward_regions(img: &DynamicImage) -> Vec<DynamicImage>` that
    returns 4 cropped sub-images, one per reward slot
  - Pre-process each crop: convert to greyscale, threshold, upscale 2× for better OCR
  - Done: given a sample reward screen PNG, returns 4 crops that visually contain
    only the item name text
  - Ref: wfinfo-ng `theme.rs` for colour-detection approach to locate reward panel;
    store a sample reward screen PNG at `tests/fixtures/reward_screen.png`
  - Gotcha: panel position shifts slightly between 1080p, 1440p and 4K — use
    relative proportions, not hardcoded pixel offsets

- [ ] Task 3.3 — Item name extraction + normalisation | macOS ✅
  - Run Tesseract on each of the 4 crops; collect raw strings
  - Strip timestamps, newlines, junk characters; normalise whitespace
  - Implement `parse_item_names(raw_texts: Vec<String>) -> Vec<String>`
  - Done: given the sample reward_screen.png, the function returns 4 strings that
    are recognisably close to real Warframe item names (e.g. "Rhino Prime Neuroptics")
  - Ref: wfinfo-ng normalisation step; common OCR errors to handle:
    `0` → `O`, `1` → `I`, trailing `s` from "Relics" label

- [ ] Task 3.4 — Add OCR tests with fixture images | macOS ✅
  - Save 2–3 real reward screen screenshots (different resolutions if possible) to
    `src-tauri/tests/fixtures/`
  - Write `cargo test` integration tests in `ocr.rs` that load each fixture and
    assert the returned item names contain expected substrings
  - Done: `cargo test -- ocr` passes on all fixtures

---

## Milestone 4 — warframe.market API Client

- [ ] Task 4.1 — `market_api.rs`: fetch full item list | macOS ✅
  - Add `reqwest` (with `json` + `rustls-tls` features) and `serde` to Cargo.toml
  - `GET /v1/items` → deserialise into `Vec<MarketItem> { item_name, url_name }`
  - Done: `cargo test` makes a real HTTP call and returns > 1000 items
  - Note: use `rustls-tls` (not `native-tls`) to keep Flatpak deps minimal

- [ ] Task 4.2 — Fetch live orders for one item | macOS ✅
  - `GET /v1/items/{url_name}/orders` → filter `order_type == "sell"`,
    `platform == "pc"`, `status == "ingame"`
  - Compute median of lowest 5 sell prices as the platinum value
  - `async fn get_platinum_price(url_name: &str) -> Result<u32>`
  - Done: calling the function for `"rhino_prime_neuroptics"` returns a plausible
    price (e.g. 15–200 plat)

- [ ] Task 4.3 — Ducat value lookup | macOS ✅
  - Ducat values are fixed and do not require live API calls — they come from
    item rarity (Common = 15, Uncommon = 45, Rare = 100 ducats)
  - Fetch item details via `GET /v1/items/{url_name}` → read `ducats` field
  - `async fn get_ducat_value(url_name: &str) -> Result<u32>`
  - Done: function returns correct ducat value for known items

- [ ] Task 4.4 — Price cache: persist to disk | macOS ✅
  - On first run (or if cache is > 24 hours old): fetch all items + their ducat values
    and write to `~/.local/share/tennohelios/prices.json`
  - On subsequent runs: load from cache; fall back to cache if API unreachable
  - Implement `PriceCache { load(), save(), is_stale() -> bool }`
  - Done: after first run the JSON file exists on disk; second run loads without
    making any network requests; file contains > 1000 entries

- [ ] Task 4.5 — Item name fuzzy matching | macOS ✅
  - OCR output is imperfect; need to match raw text to a known item url_name
  - Add `strsim` or `rapidfuzz` crate; implement
    `fn fuzzy_match(raw: &str, items: &[MarketItem]) -> Option<&MarketItem>`
  - Strategy: normalise both sides (lowercase, remove punctuation), then
    Levenshtein / Jaro-Winkler; return None if best score is below threshold
  - Done: `"Rhino Prirne Neuroptics"` (typical OCR error) matches
    `"rhino_prime_neuroptics"` correctly; `"xyzxyz"` returns None
  - Ref: wfinfo-ng uses `levenshtein 1.0.5` crate

---

## Milestone 5 — Reward Data Assembly

- [ ] Task 5.1 — `RewardData` struct and assembly function | macOS ✅
  - Define shared types in `src-tauri/src/types.rs`:
    ```
    RewardItem { name: String, url_name: String, platinum: u32, ducats: u32 }
    RewardData  { items: Vec<RewardItem> }   // always 4 items
    ```
  - Implement `async fn build_reward_data(raw_names: Vec<String>, cache: &PriceCache) -> RewardData`
    — fuzzy-match each name, look up platinum + ducat values, fill in 0 if no match
  - Done: given 4 item name strings, returns a `RewardData` with all prices filled
    (or 0 for unrecognised items)

- [ ] Task 5.2 — Best-pick selection logic | macOS ✅
  - Add methods to `RewardData`:
    - `best_platinum_idx() -> usize` — index of item with highest plat value
    - `best_ducat_idx() -> usize` — index of item with highest ducat value
    - `best_overall_idx() -> usize` — highest plat; ties broken by ducats
  - Done: `cargo test` with hardcoded `RewardData` values returns correct indices;
    test edge cases: all equal, one item at 0

- [ ] Task 5.3 — Wire full pipeline: log → screenshot → OCR → prices → emit | macOS ⚠️
  - In `lib.rs`: chain log_watcher → screenshot → OCR → build_reward_data → emit
    `"reward-data-ready"` with `RewardData` serialised as JSON
  - Use Tokio for async; keep the watcher thread synchronous (mpsc channel handoff)
  - Done on macOS: manually appending the trigger line to a test EE.log causes
    `RewardData` JSON to appear in the app console (even if OCR output is junk on macOS)
  - Done on Linux: full pipeline returns recognisable item names from a real game screenshot

---

## Milestone 6 — Frontend UI

- [ ] Task 6.1 — `ItemCard.tsx` component | macOS ✅
  - Props: `{ name, platinum, ducats, highlight: "plat" | "ducat" | "none" }`
  - Layout: item name (top), plat value with ⬡ icon (middle), ducat value (bottom)
  - Colour: green border + bg tint for `"plat"`, yellow for `"ducat"`, neutral for `"none"`
  - Done: Storybook-style `pnpm dev` page shows all three highlight states side by side
    with hardcoded props

- [ ] Task 6.2 — `RewardOverlay.tsx` component | macOS ✅
  - Receives `RewardData`; renders 4 `ItemCard` components in a row
  - Arrow / crown icon above the `best_overall_idx` card
  - Dark semi-transparent backdrop behind the row; no border on the window edge
  - Done: renders correctly at 1280px (Steam Deck width) and 1920px; best-pick arrow
    moves when a different card is marked as best

- [ ] Task 6.3 — `App.tsx`: listen for `reward-data-ready` and show overlay | macOS ✅
  - Replace placeholder UI with `RewardOverlay` shown when event arrives
  - Overlay fades in (150 ms); auto-hides after 30 seconds
  - Done: calling the Tauri command `simulate_reward` (added for testing) makes the
    overlay appear with real-looking hardcoded data, then disappear after 30 s

- [x] Task 6.4 — `SettingsOverlay.tsx`: settings window | macOS ✅
  - Separate non-transparent window opened via system tray ("Settings" menu item)
  - Draggable, resizable (min 560×400, default 720×560), always on top
  - Settings: EE.log path (editable, Apply/Reset), screenshot delay (ms input),
    overlay Y position (slider, live preview), Warframe version (read from EE.log)
  - Capabilities: `core:window:allow-start-dragging` in `capabilities/settings.json`
  - Done: settings window opens, all controls work, changes apply immediately

---

## Milestone 7 — Hotkey + Overlay Window Behaviour

- [ ] Task 7.1 — Manual trigger hotkey | macOS ✅
  - ~~F12 removed~~ — decided against a global hotkey (interferes with in-game bindings)
  - Alternative: tray icon click or a dedicated button in Settings window
  - Skipped for now; revisit if users request it

- [ ] Task 7.2 — Always-on-top transparent overlay window | Linux 🐧
  - In `tauri.conf.json`: set `"alwaysOnTop": true`, `"transparent": true`,
    `"decorations": false`, `"skipTaskbar": true`
  - In CSS: `body { background: transparent }` — Tailwind backdrop uses `bg-black/60`
    on the card container only, not the root
  - Done: app window floats above Warframe on a standard Wayland/GNOME desktop,
    background is transparent, clicking through empty areas works
  - Gotcha: `transparent: true` requires compositor support; test on GNOME + Wayland

- [ ] Task 7.3 — Gamescope overlay layer | Linux 🐧
  - Gamescope respects `_STEAM_GAME_OVERLAY` X11 property to render a window as
    overlay inside the Gamescope session
  - Set the property via `xprop` or Tauri's `set_window_level` + custom X11 call
    on startup when `GAMESCOPE_XWAYLAND_DISPLAY` env var is present
  - Done: app window is visible on top of Warframe inside a Gamescope session on
    Bazzite; overlay renders above the game, not in a separate virtual desktop
  - Ref: wfinfo-ng `overlay.rs` for the X11 property approach
  - Gotcha: Steam Deck uses a slightly different Gamescope version than desktop
    Bazzite — test on both

- [ ] Task 7.4 — Auto-hide overlay when reward screen closes | Linux 🐧
  - Watch EE.log for the "reward accepted / closed" event:
    `"Got rewards"` or `"ProjectionRewardChoice destroyed"`
  - Emit `"reward-screen-closed"` from Rust; frontend listens and hides the overlay
  - Done: overlay disappears automatically after picking a reward in the game
  - Keep the 30-second timeout from Task 6.3 as a fallback

---

## Milestone 8 — Integration & Polish

- [ ] Task 8.1 — Error handling and user feedback | macOS ✅
  - Handle the common failure modes gracefully in the UI:
    - EE.log not found at startup → show "EE.log not found — check Settings"
    - Screenshot failed → show "Screenshot failed" in overlay
    - All 4 items unrecognised by OCR → show item names as "Unknown" with 0 values
    - API unreachable + no cache → show "Prices unavailable (offline)"
  - Done: each failure scenario shows a non-crashing message in the UI

- [ ] Task 8.2 — Price cache refresh on startup | macOS ✅
  - On app launch: check cache age; if > 24 hours, re-fetch in background
  - Show subtle "Updating prices…" indicator in corner; dismiss when done
  - Done: on a clean first launch, prices are fetched; on second launch within 24 h,
    no network calls are made (verify with a network proxy or log output)

- [ ] Task 8.3 — End-to-end smoke test on Bazzite | Linux 🐧
  - Run the full MVP on a real Warframe session:
    1. Open a Void relic mission
    2. Complete the mission to the reward screen
    3. Verify overlay appears with 4 correct items and correct price highlights
    4. Pick a reward; verify overlay auto-hides
    5. Press F12 manually; verify overlay shows again
  - Done: all 5 steps work without crashing; platinum values are plausible

- [ ] Task 8.4 — End-to-end smoke test on Steam Deck | Linux 🐧
  - Repeat Task 8.3 on Steam Deck in Game Mode (Gamescope active)
  - Verify overlay is visible and readable at 1280×800
  - Done: overlay renders correctly; F12 works via the Steam Deck's virtual keyboard
    or a connected controller shortcut

---

## Milestone 9 — Flatpak Packaging

- [ ] Task 9.1 — `flatpak/io.github.tennohelios.yml` manifest | Linux 🐧
  - Write the Flatpak manifest:
    - Runtime: `org.freedesktop.Platform` 23.08 (includes Wayland, X11 portals)
    - SDK: `org.freedesktop.Sdk` + `org.freedesktop.Sdk.Extension.rust-stable`
    - Bundled: Tesseract + language data (eng), WebKitGTK (via Tauri)
    - Permissions: `--filesystem=~/.local/share/Steam:ro` (read EE.log),
      `--filesystem=xdg-config/tennohelios:create` (settings),
      `--filesystem=xdg-data/tennohelios:create` (price cache),
      `--share=network` (warframe.market API),
      `--socket=wayland`, `--socket=fallback-x11`,
      `--env=TESSDATA_PREFIX=/app/share/tessdata`
  - Done: `flatpak-builder --run` launches the app without sandbox errors

- [ ] Task 9.2 — Build and install Flatpak locally | Linux 🐧
  - `flatpak-builder --user --install --force-clean build-dir flatpak/io.github.tennohelios.yml`
  - Launch via `flatpak run io.github.tennohelios`
  - Done: app launches from Flatpak, EE.log is readable, prices load, overlay works
  - Gotcha: Tesseract language data must be in `$TESSDATA_PREFIX` inside the sandbox;
    bundle `tessdata/eng.traineddata` as a Flatpak source

---

## Dependency Map

```
M1 (log watcher) ──► M2 (screenshot) ──► M3 (OCR) ──► M5 (assembly)
                                                             │
M4 (market API) ─────────────────────────────────────────► M5
                                                             │
                                                        M6 (UI) ──► M7 (hotkey/overlay)
                                                                          │
                                                                     M8 (integration)
                                                                          │
                                                                     M9 (Flatpak)
```

M4 can be built in parallel with M2 and M3.
M6 can be started with hardcoded mock data before M5 is complete.
M7.1 (F12 hotkey) can be built on macOS; M7.2–7.4 require Linux.
