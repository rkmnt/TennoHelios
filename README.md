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

## 🎯 Features

### Phase 1 — MVP (in progress)

- 🔍 **Automatic reward detection** — watches `EE.log` for relic reward screen events
- 📸 **Instant screenshot** — captures the screen the moment the reward UI appears
- 🔤 **OCR item recognition** — extracts all 4 item names using Tesseract
- 💰 **Live platinum prices** — fetches real sell orders from [warframe.market](https://warframe.market)
- 🪙 **Ducat values** — shows Void Trader ducat value for every item
- 🟢 **Best pick highlight** — green = best platinum, yellow = best ducats, arrow on the overall winner
- ⌨️ **F12 manual trigger** — force the overlay any time you need it
- 🎮 **Gamescope overlay** — renders on top of Warframe inside Gamescope (no alt-tab needed)
- 💾 **Offline price cache** — works even when warframe.market is unreachable

### Coming Soon

- 📦 **Inventory tracker** — know your total platinum worth at a glance *(Phase 2)*
- 🪙 **Ducat optimizer** — find the best items to sell to Baro Ki'Teer *(Phase 2)*
- 💬 **Trade chat generator** — auto-format WTS/WTB messages *(Phase 2)*
- 📊 **Session earnings** — track platinum earned per play session *(Phase 2)*
- 🔔 **Deal alerts** — get notified when an item drops below your price threshold *(Phase 2)*
- ⚔️ **Riven analyzer** — price estimation for rivens *(Phase 3)*
- 🌀 **Void fissure tracker** — see which active fissures are worth running *(Phase 3)*
- 🖥️ **Native Wayland overlay** — without requiring Gamescope *(Phase 3)*

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
│       ├── RewardOverlay.tsx   4-item reward display
│       ├── ItemCard.tsx        Single item card with plat + ducat
│       └── Settings.tsx        App settings
└── flatpak/            Flatpak manifest (Milestone 9)
```

---

## 🗺️ Roadmap

### ✅ Phase 0 — Foundation
- [x] Project scaffold (Tauri + React + TypeScript + Tailwind)
- [x] `log_watcher.rs` — EE.log detection with full test suite

### 🔧 Phase 1 — MVP (active)
- [ ] Screenshot capture (X11 + Wayland/Gamescope)
- [ ] OCR pipeline with Tesseract
- [ ] warframe.market API client + price cache
- [ ] Reward overlay UI with platinum/ducat highlights
- [ ] F12 manual trigger
- [ ] Gamescope overlay layer
- [ ] Flatpak packaging

### 📋 Phase 2 — Power Tools
- [ ] Inventory tracker
- [ ] Ducat optimizer (Baro Ki'Teer prep)
- [ ] Trade chat message generator
- [ ] Session earnings tracker
- [ ] Deal alerts

### 🚀 Phase 3 — Advanced
- [ ] Riven analyzer
- [ ] Void fissure tracker
- [ ] Native Wayland overlay (without Gamescope)
- [ ] Steam Deck UI optimizations

Full task breakdown with implementation details: [TASKS.md](TASKS.md)

---

## 🤝 Contributing

Contributions are welcome — and genuinely needed. This is a one-person side project
targeting a platform that most Warframe tooling ignores.

If you use Warframe on Linux, you are the target audience. Your bug reports, real
screenshot fixtures, and EE.log samples are just as valuable as code.

**Good places to start:**

- Pick any unchecked task from [TASKS.md](TASKS.md)
- Test on your specific hardware (Steam Deck? Bazzite? Arch + Gamescope?)
- Contribute real reward screen fixtures for the OCR test suite (`src-tauri/tests/fixtures/`)
- Report EE.log trigger patterns that are missing or wrong

```bash
# Fork → branch → PR
git checkout -b feature/your-feature
# ... make changes ...
git push origin feature/your-feature
```

Please keep PRs focused — one task per PR. Every PR must compile and pass
`cargo test --lib` and `pnpm build`.

---

## ☕ Support

TennoHelios is free forever. If it saves you platinum, consider buying me a coffee.

[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20the%20project-ff5e5b?logo=ko-fi&logoColor=white)](https://ko-fi.com)
[![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-Sponsor-ea4aaa?logo=github&logoColor=white)](https://github.com/sponsors)

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
