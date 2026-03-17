# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# TennoHelios — Warframe Linux Overlay Tool

## Project Overview
We are building a Linux-native Warframe companion overlay application, similar to
Alecaframe (Windows-only), but built specifically for Linux and Steam Deck.
The app will be free and open source (GPL-3.0), distributed as a Flatpak package
with a simple landing website. Monetization is via voluntary donations (Ko-fi,
GitHub Sponsors).

The name "TennoHelios" comes from Warframe lore:
- Tenno = the players
- Helios = the companion that scans and analyzes everything

## Target Platforms
- Primary: Bazzite (immutable Fedora-based gaming distro) with GNOME desktop
- Secondary: Steam Deck (SteamOS — also uses Gamescope natively)
- Wayland display server (with XWayland compatibility)
- Warframe running via Steam + Proton
- Gamescope compositor for overlay support (pre-installed on both Bazzite and SteamOS)
- Distribution: Flatpak (eventually published to Flathub)

## Tech Stack
- **Backend**: Rust
- **Frontend/UI**: Tauri + React + Tailwind CSS
- **OCR**: Tesseract (via Rust bindings)
- **Screen capture**: libxrandr / Wayland screenshot APIs
- **Game event detection**: EE.log file watching
- **Pricing data**: warframe.market public REST API
- **Overlay**: Gamescope layer (transparent always-on-top window)
- **Distribution**: Flatpak

## EE.log Default Location (Steam/Proton)
~/.local/share/Steam/steamapps/compatdata/230410/pfx/drive_c/users/steamuser/
AppData/Local/Warframe/EE.log

## Reference Project
https://github.com/knoellle/wfinfo-ng
- Existing open source Linux Warframe overlay written in Rust (GPL-3.0)
- Study it carefully before writing any code
- Reuse or draw inspiration from: OCR pipeline, EE.log parsing, screenshot logic
- It is outdated and missing many features — TennoHelios is the modern replacement

## Project Structure
tennohelios/
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── log_watcher.rs          # EE.log file watcher
│   │   ├── screenshot.rs           # Screen capture (X11 + Wayland)
│   │   ├── ocr.rs                  # Tesseract OCR pipeline
│   │   ├── market_api.rs           # warframe.market API client
│   │   └── overlay.rs              # Window management
│   └── Cargo.toml
├── src/                            # React frontend
│   ├── components/
│   │   ├── RewardOverlay.tsx       # Relic reward screen display
│   │   ├── ItemCard.tsx            # Single item with plat + ducat value
│   │   └── Settings.tsx            # App settings
│   ├── App.tsx
│   └── main.tsx
├── flatpak/
│   └── io.github.tennohelios.yml   # Flatpak manifest
├── website/                        # Landing page (Phase 4)
├── CLAUDE.md                       # This file
└── README.md

## Core Features — Phase 1 (MVP)
1. **Relic reward screen detection**
    - Watch EE.log for reward screen events
    - Automatically trigger screenshot on detection
    - OCR to extract item names from screenshot
    - Look up platinum + ducat values via warframe.market API
    - Display overlay with all 4 items and their values
    - Highlight best platinum choice (green) and best ducat choice (yellow)
    - Arrow indicator on the single best overall pick

2. **Overlay window**
    - Transparent always-on-top window rendered via Tauri WebView
    - Works inside Gamescope session (Bazzite + Steam Deck)
    - Auto-hides when reward screen disappears
    - Manual trigger hotkey: F12
    - Non-intrusive, minimal screen space

3. **Price database**
    - Fetch and cache item prices from warframe.market
    - Auto-update on app launch
    - Offline fallback using last cached data
    - update.sh script for manual refresh

