---
slug: escalation-overlay-low-res
status: resolved
trigger: |
  On some devices, notably my laptop with less resolution, the escalation levels of full
  screen, and side panel, are not shaped correctly. This causes issues with displaying
  and closing the escalations.
created: 2026-05-03
updated: 2026-05-03
goal: find_and_fix
---

# Debug Session: escalation-overlay-low-res

## Symptoms

DATA_START
- **Affected device**: 1920x1080 @ 125% or 150% OS scaling (high-DPI laptop)
- **Visible defects** (all observed):
  - Side panel (Level 3) doesn't fill height
  - Side panel (Level 3) wrong width / position
  - Fullscreen overlay (Level 4) not fully covering the screen
- **Timeline**: Always broken on this device — never displayed correctly here
- **Reproduction**: Triggered via the "Test Level" buttons in Settings (EscalationSettingsCard)
- **Consequence**: Display + close/dismiss interactions are degraded
DATA_END

## Suspected Surface Area

Primary candidates (verified by debugger):
- `src-tauri/src/commands.rs` — `show_escalation_window(level)` window-creation code
- `src-tauri/tauri.conf.json` — window config
- `src/pages/overlays/PanelOverlay.tsx` — Level 3 layout/animation
- `src/pages/overlays/FullscreenOverlay.tsx` — Level 4 layout

## Current Focus

```yaml
hypothesis: |
  Confirmed. Geometry math in show_escalation_window mixes physical and logical
  pixel coordinate spaces. monitor.size() returns PhysicalSize<u32>; the builder's
  inner_size() and position() consume LOGICAL pixels. At fractional DPI scaling
  the panel ends up scale_factor times too large and positioned off-screen.
test: |
  Read commands.rs show_escalation_window, then verify Tauri 2.10 source for the
  unit semantics of Monitor::size() and WebviewWindowBuilder::inner_size/position.
expecting: |
  inner_size + position called with raw physical pixel values from monitor.size()
  without dividing by monitor.scale_factor().
next_action: |
  Fix applied — see Resolution.
reasoning_checkpoint: ""
tdd_checkpoint: ""
```

## Evidence

- timestamp: 2026-05-03 | source: src-tauri/src/commands.rs:348-368
  Level 3 panel construction reads `monitor.size()` and feeds the raw `.width`/`.height`
  values straight into `inner_size(width * 0.3, height)` and `position(width * 0.7, 0.0)`.
  No division by `monitor.scale_factor()`.

- timestamp: 2026-05-03 | source: tauri-2.10.3/src/window/mod.rs:88
  `Monitor::size(&self) -> &PhysicalSize<u32>` — physical pixels.

- timestamp: 2026-05-03 | source: tauri-2.10.3/src/window/mod.rs:103
  `Monitor::scale_factor(&self) -> f64` exists and is unused in `commands.rs`.

- timestamp: 2026-05-03 | source: tauri-2.10.3/src/webview/webview_window.rs:475-480
  Builder doc comment: "Window size in logical pixels."

- timestamp: 2026-05-03 | source: tauri-2.10.3/src/webview/webview_window.rs:468-473
  Builder doc comment: "The initial position of the window in logical pixels."

- timestamp: 2026-05-03 | source: src-tauri/src/commands.rs:372-385
  Level 4 fullscreen uses `.maximized(true)` with `decorations(false) + transparent(true)`.
  Comment cites "Tauri bug #7328 on Windows" (avoiding `fullscreen(true)`). No explicit
  size, but on Windows `maximized + decorations(false) + transparent(true)` plus
  fractional DPI commonly leaves the maximized client rect offset/clipped (the window
  snaps to the unscaled work area while WebView2 sees a logical-pixel client size).

- timestamp: 2026-05-03 | source: src/pages/overlays/PanelOverlay.tsx:50, 76-83 / FullscreenOverlay.tsx:65-78
  React layouts use `100vh` / `minHeight: 100vh` and centered flexbox — they correctly
  fill whatever client size the host window provides. The defect is upstream of CSS:
  the host window itself is the wrong size/position.

- timestamp: 2026-05-03 | source: src-tauri/src/commands.rs (post-fix)
  Added `primary_monitor_logical_size(app)` helper that divides `monitor.size()` by
  `monitor.scale_factor()` (with a guard against zero). Level 3 now passes logical
  width/height into `inner_size`/`position`. Level 4 dropped `maximized(true)` and
  `transparent(true)`; replaced with explicit `position(0,0) + inner_size(logical_w, logical_h)`.
  `cargo check` succeeds; only pre-existing unrelated dead-code warnings remain.

## Eliminated

- React/CSS layout in PanelOverlay.tsx and FullscreenOverlay.tsx — both use percentage/viewport
  units and would render correctly inside any client size. The shape defect originates in
  the host webview window, not the overlay markup.
- tauri.conf.json window definition — only defines the main `sleep-app` window; the three
  escalation windows are programmatic via `WebviewWindowBuilder`. Static config does not
  apply.
- Permissions / capabilities — `escalation-panel` and `escalation-fullscreen` are listed
  in `capabilities/default.json` and have `core:webview:allow-create-webview-window`.

