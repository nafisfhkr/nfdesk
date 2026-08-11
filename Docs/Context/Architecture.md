# NFDesk Widget — Architecture

**Version:** 0.1.0  
**Platform:** Windows 10/11  
**Framework:** Tauri v2  
**Frontend:** React + TypeScript  
**Backend:** Rust

---

# 1. Architecture Goals

NFDesk architecture is designed around five principles:

1. Local-first
2. Modular
3. Lightweight
4. Easy to extend
5. Safe filesystem access

The architecture must allow future features to be added without rewriting the core application.

---

# 2. High-Level Architecture

```text
┌──────────────────────────────────────────┐
│                NFDesk UI                 │
│                                          │
│  Mascot │ Timer │ Tasks │ Quick Note     │
│                                          │
│              React + TypeScript          │
└─────────────────────┬────────────────────┘
                      │
                      │ Tauri Commands / Events
                      ▼
┌──────────────────────────────────────────┐
│               Tauri Runtime              │
│                                          │
│        Rust Application Layer            │
│                                          │
│ Window │ Tray │ Hotkey │ File System     │
└─────────────────────┬────────────────────┘
                      │
          ┌───────────┴────────────┐
          ▼                        ▼
┌──────────────────┐     ┌────────────────────┐
│ Local App State  │     │ Markdown Storage   │
│                  │     │                    │
│ Settings         │     │ Obsidian Vault     │
│ Tasks            │     │ Daily Notes        │
│ Timer State      │     │ Focus Logs         │
└──────────────────┘     └────────────────────┘
```

---

# 3. Technology Stack

| Layer | Technology | Purpose |
|---|---|---|
| Desktop Runtime | Tauri v2 | Native desktop application |
| Backend | Rust | OS integration and filesystem operations |
| Frontend | React | UI |
| Language | TypeScript | Type safety |
| Styling | Tailwind CSS | UI styling |
| Animation | Framer Motion | UI transitions |
| Mascot | DotLottie | Character animation |
| Icons | Lucide React | Interface icons |
| Storage | Local files | Application persistence |
| External Notes | Markdown | Obsidian integration |

---

# 4. Tauri Architecture

Tauri is responsible for native desktop capabilities.

Primary responsibilities:

- Window management
- Always-on-top
- System tray
- Global shortcuts
- Filesystem access
- Application lifecycle
- Native dialogs
- Application settings persistence where appropriate

React should not directly perform privileged filesystem operations.

---

# 5. Frontend Architecture

The frontend is responsible for presentation and user interaction.

Suggested structure:

```text
src/
├── components/
│   ├── mascot/
│   ├── timer/
│   ├── tasks/
│   ├── quick-note/
│   ├── settings/
│   └── common/
│
├── features/
│   ├── timer/
│   ├── tasks/
│   ├── notes/
│   └── settings/
│
├── hooks/
│
├── stores/
│
├── services/
│
├── types/
│
├── utils/
│
├── App.tsx
└── main.tsx
```

---

# 6. Feature-Based Organization

Features should be modular.

Example:

```text
features/
└── timer/
    ├── components/
    │   ├── TimerDisplay.tsx
    │   ├── TimerControls.tsx
    │   └── TaskInput.tsx
    │
    ├── timer.logic.ts
    ├── timer.types.ts
    └── timer.store.ts
```

This prevents all application logic from being placed inside large React components.

---

# 7. Application State

NFDesk has three major types of state.

## 7.1 UI State

Examples:

```text
active panel
settings open/closed
modal state
animation state
```

This state belongs to the frontend.

---

## 7.2 Runtime State

Examples:

```text
timer status
remaining seconds
current task
current session
```

This state belongs to the application runtime.

---

## 7.3 Persistent State

Examples:

```text
vault path
timer preferences
tasks
window position
theme
settings
```

Persistent state should survive application restarts.

---

# 8. Timer Architecture

The timer must not depend solely on React rendering.

Recommended model:

