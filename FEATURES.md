# Feature Requests — Asteroids MVP Polish

## 1. Window Size: 1.8x (not double)
Current: 800 x 600
New: 1440 x 1080 (800*1.8=1440, 600*1.8=1080)

All object sizes, positions, speeds scale with the window.
Keep 800x600 as named constants derived from a base multiplier.

## 2. Score Counter: Top-Center (Classic Arcade Style)
- Position: horizontally centered at the top of the screen
- Format: "SCORE: 00000" (right-aligned within a frame, or just centered text)
- Style: White text, default macroquad font, slightly larger than current font size
- Keep existing scoring logic (Large:20, Medium:50, Small:100)

## 3. Engine Exhaust Indicator (Thrust Visual Feedback)
When UP key is pressed (thrust active):
- Show a small triangle at the rear of the ship, pointing opposite to facing direction
- Shape: elongated/pointy triangle, small, attached to the ship's rear
- Style options (pick simplest that looks good with macroquad):
  a) White wireframe triangle, flashing on/off every 0.08s (classic arcade style)
  b) Solid filled triangle in ORANGE/RED (later arcade hardware style)
- The indicator should NOT affect collision; purely visual
- When UP key is released, indicator disappears immediately

## Constraints
- Single file: src/main.rs (update existing, preserve all working logic)
- All three features must be implemented
- Re-verify with `cargo check`