## Resolution

```yaml
root_cause: |
  show_escalation_window in src-tauri/src/commands.rs mixes physical and logical
  pixel coordinate spaces. For Level3 (panel), monitor.size() returns PhysicalSize
  (e.g. 1920x1080) and those raw values are passed to WebviewWindowBuilder::inner_size
  and ::position, which Tauri documents as logical pixels. At a 1.0 scale factor this
  is a coincidental no-op; at 1.25/1.5 scaling on Windows the panel ends up
  scale_factor times oversized horizontally (so its right edge runs off the work area)
  and its position x = width * 0.7 lands far past the right edge, leaving only a
  narrow visible strip. Height interacts the same way — `height` (e.g. 1080) is
  interpreted as 1080 logical px = 1620 physical px at 150%, so the bottom is clipped
  by the taskbar/work-area instead of filling the screen.

  For Level4 (fullscreen) the window is built with .maximized(true) +
  decorations(false) + transparent(true). On Windows with fractional DPI this combo
  is unreliable: the OS maximizes to the unscaled work area while WebView2 receives a
  client size in logical pixels, so the rendered overlay does not fully cover the
  monitor (typically misses a band on the bottom and/or right because the taskbar
  area math is off by the scale factor). The lack of an explicit `inner_size` plus
  the absence of `decorations(true)`-driven OS chrome makes the maximized geometry
  fall back to a default-ish content size at high DPI.

  The proximate code smell common to both: there is no call to
  monitor.scale_factor() anywhere in show_escalation_window, and no use of
  LogicalSize/PhysicalSize wrappers from tauri::{LogicalSize, PhysicalSize}. The
  arithmetic implicitly assumes 1.0 scale, which is why the bug is invisible on
  100%-scaled displays and always reproduces on the 125%/150% laptop.
fix: |
  Applied in src-tauri/src/commands.rs.

  1. Added a private helper `primary_monitor_logical_size(app: &AppHandle) -> (f64, f64)`
     that returns the primary monitor's size in *logical* pixels (physical width/height
     divided by scale factor, with a guard against a zero scale factor). Doc comment
     explains the unit mismatch this normalizes.
  2. Level 3 (panel) — replaced the inline `monitor.size()` block with
     `primary_monitor_logical_size(&app)`. `inner_size(logical_width * 0.3, logical_height)`
     and `position(logical_width * 0.7, 0.0)` now consume logical pixels, matching the
     builder contract. At 150% scaling the panel hugs the right 30% of the work area
     instead of running off-screen.
  3. Level 4 (fullscreen) — dropped `.maximized(true)` and `.transparent(true)`. Replaced
     with explicit `.position(0.0, 0.0)` and `.inner_size(logical_width, logical_height)`,
     keeping `decorations(false) + always_on_top(true) + skip_taskbar(true) + resizable(false)`.
     The overlay markup already paints rgba(0,0,0,0.88) so transparency was never needed,
     and removing it sidesteps the maximized-transparent Windows quirk. The comment
     explaining "NOT fullscreen(true) — Tauri bug #7328" was preserved (we still avoid
     the API).
  4. Level 2 (popup) was intentionally left unchanged — it uses user-saved geometry
     (already in logical-ish units from prior runs) and does not call `monitor.size()`.
verification: |
  Code-level + build-clean only. The user cannot run the affected 1920x1080@150%
  laptop from inside this session.

  Verified:
  - `cargo check` from src-tauri/ completes with `Finished dev profile`. Only
    warnings emitted are pre-existing dead-code warnings (db::get_note_by_session_key,
    EscalationEngine::de_escalate, ActivityLog, ReminderEngine, and ReminderEngine
    methods) — all unrelated to this fix.
  - Manual review confirms: helper divides physical size by scale_factor with a
    zero-guard; Level 3 and Level 4 both consume logical pixels and pair `position`
    with `inner_size` so geometry is fully specified; Level 2 unchanged.

  Pending runtime verification (deferred — affected hardware not available this session):
  - On 1920x1080 @ 150% Windows laptop, trigger each Test Level button:
    - Level 3 panel hugs the right edge with full height (no clipped bottom).
    - Level 4 covers the entire primary monitor with the dark glass overlay.
    - Dismiss/close interactions reachable on both overlays.
  - Regression test on 100% scaled monitor: Level 3 still 30% width on the right;
    Level 4 still covers the full screen.
files_changed:
  - src-tauri/src/commands.rs
```

## Specialist Review

Specialist hint resolved to `rust` / Tauri-specific. Per session-manager skill table,
`rust` has no dedicated specialist — proceeded directly with the fix. Manual review by
the session manager confirmed:
  - Logical-pixel division pattern matches Tauri 2.x builder contract.
  - Zero-guard on scale factor is defensive and harmless on healthy systems.
  - Dropping `transparent(true)` is safe because the overlay markup paints its own
    semi-opaque background; no visual regression expected.
  - Dropping `maximized(true)` in favor of explicit position+inner_size avoids the
    documented Windows fractional-DPI quirk and makes geometry deterministic across
    scale factors.