```text
Timer State
│
├── status
│   ├── idle
│   ├── running
│   ├── paused
│   └── completed
│
├── mode
│   ├── focus
│   └── break
│
├── startedAt
├── duration
├── remaining
└── currentTask
```

Prefer calculating elapsed time from timestamps instead of blindly decrementing a counter every second.

Example concept:

```text
remaining =
duration - (currentTime - startedAt)
```

This helps prevent timer drift.

---

# 9. Timer State Machine

```text
             ┌───────────┐
             │   IDLE    │
             └─────┬─────┘
                   │ Start
                   ▼
             ┌───────────┐
             │  RUNNING  │
             └─────┬─────┘
                   │ Pause
                   ▼
             ┌───────────┐
             │  PAUSED   │
             └─────┬─────┘
                   │ Resume
                   └──────────────┐
                                  ▼
                              RUNNING
                                  │
                                  │ Time = 0
                                  ▼
                            ┌───────────┐
                            │ COMPLETED │
                            └─────┬─────┘
                                  │
                                  ▼
                              BREAK / IDLE
```

---

# 10. Markdown Architecture

Markdown integration should be isolated from the rest of the application.

Do not allow React components to directly manipulate files.

Instead:

```text
QuickNote Component
       │
       ▼
Note Service
       │
       ▼
Tauri Command
       │
       ▼
Markdown Service
       │
       ▼
Daily Note
```

---

# 11. Markdown Adapter

Create an abstraction around external storage.

Concept:

```text
StorageProvider
│
├── appendNote()
├── appendFocusSession()
├── createDailyNote()
└── validatePath()
```

The MVP implementation:

```text
MarkdownStorageProvider
```

Future implementations may include:

```text
ObsidianProvider
NotionProvider
GitProvider
CloudProvider
```

However, only Markdown/local storage is required for MVP.

---

# 12. Daily Note Strategy

Default path:

```text
<VaultPath>/<YYYY-MM-DD>.md
```

Example:

```text
Obsidian Vault/
└── 2026-08-11.md
```

If the file does not exist:

```text
Create file
```

If it exists:

```text
Read → Validate → Append
```

---

# 13. Markdown Append Strategy

NFDesk should append content rather than rewriting the entire document whenever possible.

Example:

```markdown
# Daily Note

## Notes

- **14:30** — Investigate API error.

## Focus Sessions

- **14:00–14:25** — Build landing page generator
```

The exact section management can remain simple during MVP.

NFDesk must avoid destructive Markdown transformations.

---

# 14. Obsidian Compatibility

NFDesk should treat Obsidian as a Markdown environment rather than attempting to replicate Obsidian's internal architecture.

MVP supports:

- Local vault folder
- Markdown files
- Daily note
- Append operations

MVP does NOT attempt to understand:

- Dataview
- Obsidian plugins
- Complex task syntax
- Canvas
- Bases
- Graph relationships
- Metadata databases

---

# 15. Filesystem Security

Filesystem access should be centralized.

Recommended flow:

```text
React
  │
  ▼
Tauri Command
  │
  ▼
Validate requested path
  │
  ▼
Filesystem Service
  │
  ▼
Read / Write
```

Never allow arbitrary filesystem operations from unvalidated UI input.

---

# 16. Local Application Storage

NFDesk should maintain its own application state separately from the Obsidian vault.

Conceptual structure:

```text
NFDesk App Data
│
├── settings
│
├── tasks
│
├── timer
│
└── window state
```

User notes remain in:

```text
Obsidian Vault
```

This separation prevents NFDesk metadata from polluting the user's notes.

---

# 17. Settings Model

Example:

```json
{
  "alwaysOnTop": true,
  "theme": "dark",
  "focusDuration": 25,
  "shortBreakDuration": 5,
  "vaultPath": "C:\\Users\\User\\Documents\\Vault",
  "dailyNoteFormat": "YYYY-MM-DD.md"
}
```

The exact storage implementation may change.

---

# 18. Task Model

MVP task model:

```text
Task
├── id
├── title
├── completed
├── createdAt
└── completedAt
```

Example:

```json
{
  "id": "task-001",
  "title": "Implement upload API",
  "completed": false,
  "createdAt": "2026-08-11T08:00:00Z"
}
```

Tasks are internal NFDesk data in MVP.

They should not automatically modify arbitrary Obsidian task lists.

---

# 19. Mascot Architecture

The mascot should be decoupled from application logic.

The application provides a semantic state:

```text
idle
focus
break
success
```

The mascot component maps that state to an animation.

```text
Application State
       │
       ▼
Mascot State
       │
       ▼
Animation Asset
```

Example:

```text
focus → focus.lottie
break → break.lottie
idle  → idle.lottie
success → success.lottie
```

The mascot assets can therefore be replaced without changing timer logic.

---

# 20. Window Management

The desktop window should support:

- Frameless mode
- Transparent/rounded visual container
- Dragging
- Always-on-top
- Show/hide
- Position persistence
- Tray integration

Window management belongs to the Tauri layer.

---

# 21. System Tray

Tray actions:

```text
Show
Hide
Always on Top
Settings
Quit
```

The tray should allow NFDesk to continue running even when the main widget is hidden.

---

# 22. Global Hotkey

Global shortcut:

```text
Alt + Shift + N
```

Behavior:

```text
Widget hidden
    ↓
Hotkey
    ↓
Show widget

Widget visible
    ↓
Hotkey
    ↓
Hide widget
```

The implementation should be handled through Tauri's native/global shortcut capability.

---

# 23. Communication Between Frontend and Rust

Use explicit Tauri commands/events.

Conceptually:

```text
React
 │
 ├── invoke("save_note")
 ├── invoke("get_settings")
 ├── invoke("save_settings")
 └── invoke("append_focus_log")
 │
 ▼
Rust
 │
 ├── validate
 ├── execute
 └── return result
```

Avoid exposing unnecessary native functionality.

---

# 24. Error Handling

All filesystem operations must return explicit success/error states.

Example:

```text
Success
{
    ok: true
}
```

Error:

```text
Error
{
    ok: false,
    code: "VAULT_NOT_ACCESSIBLE",
    message: "The selected vault cannot be accessed."
}
```

Frontend should translate technical errors into understandable user messages.

---

# 25. Logging

The application should have development logs for:

- Timer errors
- Filesystem errors
- Settings errors
- Tauri command failures

Do not log:

- Private note contents
- Entire Markdown files
- Sensitive user information

---

# 26. Testing Strategy

## Unit Tests

Test:

- Timer calculations
- State transitions
- Markdown formatting
- Filename generation
- Settings validation

## Integration Tests

Test:

- Markdown creation
- Markdown append
- Invalid vault path
- Application persistence

## Manual Windows Tests

Test:

- Startup
- Tray
- Global hotkey
- Window dragging
- Always-on-top
- Sleep/wake behavior
- Multiple monitors
- Different display scaling

---

# 27. Architecture Rules

The following rules should be maintained:

### Rule 1

React components must not directly perform privileged filesystem operations.

### Rule 2

Timer logic must be independent from UI rendering.

### Rule 3

Markdown integration must be isolated behind a service/provider abstraction.

### Rule 4

Mascot animation must not control application logic.

### Rule 5

Future integrations must not require rewriting core productivity features.

### Rule 6

Do not introduce a database unless a real requirement appears.

### Rule 7

Prefer local files and simple data structures over unnecessary infrastructure.

---

# 28. Future Extensibility

The architecture should allow future additions such as:

```text
Calendar
     │
Statistics
     │
Custom Themes
     │
Custom Mascots
     │
Plugin System
     │
Additional Storage Providers
```

These should be added as modules rather than modifying the core timer/task logic.

---

# 29. Architecture Principle

> **Keep the core small. Put complexity at the edges.**

The core application should understand:

```text
Timer
Tasks
Notes
State
```

Integrations should understand:

```text
Obsidian
Filesystem
Calendar
Cloud
Plugins
```

This keeps NFDesk maintainable as the project grows.