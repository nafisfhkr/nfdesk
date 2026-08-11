# NFDesk Widget — MVP Product Requirements Document

**Project Name:** NFDesk Widget  
**Version:** 0.1.0  
**Status:** MVP  
**Platform:** Windows 10/11  
**License:** MIT  
**Architecture:** Tauri v2 + Rust + React + TypeScript  
**Storage Model:** Local-first / Offline-first

---

# 1. Product Overview

## 1.1 What is NFDesk?

NFDesk Widget is a small, floating desktop productivity companion designed to stay beside the user's main workspace.

Instead of forcing users to open a separate productivity application, NFDesk provides essential productivity tools directly on the Windows desktop:

- Pomodoro / Focus Timer
    
- Current Task
    
- Simple Task Checklist
    
- Quick Daily Note
    
- Animated Mascot
    
- Local Markdown / Obsidian integration
    

The application is designed to minimize friction during focused work.

### Product Philosophy

> **A tiny desktop companion for focused work.**

NFDesk is not intended to become a full project-management application.

The MVP should remain intentionally small, fast, local-first, and easy to use.

---

# 2. Problem Statement

When working on a computer, users often need to switch between their main workspace and productivity applications to:

- Start a focus timer
    
- Check what they are currently working on
    
- Write a quick thought
    
- Record a completed focus session
    
- Check or update simple tasks
    

These context switches create unnecessary friction.

NFDesk aims to solve this by providing a persistent, lightweight widget that stays close to the user's active workspace.

---

# 3. Target Users

## Primary User

Developers, students, designers, researchers, and knowledge workers who:

- Spend most of their time on a Windows computer
    
- Work in applications such as VS Code, browsers, terminals, or design tools
    
- Use Pomodoro or focused work sessions
    
- Prefer simple productivity tools
    
- Use Markdown or Obsidian for personal notes
    
- Prefer local-first software
    

## Secondary User

Users who want:

- A small desktop productivity widget
    
- A visual mascot companion
    
- Offline productivity tools
    
- Open-source productivity software
    

---

# 4. Product Goals

The MVP must achieve the following goals.

## Goal 1 — Zero-Friction Focus

A user should be able to start a focus session within a few seconds.

## Goal 2 — Persistent Desktop Presence

The widget should remain available while the user works in other applications.

## Goal 3 — Simple Task Context

The user should always be able to see what they are currently working on.

## Goal 4 — Fast Capture

The user should be able to write a quick note without opening Obsidian.

## Goal 5 — Local Ownership

User data must remain local.

No account, cloud database, or external API should be required.

## Goal 6 — Open Source Foundation

The MVP should be structured so the project can evolve through GitHub issues, pull requests, and community contributions.

---

# 5. MVP Scope

The MVP contains six core capabilities:

1. Desktop Widget
    
2. Pomodoro Timer
    
3. Current Task
    
4. Task Checklist
    
5. Quick Daily Note
    
6. Local Markdown / Obsidian Integration
    

Animated mascot is included as part of the core visual identity but should remain technically simple during MVP.

---

# 6. Feature Requirements

# 6.1 Desktop Widget

The application must run as a small frameless desktop window.

### Requirements

- Frameless window
    
- Rounded visual container
    
- Draggable window
    
- Always-on-top toggle
    
- Show/hide functionality
    
- System tray integration
    
- Global keyboard shortcut
    
- Remember previous window position
    
- Remember window visibility state where practical
    

### Default Shortcut

```text
Alt + Shift + N
```

The shortcut should be configurable in the future but may remain fixed in MVP.

### System Tray

The tray menu must provide:

```text
Show NFDesk
Hide NFDesk
Always on Top
Settings
Quit
```

---

# 6.2 Pomodoro Timer

The Pomodoro timer is the primary productivity feature.

## Default Configuration

```text
Focus: 25 minutes
Short Break: 5 minutes
```

The user should be able to customize durations through Settings.

## Timer States