## Phase 2 Features
- Inventory tracker (owned items + total estimated platinum value)
- Ducat optimizer (best items to sell to Void Trader Baro Ki'Teer)
- Trade chat message auto-generator (WTS/WTB format)
- Session earnings tracker (platinum earned per play session)
- Deal alerts (notify when item drops below price threshold)
- Syndicate standing tracker

## Phase 3 Features
- Riven analyzer with price estimation
- Void fissure tracker (which active fissures are worth running)
- Multi-display support
- Native Wayland overlay (without requiring Gamescope)
- Steam Deck specific UI optimizations (larger touch targets etc.)

## Phase 4
- Landing website (tennohelios.app)
- Ko-fi and GitHub Sponsors donation links
- Flathub submission
- Reddit/Discord community announcement

## Design Guidelines
- Dark semi-transparent overlay — does not obstruct gameplay
- Color coding:
    - 🟢 Green = best platinum value
    - 🟡 Yellow = best ducat value
    - ⬜ White = standard item
- Warframe aesthetic: dark, sci-fi, clean minimal UI
- Responsive: works on 1080p, 1440p, 4K and Steam Deck (1280x800)
- Animations should be subtle and fast — never distracting during gameplay

## Warframe Market API
Base URL: https://api.warframe.market/v1
- GET /items — full list of all tradeable items
- GET /items/{url_name}/orders — live buy/sell orders
- GET /items/{url_name}/statistics — price history and averages
- Platform: pc
- Language: en
- No authentication required for read operations
- Be respectful with request rate — cache aggressively

## Project Identity
- App name: TennoHelios
- Flatpak ID: io.github.tennohelios
- GitHub: github.com/[username]/tennohelios (public, GPL-3.0)
- Website: tennohelios.app (planned)
- Licence: GPL-3.0 (same as wfinfo-ng reference project)
- Monetization: voluntary donations only (Ko-fi + GitHub Sponsors)

## Important Constraints
- Must work without root or sudo
- Flatpak sandbox compatible (document any required permissions)
- Low CPU and memory usage — Warframe is already demanding
- Must not violate Warframe Terms of Service:
  ✅ Screenshot capture
  ✅ EE.log file reading
  ✅ warframe.market public API
  ❌ No game memory reading or injection
  ❌ No game file modification
  ❌ No automation of in-game actions

## Development Approach
- Always focus on current phase only — do not jump ahead
- Get each component working and tested before integrating
- Reference wfinfo-ng carefully before writing any Rust code
- Keep code clean, commented and ready for open source contributors
- Every PR or feature should work on both Bazzite and Steam Deck

## Commands

```bash
# Frontend
pnpm dev              # Vite dev server only (port 1420)
pnpm build            # TypeScript check + Vite build

# Tauri (runs both Vite + Rust)
pnpm tauri dev        # Full dev mode (hot reload)
pnpm tauri build      # Release build

# Rust (run from src-tauri/)
cargo test --lib      # Run all unit tests
cargo test --lib -- log_watcher  # Run only log_watcher tests
cargo clippy          # Lint
cargo check           # Type-check without building

# Logging (set before running dev or tests)
RUST_LOG=debug pnpm tauri dev          # Verbose Rust logs
RUST_LOG=tennohelios_lib=debug cargo test --lib  # Debug logs in tests
```

## Architecture

```
src-tauri/src/
├── main.rs          — binary entry point (calls lib::run)
├── lib.rs           — Tauri setup, spawns log_watcher thread, forwards events to frontend
│                      exposes set_log_path Tauri command (useful for dev/testing with custom EE.log)
└── log_watcher.rs   — EE.log file watcher (notify crate), pattern detection, channel send
src/
├── App.tsx          — Listens for "reward-screen-detected" Tauri event
└── components/      — RewardOverlay, ItemCard, Settings (Phase 1, not yet built)
```

**Rust lib crate name:** `tennohelios_lib` (set in `Cargo.toml` `[lib]`). Use this name when filtering test output or setting `RUST_LOG`.

**Custom Tailwind tokens** (defined in `tailwind.config.js`): `wf-bg`, `wf-surface`, `wf-border`, `wf-text`, `wf-plat` (green/platinum), `wf-ducat` (yellow/ducats), `wf-accent` (sky blue). Use these for all new UI components — do not add ad-hoc hex colors.

**CSS animations** (defined in `src/index.css`): `card-in`, `scan-sweep`, `glow-pulse`, `arrow-bounce`, `header-in`, `best-pick-in`, `bracket-left`, `bracket-right`, `best-pick-text`. All animations use inline `style` props (not Tailwind `animate-*` classes) to support per-element delays.

**Frontend component tree:**
```
App.tsx
└── RewardOverlay.tsx        — 4-card layout + header + BestPickIndicator
    ├── BestPickIndicator.tsx — animated brackets + chevron above best card
    └── ItemCard.tsx          — single item card (uses useCountUp hook + PlatIcon/DucatIcon)
        └── icons.tsx         — PlatIcon, DucatIcon (real PNG assets from Warframe wiki)
src/hooks/useCountUp.ts       — animates number from 0 to target value
src/assets/plat.png           — Warframe platinum icon (64×64)
src/assets/ducat.png          — Warframe ducat icon (512×512, rendered larger to match visual weight)
```

**Data flow (Phase 1):**
`EE.log write` → `notify crate` → `log_watcher::read_new_lines()` → `mpsc channel` → `lib.rs forwarder` → `tauri Emitter::emit("reward-screen-detected")` → `App.tsx listen()`

**Key EE.log trigger line:**
`"Created /Lotus/Interface/ProjectionRewardChoice.swf"` — fires when the relic reward choice screen appears. Fallback: `"Pause countdown done"`.

**notify on macOS vs Linux:**
The `RecommendedWatcher` uses FSEvents on macOS and inotify on Linux. FSEvents is unreliable in test contexts (symlinked temp paths), so the integration test uses a polling loop instead. Production behaviour on Linux is fine.

## Phase Status
- [x] Phase 1 scaffolding: Tauri + React + TypeScript + Tailwind
- [x] `log_watcher.rs` — file watching + pattern detection + Tauri event emit
- [x] `ItemCard.tsx` — full design with animations, Warframe icons, count-up
- [x] `RewardOverlay.tsx` — 4-card layout, BestPickIndicator, header
- [x] `BestPickIndicator.tsx` — animated brackets, chevron, glow pulse
- [x] `useCountUp.ts` — number scan-up animation hook
- [x] `src/assets/plat.png` + `ducat.png` — real Warframe icons (from wiki)
- [ ] `screenshot.rs` — screen capture (next after reboot)
- [ ] `ocr.rs` — Tesseract OCR pipeline
- [ ] `market_api.rs` — warframe.market REST client
- [ ] Wire full pipeline: log → screenshot → OCR → prices → overlay

## Next Session (after reboot)
1. `pnpm tauri dev` — první spuštění po `rpm-ostree install` (kompilace ~10 min)
2. Ověřit že log watcher detekuje EE.log v živé hře
3. Začít `screenshot.rs` — Task 2.1 (xcap crate, X11/XWayland capture)
4. Pak Task 2.2 — Wayland path přes `grim`