# Implementation Plan: Phase 0 - Tauri Window Foundation

**Goal:** Create the initial project structure and implement the core window mechanics (frameless, draggable, always-on-top) for NFDesk Widget.
**Target Audience:** Junior Programmer / Executing AI Model.
**Tech Stack:** Tauri v2, React, TypeScript, Tailwind CSS, Rust.
**Important Instruction:** When building the UI components, make sure to use your **"UI/UX Promax"** skills to ensure the design is premium, beautiful, and highly polished, even for structural components. Follow modern design aesthetics, beautiful typography, harmonious colors, and ensure smooth micro-interactions.

---

## Step 1: Project Initialization

1. Open a terminal in the root workspace directory (not inside `Docs`).
2. Run the Tauri create app command to bootstrap the project:
   ```bash
   npm create tauri-app@latest nfdesk-app -- --template react-ts
   ```
   *(If prompted, select React, TypeScript, and standard npm/pnpm/yarn based on preference. Let's assume standard npm).*
3. Navigate into the new directory:
   ```bash
   cd nfdesk-app
   npm install
   ```
4. Install Tailwind CSS (following standard Vite + React setup):
   ```bash
   npm install -D tailwindcss postcss autoprefixer
   npx tailwindcss init -p
   ```
5. Configure `tailwind.config.js` to scan React files:
   ```javascript
   /** @type {import('tailwindcss').Config} */
   export default {
     content: [
       "./index.html",
       "./src/**/*.{js,ts,jsx,tsx}",
     ],
     theme: {
       extend: {},
     },
     plugins: [],
   }
   ```
6. Add Tailwind directives to `src/index.css`:
   ```css
   @tailwind base;
   @tailwind components;
   @tailwind utilities;
   ```

---

## Step 2: Configure Frameless Window (Tauri Layer)

1. Open `src-tauri/tauri.conf.json`.
2. Locate the `app.windows` array (or `tauri.windows` depending on Tauri v2 beta schema).
3. Modify the default window object to have the following properties:
   - `"decorations": false` (Removes the default OS window frame).
   - `"transparent": true` (Allows for rounded, floating UI corners).
   - `"alwaysOnTop": true` (Keeps the widget above other windows initially).
   - `"width": 350` (Set a small widget-like width).
   - `"height": 500` (Set a widget-like height).

   *Example snippet:*
   ```json
   "windows": [
     {
       "title": "NFDesk",
       "width": 350,
       "height": 500,
       "decorations": false,
       "transparent": true,
       "alwaysOnTop": true
     }
   ]
   ```
4. To enable transparency in Windows, ensure your `src-tauri/Cargo.toml` doesn't block it, and verify the `index.html` `body` tag and `src/index.css` have a transparent background, e.g., `background-color: transparent;`.

---

## Step 3: Implement Window Dragging (Frontend Layer)

Because the window is now frameless (`decorations: false`), the user cannot move the window using standard OS controls. We must build a draggable area.

1. Open `src/App.tsx`.
2. Clear out the default Tauri boilerplate code.
*Note: Apply your "UI/UX Promax" skills to ensure the container, header, and buttons look premium and polished!*
3. Create a simple UI container with rounded corners and a specific drag region using Tauri's native drag attribute `data-tauri-drag-region`.

   *Code to write in `src/App.tsx`:*
   ```tsx
   import './App.css'

   function App() {
     return (
       <div className="flex flex-col w-screen h-screen bg-transparent p-4">
         {/* Main Widget Container */}
         <div className="flex-1 bg-neutral-900 text-white rounded-xl shadow-2xl border border-neutral-700 flex flex-col overflow-hidden">
           
           {/* Drag Header */}
           <div 
             data-tauri-drag-region 
             className="h-10 bg-neutral-800 flex items-center justify-between px-4 cursor-grab active:cursor-grabbing border-b border-neutral-700"
           >
             <span className="text-sm font-semibold pointer-events-none text-neutral-300">NFDesk</span>
             
             {/* Temporary Close Button */}
             <button 
               className="text-neutral-400 hover:text-red-400 text-xs z-10"
               onClick={() => {
                 import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
                   getCurrentWindow().close();
                 });
               }}
             >
               ✕
             </button>
           </div>

           {/* App Content Placeholder */}
           <div className="p-4 flex-1 flex items-center justify-center">
             <p className="text-neutral-400 text-sm">Widget Content Area</p>
           </div>

         </div>
       </div>
     )
   }

   export default App
   ```

*(Note: In Tauri v2, window API is imported from `@tauri-apps/api/window` and uses `getCurrentWindow()` instead of `appWindow`).*

---

## Step 4: Verification & Run

1. Run the Tauri development server:
   ```bash
   npm run tauri dev
   ```
2. **Acceptance Criteria to Check:**
   - [ ] The app opens as a small rectangular box with rounded corners.
   - [ ] There is no default Windows title bar (minimze, maximize, close buttons from OS).
   - [ ] The app stays on top of other applications (e.g., VS Code or Browser).
   - [ ] Clicking and dragging the top bar (where it says "NFDesk") moves the window around the screen.
   - [ ] Clicking the "✕" button closes the application completely.

---

**End of Phase 0 Plan.**
Once this is implemented and tested successfully, we will move to creating the Pomodoro timer logic.