```text
IDLE
  ↓
FOCUSING
  ↓
FOCUS_COMPLETED
  ↓
BREAK
  ↓
IDLE
```

The application should also support:

```text
FOCUSING
   ↓
PAUSED
   ↓
FOCUSING
```

## Timer Controls

The UI must provide:

- Start
    
- Pause
    
- Resume
    
- Reset
    
- Skip
    
- Start Break
    

## Current Task

Before starting a focus session, the user may specify:

```text
What are you working on?
```

Example:

```text
Build landing page generator
```

The current task should remain visible while the timer is running.

---

# 6.3 Focus Session Logging

When a focus session finishes, the user may save the session to their daily Markdown file.

Example:

```markdown
## Focus Sessions

- 14:00 - 14:25 — Build landing page generator
```

The log should include:

- Start time
    
- End time
    
- Duration
    
- Task name
    

Example:

```markdown
- **14:00–14:25** — Build landing page generator
```

Logging should not require an internet connection.

---

# 6.4 Task Checklist

NFDesk must provide a lightweight checklist.

Example:

```text
Today's Tasks

☑ Setup project
☑ Create database
□ Implement API
□ Write documentation
```

### MVP Requirements

Users can:

- Create a task
    
- Complete a task
    
- Uncomplete a task
    
- Delete a task
    
- View today's tasks
    

Tasks should persist between application restarts.

---

# 6.5 Quick Daily Note

The user must be able to write a short note directly from the widget.

Example:

```text
Quick Note

Need to investigate API upload error.
```

When saved:

```markdown
- **14:32** — Need to investigate API upload error.
```

### Requirements

- Text input
    
- Save button
    
- `Ctrl + Enter` shortcut
    
- Timestamp
    
- Append mode
    
- Clear input after successful save
    
- Error feedback when writing fails
    

---

# 6.6 Markdown / Obsidian Integration

NFDesk uses plain Markdown files as the primary external document format.

## Vault Configuration

The user can select a local folder:

```text
C:\Users\User\Documents\Obsidian Vault
```

The selected folder becomes the configured vault path.

## Daily Note

MVP assumes a configurable daily note filename pattern.

Default:

```text
YYYY-MM-DD.md
```

Example:

```text
2026-08-11.md
```

## MVP Behavior

NFDesk does not attempt to become a full Obsidian parser.

It only performs controlled Markdown operations:

- Create today's daily note if it does not exist
    
- Append quick notes
    
- Append focus session logs
    

---

# 7. Mascot

The mascot is a core part of NFDesk's product identity.

However, the MVP should prioritize functionality over complex animation.

## MVP States

### Idle

Calm animation.

### Focus

Focused/work animation.

### Break

Relaxed animation.

### Success

Short success animation after completing a session or task.

The exact character design and animation assets may evolve independently from the application architecture.

---

# 8. UI/UX Requirements

## Visual Direction

The interface should feel:

- Cozy
    
- Minimal
    
- Modern
    
- Friendly
    
- Slightly playful
    
- Desktop-native
    
- Non-distracting
    

Possible visual language:

```text
Dark mode
Rounded cards
Subtle transparency
Soft borders
Small shadows
Pastel accents
Minimal typography
```

Glassmorphism should be used carefully.

Performance and readability take priority over visual effects.

---

# 9. Keyboard Interaction

MVP keyboard interactions:

|Shortcut|Action|
|---|---|
|`Alt + Shift + N`|Show / Hide NFDesk|
|`Ctrl + Enter`|Save Quick Note|
|`Space`|Pause / Resume timer when timer is focused|
|`Esc`|Close temporary UI / dialog|

Shortcuts must not interfere unnecessarily with common applications.

---

# 10. Settings

MVP Settings should remain minimal.

## Required Settings

### General

- Always on Top
    
- Launch behavior
    

### Timer

- Focus duration
    
- Short break duration
    
- Long break duration may be deferred
    

### Storage

