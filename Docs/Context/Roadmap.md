# NFDesk Widget — Roadmap

**Project:** NFDesk Widget  
**License:** MIT  
**Current Version:** 0.1.0  
**Status:** Open Source / Early Development

---

# 1. Roadmap Philosophy

NFDesk will be developed incrementally.

The project should not attempt to become a complete productivity suite immediately.

The development strategy is:

```text
Build
  ↓
Use
  ↓
Observe problems
  ↓
Improve
  ↓
Release
  ↓
Collect feedback
  ↓
Repeat
```

The developer should use NFDesk personally before prioritizing large new features.

---

# 2. Release Strategy

Versioning follows semantic versioning:

```text
MAJOR.MINOR.PATCH
```

Example:

```text
0.1.0
0.2.0
0.3.0
1.0.0
```

During the `0.x` phase, features and architecture may change significantly.

---

# 3. Phase 0 — Prototype

**Status:** Planned

## Objective

Validate whether the basic interaction feels useful before investing heavily in architecture and visual polish.

## Features

- Tauri application
- Small floating window
- Basic React UI
- Basic Pomodoro timer
- Placeholder mascot
- Drag window
- Always-on-top

## Success Criteria

The developer can keep the widget beside VS Code/browser and use it comfortably for a real work session.

---

# 4. Phase 1 — MVP Core

**Target:** v0.1.0

## Objective

Build the smallest genuinely useful NFDesk.

### Desktop

- Frameless window
- Dragging
- Always-on-top
- System tray
- Show/hide
- Global shortcut

### Productivity

- Pomodoro
- Pause/resume
- Current task
- Task checklist
- Quick note

### Storage

- Local application state
- Vault path
- Daily Markdown
- Focus session logging
- Quick note logging

### Mascot

- Idle
- Focus
- Break
- Success

### Documentation

- README
- LICENSE
- CONTRIBUTING
- Basic architecture documentation

---

# 5. Phase 2 — Stability & UX

**Target:** v0.2.0

## Objective

Make NFDesk pleasant and reliable enough for daily use.

### Improvements

- Better keyboard-first interaction
- Better error messages
- Improved animations
- Better window positioning
- Remember widget position
- Better startup behavior
- Improved settings
- Better timer notifications
- Sound settings
- Theme settings

### Reliability

- Better filesystem error handling
- More timer tests
- More Windows testing
- Sleep/wake handling
- Multi-monitor testing

---

# 6. Phase 3 — Focus History

**Target:** v0.3.0

## Objective

Allow users to understand their focus habits without turning NFDesk into a complex analytics application.

Potential features:

```text
Today's focus
Weekly focus
Number of sessions
Total focused minutes
Completed tasks
```

Example:

```text
Today

Focus Sessions       5
Focused Time      2h 05m
Tasks Completed       8
```

The initial analytics should remain local.

---

# 7. Phase 4 — Better Obsidian Integration

**Target:** v0.4.0

Only pursue this phase if users actually need deeper Obsidian support.

Potential features:

- Configurable daily note directory
- Custom note templates
- Custom Markdown sections
- Frontmatter support
- Tags
- Project-specific logs
- Better task synchronization

Potential configuration:

```text
Daily Notes/
    YYYY/
        MM/
            YYYY-MM-DD.md
```

The integration should remain non-destructive.

---

# 8. Phase 5 — Customization

**Target:** v0.5.0

## Objective

Allow users to make NFDesk feel personal.

Potential features:

### Themes

```text
Dark
Light
Cozy
Retro
Minimal
```

### Mascots

Users may eventually be able to select different mascot assets.

### Widget Layout

Potential options:

```text
Compact
Normal
Expanded
```

### Appearance

- Accent color
- Transparency
- Border
- Animation intensity
- Mascot visibility

---

# 9. Phase 6 — Developer / Power User Features

**Target:** v0.6.0+

Potential features:

- Advanced keyboard shortcuts
- Command palette
- Quick actions
- Multiple timers
- Custom timer presets
- Task filtering
- Project context
- Custom Markdown templates
- CLI integration

Possible command palette:

```text
> Start focus
> Add task
> Add note
> Start break
> Open daily note
> Show statistics
```

---

# 10. Phase 7 — Plugin Architecture

**Target:** Future / Experimental

A plugin system should only be considered after the core application becomes stable.

Potential plugin capabilities:

```text
Plugin
├── UI widget
├── Commands
├── Events
├── Storage adapter
└── Settings
```

Potential plugins:

```text
Google Calendar
GitHub
Git
Notion
Todoist
Spotify
Weather
```

Plugins should be optional.

The core application must continue working without them.

---

# 11. Phase 8 — Integrations

Potential integrations based on community demand:

### Calendar

```text
Google Calendar
Outlook Calendar
iCal
```

### Development

```text
Git
GitHub
VS Code
Terminal
```

### Productivity

```text
Notion
Todoist
Linear
```

These should not be prioritized until the local-first core is stable.

---

# 12. Phase 9 — Cross-Platform

Windows is the primary platform.

Only consider broader support after Windows reaches a stable state.

Potential targets:

```text
Windows
   ↓
macOS
   ↓
Linux
```

