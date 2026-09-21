# ticket-viewer: theme settings

The ticket-viewer's theme settings panel **MUST** conform to the canonical spec
[`viewer-api/theme-settings`](../viewer-api/theme-settings) — same layout, same 17 sections,
same controls, same persistence keys.

## Open/close contract

- Clicking the header `Theme settings` action MUST open a visible modal overlay, not just mount the panel in the DOM.
- The overlay MUST use `.modal-backdrop[role="dialog"][aria-label="Theme settings"]` with fixed positioning, a non-transparent backdrop tint, and a centered `.modal-panel.theme-settings-modal` container.
- The underlying page MUST remain visibly dimmed behind the modal while the theme settings panel is open.

## Viewer-specific overrides

_None._ The viewer conforms to the canonical defaults verbatim:

| Field | Value |
|---|---|
| GPU master toggle default | **ON** (`viewer-api-gpu-enabled` defaults to `"true"`) |
| Default `EffectSettings` | `DEFAULT_EFFECT_SETTINGS` (all effects on at default intensity) |
| Presets | inherited from `viewer-api`'s shared preset list |

## Rationale

The ticket-viewer is fully GPU-accelerated by default — the 3D graph view, glass
panels, animated background and particle effects are part of the visual identity of
the app. The master GPU toggle is provided so users can opt out of the overlay if
they prefer a lightweight, static presentation. The graph3d view continues to use
WebGPU regardless of the master toggle (it owns the canvas via `GPU_CANVAS_OWNER`).

## Implementation pointers

- Panel lives in `viewer-api/viewer-api/frontend/dioxus/src/components/theme_settings.rs`
  (shared component).
- Store: `ThemeStore` in `viewer-api/viewer-api/frontend/dioxus/src/store/theme.rs` with
  `gpu_enabled: true` default.
- The ticket-viewer Trunk shell in `memory-viewers/ticket-viewer/frontend/dioxus/index.html` MUST load the shared `modal.css` bundle in addition to `glass-panel.css` and `theme-settings.css`.
- Shared modal CSS lives in `viewer-api/viewer-api/frontend/dioxus/public/css/modal.css`.
- Canvas ownership arbitration with graph3d: `GPU_CANVAS_OWNER` thread-local in
  `viewer-api/viewer-api/frontend/dioxus/src/effects/mod.rs`.

## Validation

- `npm run test:e2e:release -- e2e-release/ticket-viewer.release.spec.ts -g "theme settings palette button opens and closes the theme settings panel"` in `memory-viewers/ticket-viewer/frontend/dioxus`
- The focused Playwright test MUST attach a screenshot of `.theme-settings.glass-panel` so the open-state is visually reviewable in addition to DOM assertions.
