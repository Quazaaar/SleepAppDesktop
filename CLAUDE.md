# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sleep App is a desktop application built with **Tauri 2** (Rust backend + React/TypeScript frontend). Currently at the starter template stage with a simple IPC greeting demo.

## Commands

### Development
```bash
npm run tauri dev        # Run full desktop app in dev mode (Rust + React)
npm run dev              # Start Vite dev server only (frontend at port 1420)
```

### Build
```bash
npm run build            # TypeScript check + Vite bundle (frontend only)
npm run tauri build      # Full production desktop build (runs npm build first)
```

### Preview
```bash
npm run preview          # Preview production frontend build
```

No test framework or linter is currently configured.

## Architecture

```
Frontend (React/TS)  ←→  Tauri IPC Bridge  ←→  Backend (Rust)
   src/                                          src-tauri/src/
```

- **Frontend entry:** `src/main.tsx` → `App.tsx`
- **Backend entry:** `src-tauri/src/main.rs` → `lib.rs`
- **IPC pattern:** React calls `invoke("command_name", args)` from `@tauri-apps/api/core`, which maps to Rust functions annotated with `#[tauri::command]`

### Adding new Tauri commands
1. Define the function in `src-tauri/src/lib.rs` with `#[tauri::command]`
2. Register it in the `invoke_handler` macro call in the `run()` function
3. Call it from React with `invoke("command_name", { arg: value })`

### Key configuration
- **Tauri config:** `src-tauri/tauri.conf.json` (window size, app ID, build commands, bundle targets)
- **Tauri capabilities/permissions:** `src-tauri/capabilities/default.json`
- **Vite config:** `vite.config.ts` (port 1420 is required by Tauri)
- **TypeScript:** strict mode enabled, target ES2020
