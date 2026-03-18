# ⚙️ TennoHelios

```
  ████████╗███████╗███╗   ██╗███╗   ██╗ ██████╗ ██╗  ██╗███████╗██╗     ██╗ ██████╗ ███████╗
     ██╔══╝██╔════╝████╗  ██║████╗  ██║██╔═══██╗██║  ██║██╔════╝██║     ██║██╔═══██╗██╔════╝
     ██║   █████╗  ██╔██╗ ██║██╔██╗ ██║██║   ██║███████║█████╗  ██║     ██║██║   ██║███████╗
     ██║   ██╔══╝  ██║╚██╗██║██║╚██╗██║██║   ██║██╔══██║██╔══╝  ██║     ██║██║   ██║╚════██║
     ██║   ███████╗██║ ╚████║██║ ╚████║╚██████╔╝██║  ██║███████╗███████╗██║╚██████╔╝███████║
     ╚═╝   ╚══════╝╚═╝  ╚═══╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚══════╝
```

### Built for Linux. Built for Tenno.

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Steam%20Deck-brightgreen)](https://github.com/rkmnt/TennoHelios)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-orange)](https://tauri.app)
[![Ko-fi](https://img.shields.io/badge/support-Ko--fi-ff5e5b)](https://ko-fi.com)

---

**TennoHelios** is a free, open source Warframe companion overlay for Linux and Steam Deck.
When your relic reward screen appears, it instantly shows you the platinum and ducat value of every choice — so you always pick the best reward, without alt-tabbing or guessing.

Named after Helios — the Warframe companion that scans and analyzes everything, so you don't have to.

> No Windows required. No subscriptions. No bullshit.

---

## 🗺️ Roadmap

### ✅ Phase 0 — Foundation
- [x] 🔧 Project scaffold — Tauri + React + TypeScript + Tailwind
- [x] 📋 `log_watcher.rs` — EE.log detection with full test suite

### ✅ Phase 1 — MVP (complete)
- [x] 📸 Screenshot capture — X11/XWayland via ImageMagick `import`
- [x] 🔤 OCR pipeline — per-card Tesseract strips, 4 items detected
- [x] 💰 warframe.market client — live plat prices (v1 stats API) + ducat values (v2 items API)
- [x] 🟢 Reward overlay UI — green best plat · yellow best ducats · arrow on overall winner
- [x] 🖥️ System tray — tray icon with Settings and Quit menu items
- [x] ⚙️ Settings window — draggable, resizable; EE.log path · screenshot delay · overlay position · Warframe version
- [ ] 🎮 Gamescope overlay layer — renders on top of Warframe without alt-tab
- [ ] 📦 Flatpak packaging

### 📋 Phase 2 — Power Tools
- [ ] 📦 Inventory tracker — total platinum worth at a glance
- [ ] 🪙 Ducat optimizer — best items to sell to Baro Ki'Teer
- [ ] 💬 Trade chat generator — auto-format WTS/WTB messages
- [ ] 📊 Session earnings — track platinum earned per play session
- [ ] 🔔 Deal alerts — notify when an item drops below your price threshold

### 🚀 Phase 3 — Advanced
- [ ] ⚔️ Riven analyzer — price estimation for rivens
- [ ] 🌀 Void fissure tracker — which active fissures are worth running
- [ ] 🖥️ Native Wayland overlay — without requiring Gamescope
- [ ] 🎮 Steam Deck UI optimizations — larger touch targets, controller-friendly layout

Full implementation details and task breakdown: [TASKS.md](TASKS.md)

---

## 🖥️ Supported Platforms

| Platform | Status |
|---|---|
| **Bazzite** (GNOME + Wayland + Gamescope) | ✅ Primary target |
| **Steam Deck** (SteamOS + Gamescope) | ✅ Fully supported |
| Any Linux with Gamescope + Warframe via Steam + Proton | ✅ Should work |
| Windows / macOS | ❌ Not planned |

> Warframe must be running via **Steam + Proton**. The Flatpak release requires no extra setup.

---

## 📦 Installation

### Flatpak (recommended)

> **Coming soon** — Flatpak packaging is tracked in [TASKS.md](TASKS.md) (Milestone 9).
> Until then, build from source.

```bash
# Future install command — not yet available
flatpak install flathub io.github.tennohelios
```

### Build from Source

See the [Development Setup](#-development-setup) section below.

---

## 🛠️ Development Setup

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | stable (≥ 1.80) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥ 20 | via [nvm](https://github.com/nvm-sh/nvm) or your distro |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Tesseract | ≥ 5 + `eng` data | `sudo apt install tesseract-ocr` / `sudo dnf install tesseract` |
| Tauri system deps | — | See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your distro |

### Clone and Run

```bash
git clone https://github.com/rkmnt/TennoHelios.git
cd TennoHelios

# Install JS dependencies
pnpm install

# Start in dev mode (hot reload for both Rust and React)
pnpm tauri dev
```

### Run Tests

```bash
# All Rust unit tests
cd src-tauri && cargo test --lib

# TypeScript check
pnpm build
```

### Project Structure

```
TennoHelios/
├── src-tauri/          Rust backend (Tauri)
│   └── src/
│       ├── log_watcher.rs   EE.log file watcher + trigger detection
│       ├── screenshot.rs    Screen capture (X11 + Wayland)
│       ├── ocr.rs           Tesseract OCR pipeline
│       ├── market_api.rs    warframe.market REST client + price cache
│       └── lib.rs           Tauri setup, event wiring
├── src/                React + TypeScript frontend
│   └── components/
│       ├── RewardOverlay.tsx      4-item reward display
│       ├── ItemCard.tsx           Single item card with plat + ducat
│       ├── BestPickIndicator.tsx  Animated arrow + brackets above best card
│       └── SettingsOverlay.tsx   Settings window (log path, delay, overlay position)
└── flatpak/            Flatpak manifest (Milestone 9)
```

---

## 🤖 AI Disclosure

This project is developed with the assistance of **Claude AI** by [Anthropic](https://anthropic.com).

We believe in full transparency: Claude helps write and review code, structure
architecture, and draft documentation. The vision, design decisions, and direction
are driven by a human who plays Warframe on Linux and got tired of alt-tabbing.

AI-assisted development doesn't make the code less yours to read, audit, fork, or improve.
The GPL-3.0 license means you have every right to do all of the above.

---

## ⚖️ License

TennoHelios is licensed under the **GNU General Public License v3.0**.
See [LICENSE](LICENSE) for the full text.

---

*Not affiliated with Digital Extremes. Warframe is a trademark of Digital Extremes.*