Tauri provides a technical foundation for this, but platform-specific testing will still be required.

---

# 13. Phase 10 — Community Ecosystem

If the project develops an active community, future possibilities include:

- Community themes
- Community mascot packs
- Plugins
- Templates
- Documentation
- Contributor guides
- Community showcase

Potential GitHub structure:

```text
Issues
Discussions
Pull Requests
Releases
Wiki / Documentation
```

---

# 14. Possible v1.0 Vision

NFDesk v1.0 should not necessarily contain every possible feature.

Instead, v1.0 should represent a mature version of the original concept:

> **A lightweight desktop companion for focused work.**

Potential v1.0 capabilities:

```text
┌─────────────────────────────┐
│         NFDesk              │
│                             │
│       Animated Mascot       │
│                             │
│       24:32                 │
│       Focus                 │
│                             │
│   Current Task              │
│   Build landing page        │
│                             │
│   ☑ Research                │
│   ☑ Design                  │
│   □ Implement               │
│                             │
│   📝 Quick note...          │
│                             │
│   Today's Focus: 2h 35m     │
└─────────────────────────────┘
```

The application should remain small even as capabilities increase.

---

# 15. Features That Should NOT Automatically Be Added

A feature should not be added merely because another productivity application has it.

Avoid feature accumulation such as:

```text
Calendar
Email
Chat
AI
CRM
Project Management
Team Collaboration
Social Network
Cloud Drive
```

unless there is strong evidence that these features support the core NFDesk workflow.

---

# 16. Feature Prioritization

Future features should be evaluated using four questions:

### 1. Does it reduce friction?

Does the feature make focused work easier?

### 2. Does the developer actually need it?

Is the feature solving a real problem encountered while using NFDesk?

### 3. Does it preserve the local-first philosophy?

Does it introduce unnecessary cloud dependency?

### 4. Does it increase complexity disproportionately?

A small feature that dramatically increases maintenance cost should be carefully evaluated.

---

# 17. Feature Priority Levels

## P0 — Core

Must have.

```text
Timer
Task
Quick Note
Markdown
Window
Tray
Hotkey
```

## P1 — Important

Should be considered after MVP.

```text
Statistics
Themes
Better Obsidian integration
Keyboard-first UX
Custom timer presets
```

## P2 — Optional

Only implement based on demand.

```text
Calendar
GitHub
Notion
Custom mascots
Plugins
```

## P3 — Experimental

Ideas that require significant validation.

```text
AI assistant
Cloud sync
Marketplace
Social features
Team collaboration
```

---

# 18. GitHub Development Strategy

The project should be developed openly.

Suggested repository structure:

```text
nfdesk-widget/
│
├── src/
├── src-tauri/
├── public/
│
├── docs/
│   ├── PRD-MVP.md
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
│
├── README.md
├── CONTRIBUTING.md
├── LICENSE
└── CHANGELOG.md
```

---

# 19. Issue Strategy

GitHub Issues should be organized around concrete work.

Example:

```text
feat: add Pomodoro timer
feat: add quick note
feat: add system tray
fix: timer drift after sleep
fix: invalid vault path
refactor: isolate markdown provider
docs: improve installation guide
```

Avoid creating issues such as:

```text
"Make app better"
"Improve UI"
"Add more features"
```

Issues should describe a concrete problem or deliverable.

---

# 20. Community-Driven Development

Community feedback should influence future roadmap priorities.

Potential signals:

```text
GitHub Issues
GitHub Discussions
Feature Requests
Pull Requests
Usage feedback
Personal daily usage
```

Features should not be prioritized only because they sound interesting.

---

# 21. Success Metrics

The project should measure success primarily through usage rather than vanity metrics.

Early indicators:

- Developer uses NFDesk every workday
- User completes real focus sessions
- Quick notes are actually captured
- Markdown logs are useful
- Users report reduced context switching
- GitHub contributors appear
- Issues lead to meaningful improvements

GitHub stars are useful as a signal, but they should not be the primary product goal.

---

# 22. Long-Term Product Principle

NFDesk should follow this principle:

> **Small enough to stay open. Useful enough to keep open.**

The application should never lose its original identity by becoming an overloaded productivity suite.

---

# 23. Roadmap Decision Rule

Before adding a major feature:

```text
Is there a real problem?
        │
        ▼
Does NFDesk solve it naturally?
        │
        ▼
Does it strengthen the core workflow?
        │
        ▼
Can it remain local-first?
        │
        ▼
Can it be implemented without excessive complexity?
        │
        ▼
YES → Consider feature
NO  → Defer / Reject
```

---

# 24. Current Development Priority

At the beginning of the project, development should follow this order:

```text
1. Window
2. Timer
3. Task
4. Quick Note
5. Local Persistence
6. Markdown Integration
7. System Tray
8. Global Hotkey
9. Mascot
10. UI Polish
11. Testing
12. GitHub Release
```

Do not reverse this order by spending most development time on animations before the core workflow works.

---

# 25. Current Milestone

## NFDesk v0.1.0

The immediate objective is:

> **Build a small Windows widget that the developer can actually use every day for focused work.**

Everything else belongs to a later milestone.