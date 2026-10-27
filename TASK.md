# Rust Asteroids Clone — MVP Specification

## Objective
Implement a minimal playable Asteroids clone in Rust using the `macroquad` game framework.

## Tech Stack
- Language: Rust (edition 2021)
- Framework: macroquad 0.4
- Random: rand 0.8

## IN Scope (MVP)

### Player Ship
- Render as a triangle pointing up
- Controls:
  - Left/Right Arrow: rotate ship (3 radians/sec)
  - Up Arrow: thrust (accelerate in facing direction)
  - Space: shoot bullet
- Physics: position, velocity, facing angle; friction/drag minimal (space-like)
- Wrap around screen edges

### Asteroids
- Render as irregular polygons (6-8 vertices, jagged)
- Spawn at random screen edges, drift with random velocity
- 3 size classes: Large, Medium, Small
- Rotate slowly (random direction)
- Wrap around screen edges

### Bullets
- Shoot from ship nose when space pressed
- Small dots/circles, travel in fixed direction
- Lifetime: 1.5 seconds (then despawn)
- Maximum 4 bullets on screen at once

### Collision & Splitting
- Circle-based collision for all objects
- Bullet hits asteroid:
  - Large → 2 Medium
  - Medium → 2 Small
  - Small → destroyed (+score)
- Ship hits asteroid: game over (for MVP, just freeze/reset)

### Score
- Display top-left corner
- Large: 20 pts, Medium: 50 pts, Small: 100 pts

### Game Loop
- Window: 800x600, title "Asteroids MVP"
- Background: black
- Asteroids: start with 4 large ones
- When all asteroids destroyed: spawn 4 new large ones (infinite loop for MVP)

## OUT of Scope (Future Phases)
- Sound effects
- Particle explosions
- Ship lives/respawn
- UFO enemies
- High score table
- Menu screen
- Score pop-up animations
- Power-ups
- Screen shake

## Deliverable
Single file: `src/main.rs` containing all game code.

## Success Criteria
1. `cargo run` compiles and opens a window
2. Ship responds to controls
3. Bullets shoot and despawn after 1.5s
4. Asteroids spawn, drift, rotate
5. Hitting asteroid splits it (or destroys if small)
6. Score increments on destruction
7. Game loops infinitely (new wave when cleared)
