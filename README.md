# NFDesk Widget

<p align="center">
  <strong>A tiny, local-first desktop productivity companion for focused work.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%2F11-0078d7?logo=windows" alt="Platform" />
  <img src="https://img.shields.io/badge/Architecture-Tauri%20v2%20%2B%20Rust-orange?logo=rust" alt="Architecture" />
  <img src="https://img.shields.io/badge/Frontend-React%2019%20%2B%20TypeScript-blue?logo=react" alt="Frontend" />
  <img src="https://img.shields.io/badge/Styling-Tailwind%20CSS-38bdf8?logo=tailwindcss" alt="Styling" />
</p>

---

## 📖 Overview

**NFDesk** is a lightweight, floating desktop widget designed to live right beside your main workspace (VS Code, terminal, browser, or design tools). 

Instead of switching back and forth between full-sized productivity apps, NFDesk keeps your focus timer, daily task checklist, and quick capture notes persistently accessible on your screen with zero friction and 100% offline local privacy.

---

## 📥 Download & Quick Install

Get the latest release for **Windows 10 / 11** (64-bit):

<p align="center">
  <a href="https://github.com/nafisfhkr/nfdesk/releases/download/v0.1.0/NFDesk_0.1.0_x64-setup.exe">
    <img src="https://img.shields.io/badge/Download_NFDesk_v0.1.0-(.exe)_~2.2_MB-6366f1?style=for-the-badge&logo=windows&logoColor=white" alt="Download NFDesk for Windows" />
  </a>
</p>

| Package | Format | Architecture | Direct Download |
|---|---|---|---|
| **Windows Setup Installer** | `.exe` *(Standar)* | `x64` (64-bit) | [**Download `NFDesk_0.1.0_x64-setup.exe`**](https://github.com/nafisfhkr/nfdesk/releases/download/v0.1.0/NFDesk_0.1.0_x64-setup.exe) (~2.2 MB) |

> 💡 **Quick Install:** Simply download the `.exe`, run the installer, and launch NFDesk from your desktop or start menu.

---

## 💡 Why NFDesk? (The Motivation)

NFDesk was born out of a real personal problem: **constant context-switching and losing focus across dozens of open tabs and apps.**

When deep into coding or working, switching to full-blown productivity suites or opening heavy note apps just to start a timer, record a task, or write a single line of thought breaks deep concentration and causes distraction.

NFDesk solves this by sitting quietly in the corner of your desktop:
- **Stay in Flow:** No need to switch tabs or minimize your IDE just to manage your focus session.
- **Immediate Task Context:** Always see exactly what you are currently focusing on without mental clutter.
- **Fast Obsidian Capture:** Save quick thoughts and check off tasks directly into your local Obsidian markdown files.

> **Note on Project Status:** NFDesk is currently in its **MVP (Minimum Viable Product)** stage, intentionally keeping things fast, silent, and zero-distraction for daily focused work. It will continue to evolve and grow with additional features over time.

---

## ✨ Core Features

- ⏱️ **Pomodoro / Focus Timer:**
  - Drift-free timestamp-based timer calculations (remains accurate during OS sleep or lag).
  - Configurable Focus and Short Break durations via Settings.
  - "What are you working on?" context input that locks during active focus sessions.
  - Interactive circular progress ring and status indicators.

- 📝 **Obsidian & Markdown Task Synchronization:**
  - Daily checklist synchronized directly with your local Obsidian Vault.
  - Formatted in standard markdown checklist syntax (`- [ ]` and `- [x]`).
  - Isolated daily files organized in configurable subfolders (`Tasks/YYYY-MM-DD.md`).

- ⚡ **Quick Daily Note Capture:**
  - Fast thought capture without opening external note apps.
  - Auto-timestamped entries (`- **HH:MM** — note content`) appended to `Daily Notes/YYYY-MM-DD.md`.
  - Instant visual feedback with toast notifications.

- 📊 **Focus Session Logging:**
  - Log completed focus sessions directly to your daily markdown note with one click (`- **HH:MM–HH:MM** — Task Name`).

- 🪟 **Desktop-Native Window Controls:**
  - Frameless dark glassmorphism container.
  - Draggable window header.
  - **Always on Top (Pin):** Keeps the widget floating above active windows.
  - **System Tray:** Minimize to tray, restore, or quit anytime.
  - **Autostart:** Toggle launch at Windows startup from Settings.
  - Window position persistence across restarts.

---

## ⌨️ Keyboard Shortcuts

NFDesk is built for speed and seamless keyboard-driven workflows:

| Shortcut | Scope | Action |
|---|---|---|
| **`Alt + Shift + N`** | **Global (OS-wide)** | Toggle Show / Hide NFDesk widget from any application. |
| **`Space`** | **Timer Tab** | Start / Pause / Resume the focus timer *(Includes smart typing guard)*. |
| **`Ctrl + Enter`** | **Note Tab** | Instantly save quick note to today's daily markdown file. |
| **`Enter`** | **Tasks Tab** | Add a new task item to today's checklist. |
| **`Escape (Esc)`** | **Settings Modal** | Instantly dismiss / close the Settings modal. |

> **Note on Spacebar Shortcut:** The `Space` key timer toggle is protected by an intelligent typing guard. When you are typing inside the task title or note textarea, the spacebar functions normally without interrupting or toggling the timer.

---

## 📂 Obsidian Vault Structure

NFDesk organizes user notes and tasks cleanly inside your configured Obsidian Vault folder:

```text
Your Obsidian Vault/
├── Daily Notes/
│   ├── 2026-08-14.md
│   └── 2026-08-15.md       <-- Quick notes & Focus session logs
│
└── Tasks/
    ├── 2026-08-14.md
    └── 2026-08-15.md       <-- Daily checklist (- [ ] / - [x])
```

*Subfolder names (`Daily Notes` and `Tasks`) can be customized in the Settings modal.*

---

## 🛠️ Tech Stack

- **Desktop Framework:** [Tauri v2](https://v2.tauri.app/) (Rust backend)
- **Frontend:** [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/)
- **Styling & UI:** [Tailwind CSS](https://tailwindcss.com/) + [Framer Motion](https://www.framer.com/motion/)
- **Icons:** [Lucide React](https://lucide.dev/)
- **Storage Model:** 100% Local-first / Offline-first

---

## 🛠️ Building from Source (For Developers)

If you wish to contribute or build NFDesk from the source code:

### Prerequisites

1. **Node.js:** `v18+` or `v20+` ([Download Node.js](https://nodejs.org/))
2. **Rust:** Latest stable toolchain ([Install Rust](https://www.rust-lang.org/tools/install))
3. **C++ Build Tools:** Visual Studio C++ Build Tools on Windows (required by Tauri/Rust)

### Clone & Build

1. **Clone the repository:**
   ```bash
   git clone https://github.com/nafisfhkr/nfdesk.git
   cd nfdesk
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

4. **Build production executable:**
   ```bash
   npm run tauri build
   ```
   The compiled installer and `.exe` will be located in `src-tauri/target/release/`.

---

## 🧪 Testing

- **Frontend build & type-checking:**
  ```bash
  npm run build
  ```

- **Backend Rust unit tests:**
  ```bash
  cd src-tauri
  cargo test
  ```