- Vault path
    
- Daily note filename format
    

### Appearance

- Theme
    
- Mascot enable/disable
    

---

# 11. Data Storage

NFDesk is local-first.

No user account is required.

The application may store internal application state separately from Markdown content.

Example:

```text
Application Data
│
├── settings
├── task state
└── application state
```

User-generated notes and focus logs are stored as Markdown inside the configured vault.

---

# 12. Offline-First Requirement

The MVP must function without an internet connection.

The application must not require:

- Login
    
- Cloud database
    
- Remote API
    
- Analytics service
    
- Online authentication
    

Internet access should not be required for normal operation.

---

# 13. Performance Requirements

Performance should be measured rather than relying on unrealistic hard limits.

## Targets

### Startup

Usable UI should become available within approximately:

```text
1.5–2 seconds
```

on a typical Windows development machine.

### Memory

The application should minimize idle memory usage.

The MVP should establish a real benchmark after implementation.

A strict `<50 MB RAM` requirement is intentionally removed because WebView2, React, animations, and Windows runtime behavior can make such a guarantee unrealistic.

### CPU

Idle CPU usage should remain low.

Animations should not continuously consume excessive CPU.

---

# 14. Reliability Requirements

The application must:

- Handle missing Markdown files
    
- Handle invalid vault paths
    
- Handle inaccessible folders
    
- Handle file write errors
    
- Avoid corrupting existing Markdown content
    
- Avoid silently losing user data
    
- Recover gracefully after restart
    

Before modifying a file, the application should validate the target path.

---

# 15. Security Requirements

NFDesk is a local desktop application.

The MVP should:

- Restrict filesystem access to required paths
    
- Avoid unnecessary network permissions
    
- Never upload user notes
    
- Never transmit Obsidian content externally
    
- Never require cloud authentication
    

---

# 16. MVP Out of Scope

The following features are intentionally excluded from MVP.

- Cloud synchronization
    
- User accounts
    
- Online authentication
    
- AI assistant
    
- AI task generation
    
- Google Calendar integration
    
- Notion integration
    
- Advanced analytics
    
- Multi-device synchronization
    
- Mobile application
    
- Plugin marketplace
    
- Complex Obsidian task parsing
    
- Full Obsidian database integration
    
- Custom mascot editor
    
- Multiple mascot marketplace
    
- Team collaboration
    
- Social features
    
- Cross-platform optimization
    
- Automatic project management
    
- Full task management system
    

These features may be considered in future roadmap versions.

---

# 17. MVP Acceptance Criteria

The MVP is considered complete when a user can perform this workflow:

```text
Launch NFDesk
      ↓
Widget appears
      ↓
Set current task
      ↓
Start 25-minute focus
      ↓
Work in VS Code/browser/etc.
      ↓
Pause/resume if needed
      ↓
Focus session completes
      ↓
Save focus log
      ↓
Log appears in today's Markdown file
      ↓
Write quick note
      ↓
Quick note appears in today's Markdown file
      ↓
Check/uncheck tasks
      ↓
Close NFDesk
      ↓
Reopen NFDesk
      ↓
State remains available
```

---

# 18. Definition of Done

The MVP is considered technically ready when:

- Windows build works
    
- Application can be installed/launched
    
- System tray works
    
- Global shortcut works
    
- Window can be dragged
    
- Always-on-top works
    
- Timer works correctly
    
- Tasks persist
    
- Quick notes work
    
- Markdown writing works
    
- Invalid paths are handled
    
- Application works offline
    
- No critical data-loss bugs exist
    
- README exists
    
- LICENSE exists
    
- Basic contribution guide exists
    
- GitHub Release can be created
    

---

# 19. Product Principle

When deciding whether a feature belongs in MVP, ask:

> **Does this make focused work faster or easier without adding unnecessary complexity?**

If the answer is no, defer the feature.

NFDesk should remain a **small productivity companion**, not become a full productivity suite.