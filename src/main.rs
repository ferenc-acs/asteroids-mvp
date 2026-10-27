//! Asteroids MVP — a minimal clone built with macroquad 0.4.
//! Single-file implementation per TASK.md specification.

use ::rand::Rng; // `::rand` — macroquad's prelude re-exports its own `rand`
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const SCALE: f32 = 1.8;
const BASE_WINDOW_W: f32 = 800.0;
const BASE_WINDOW_H: f32 = 600.0;
const WINDOW_W: f32 = BASE_WINDOW_W * SCALE; // 1440
const WINDOW_H: f32 = BASE_WINDOW_H * SCALE; // 1080

const SHIP_RADIUS: f32 = 12.0 * SCALE; // circle-collision radius of the ship
const SHIP_ROTATION: f32 = 3.0; // radians per second (per spec)
const SHIP_THRUST: f32 = 320.0 * SCALE; // px/s^2 of acceleration
const SHIP_DRAG: f32 = 0.4; // fraction of velocity lost per second

const SHIP_NOSE_LEN: f32 = 15.0 * SCALE;
const SHIP_REAR_LEN: f32 = 11.0 * SCALE;
const SHIP_REAR_SPREAD: f32 = 2.5; // radians from the facing direction
const SHIP_FLAME_LEN: f32 = 25.0 * SCALE; // exhaust tip distance behind the ship

const BULLET_RADIUS: f32 = 2.0 * SCALE;
const BULLET_SPEED: f32 = 500.0 * SCALE;
const BULLET_LIFETIME: f32 = 1.5; // seconds
const MAX_BULLETS: usize = 4;

const LINE_THICKNESS: f32 = 2.0 * SCALE;

const WAVE_COUNT: usize = 4; // large asteroids per wave

const SCORE_FONT_SIZE: f32 = 36.0; // top-center score, slightly larger than before
const SCORE_TOP_MARGIN: f32 = 8.0 * SCALE;
const GAME_OVER_FONT_SIZE: f32 = 40.0 * SCALE;

// ---------------------------------------------------------------------------
// Asteroid sizes
// ---------------------------------------------------------------------------
/// Size classes; each carries its radius, score value and split behaviour.
#[derive(Clone, Copy, PartialEq)]
enum Size {
    Large,
    Medium,
    Small,
}

impl Size {
    fn radius(self) -> f32 {
        match self {
            Size::Large => 40.0 * SCALE,
            Size::Medium => 24.0 * SCALE,
            Size::Small => 13.0 * SCALE,
        }
    }

    fn score(self) -> i32 {
        match self {
            Size::Large => 20,
            Size::Medium => 50,
            Size::Small => 100,
        }
    }

    /// The size this one splits into; `None` means it is destroyed outright.
    fn split(self) -> Option<Size> {
        match self {
            Size::Large => Some(Size::Medium),
            Size::Medium => Some(Size::Small),
            Size::Small => None,
        }
    }

    /// Drift speed range: smaller asteroids move faster.
    fn speed_range(self) -> (f32, f32) {
        match self {
            Size::Large => (20.0 * SCALE, 60.0 * SCALE),
            Size::Medium => (50.0 * SCALE, 100.0 * SCALE),
            Size::Small => (80.0 * SCALE, 140.0 * SCALE),
        }
    }
}

// ---------------------------------------------------------------------------
// Ship
// ---------------------------------------------------------------------------
struct Ship {
    pos: Vec2,
    vel: Vec2,
    angle: f32, // 0 = right, -PI/2 = up (screen coords, y-down)
}

impl Ship {
    fn new() -> Self {
        Self {
            pos: Vec2::new(WINDOW_W * 0.5, WINDOW_H * 0.5),
            vel: Vec2::ZERO,
            angle: -std::f32::consts::FRAC_PI_2, // pointing up
        }
    }

    fn update(&mut self, dt: f32) {
        // Rotation
        if is_key_down(KeyCode::Left) {
            self.angle -= SHIP_ROTATION * dt;
        }
        if is_key_down(KeyCode::Right) {
            self.angle += SHIP_ROTATION * dt;
        }
        // Thrust along the facing direction
        if is_key_down(KeyCode::Up) {
            self.vel += Vec2::from_angle(self.angle) * SHIP_THRUST * dt;
        }
        // Light space-like drag + integrate
        self.vel *= 1.0 - SHIP_DRAG * dt;
        self.pos += self.vel * dt;
        wrap(&mut self.pos, SHIP_RADIUS);
    }

    fn draw(&self) {
        // Triangle: nose plus two rear corners
        let nose = self.pos + Vec2::from_angle(self.angle) * SHIP_NOSE_LEN;
        let rear_l = self.pos + Vec2::from_angle(self.angle + SHIP_REAR_SPREAD) * SHIP_REAR_LEN;
        let rear_r = self.pos + Vec2::from_angle(self.angle - SHIP_REAR_SPREAD) * SHIP_REAR_LEN;
        for (a, b) in [(nose, rear_l), (rear_l, rear_r), (rear_r, nose)] {
            draw_line(a.x, a.y, b.x, b.y, LINE_THICKNESS, WHITE);
        }
    }

    /// Engine exhaust indicator: an elongated filled triangle at the ship's
    /// rear, pointing opposite to the facing direction. Purely visual.
    fn draw_exhaust(&self) {
        let tip = self.pos + Vec2::from_angle(self.angle + PI) * SHIP_FLAME_LEN;
        let rear_l = self.pos + Vec2::from_angle(self.angle + SHIP_REAR_SPREAD) * SHIP_REAR_LEN;
        let rear_r = self.pos + Vec2::from_angle(self.angle - SHIP_REAR_SPREAD) * SHIP_REAR_LEN;
        draw_triangle(tip, rear_l, rear_r, ORANGE);
    }
}

// ---------------------------------------------------------------------------
// Bullet
// ---------------------------------------------------------------------------
struct Bullet {
    pos: Vec2,
    vel: Vec2,
    life: f32, // remaining seconds
}

impl Bullet {
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.life -= dt;
        wrap(&mut self.pos, BULLET_RADIUS);
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, BULLET_RADIUS, WHITE);
    }
}

// ---------------------------------------------------------------------------
// Asteroid
// ---------------------------------------------------------------------------
struct Asteroid {
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    rot_speed: f32,
    size: Size,
    verts: Vec<f32>, // per-vertex radius multipliers (jagged silhouette)
}

impl Asteroid {
    fn new(pos: Vec2, size: Size) -> Self {
        let mut rng = ::rand::thread_rng();
        let n = rng.gen_range(6..=8); // 6-8 vertices per spec
        let (lo, hi) = size.speed_range();
        let dir = rng.gen_range(0.0..TAU);
        Self {
            pos,
            vel: Vec2::new(dir.cos(), dir.sin()) * rng.gen_range(lo..hi),
            rot: rng.gen_range(0.0..TAU),
            rot_speed: rng.gen_range(-1.0..1.0), // slow, random direction
            size,
            verts: (0..n).map(|_| rng.gen_range(0.7..1.3)).collect(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.rot += self.rot_speed * dt;
        wrap(&mut self.pos, self.size.radius());
    }

    /// Draw as a closed polygon of lines (macroquad's `draw_poly` is regular).
    fn draw(&self) {
        let n = self.verts.len();
        let r = self.size.radius();
        let pts: Vec<Vec2> = (0..n)
            .map(|i| {
                let a = self.rot + i as f32 * TAU / n as f32;
                self.pos + Vec2::new(a.cos(), a.sin()) * (r * self.verts[i])
            })
            .collect();
        for i in 0..n {
            let a = pts[i];
            let b = pts[(i + 1) % n];
            draw_line(a.x, a.y, b.x, b.y, LINE_THICKNESS, WHITE);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
/// Teleport `pos` to the opposite edge once it moves past the current one.
fn wrap(pos: &mut Vec2, margin: f32) {
    if pos.x < -margin {
        pos.x = WINDOW_W + margin;
    }
    if pos.x > WINDOW_W + margin {
        pos.x = -margin;
    }
    if pos.y < -margin {
        pos.y = WINDOW_H + margin;
    }
    if pos.y > WINDOW_H + margin {
        pos.y = -margin;
    }
}

/// A random point on one of the four screen edges (asteroid spawn point).
fn edge_pos() -> Vec2 {
    let mut rng = ::rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => Vec2::new(rng.gen_range(0.0..WINDOW_W), 0.0), // top
        1 => Vec2::new(rng.gen_range(0.0..WINDOW_W), WINDOW_H), // bottom
        2 => Vec2::new(0.0, rng.gen_range(0.0..WINDOW_H)), // left
        _ => Vec2::new(WINDOW_W, rng.gen_range(0.0..WINDOW_H)), // right
    }
}

/// Start a fresh wave of large asteroids at the screen edges.
fn spawn_wave(asteroids: &mut Vec<Asteroid>) {
    for _ in 0..WAVE_COUNT {
        asteroids.push(Asteroid::new(edge_pos(), Size::Large));
    }
}

/// Circle collision test between two positions with the given radii.
fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    a.distance_squared(b) < (ra + rb).powi(2)
}

// ---------------------------------------------------------------------------
// Window config
// ---------------------------------------------------------------------------
fn conf() -> Conf {
    Conf {
        window_title: "Asteroids MVP".to_owned(),
        window_width: WINDOW_W as i32,
        window_height: WINDOW_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Game loop
// ---------------------------------------------------------------------------
#[macroquad::main(conf)]
async fn main() {
    let mut score = 0;
    let mut ship = Ship::new();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut game_over = false;
    spawn_wave(&mut asteroids);

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        if game_over {
            // Frozen world: keep drawing, overlay the message.
            if is_key_pressed(KeyCode::R) {
                score = 0;
                ship = Ship::new();
                bullets.clear();
                asteroids.clear();
                spawn_wave(&mut asteroids);
                game_over = false;
            }
        } else {
            // --- Ship input & physics ---
            ship.update(dt);

            // Shooting: one bullet per press, capped at MAX_BULLETS on screen.
            if is_key_pressed(KeyCode::Space) && bullets.len() < MAX_BULLETS {
                let nose = ship.pos + Vec2::from_angle(ship.angle) * SHIP_NOSE_LEN;
                bullets.push(Bullet {
                    pos: nose,
                    vel: Vec2::from_angle(ship.angle) * BULLET_SPEED,
                    life: BULLET_LIFETIME,
                });
            }

            // --- Bullets: move, expire ---
            bullets.retain_mut(|b| {
                b.update(dt);
                b.life > 0.0
            });

            // --- Asteroids: move & rotate ---
            for a in asteroids.iter_mut() {
                a.update(dt);
            }

            // --- Bullet vs asteroid collisions (circle-based) ---
            let mut bi = 0;
            while bi < bullets.len() {
                let hit = asteroids.iter().position(|a| {
                    circles_overlap(bullets[bi].pos, BULLET_RADIUS, a.pos, a.size.radius())
                });
                match hit {
                    Some(ai) => {
                        let b = bullets.remove(bi);
                        let a = asteroids.remove(ai);
                        score += a.size.score();
                        // Split: Large -> 2 Medium, Medium -> 2 Small, Small -> gone.
                        if let Some(child) = a.size.split() {
                            for _ in 0..2 {
                                asteroids.push(Asteroid::new(a.pos, child));
                            }
                        }
                        let _ = b; // bullet consumed by the impact
                    }
                    None => bi += 1,
                }
            }

            // --- Ship vs asteroid collision: game over ---
            if asteroids
                .iter()
                .any(|a| circles_overlap(ship.pos, SHIP_RADIUS, a.pos, a.size.radius()))
            {
                game_over = true;
            }

            // --- New wave when the field is cleared ---
            if asteroids.is_empty() {
                spawn_wave(&mut asteroids);
            }
        }

        // --- Rendering ---
        if !game_over && is_key_down(KeyCode::Up) {
            ship.draw_exhaust();
        }
        ship.draw();
        for b in &bullets {
            b.draw();
        }
        for a in &asteroids {
            a.draw();
        }
        let score_text = format!("SCORE: {score:05}");
        let dims = measure_text(&score_text, None, SCORE_FONT_SIZE as u16, 1.0);
        draw_text(
            &score_text,
            (WINDOW_W - dims.width) * 0.5,
            SCORE_TOP_MARGIN + dims.offset_y,
            SCORE_FONT_SIZE,
            WHITE,
        );

        if game_over {
            let msg = "GAME OVER — press R to restart";
            let dims = measure_text(msg, None, GAME_OVER_FONT_SIZE as u16, 1.0);
            draw_text(
                msg,
                (WINDOW_W - dims.width) * 0.5,
                (WINDOW_H - dims.height) * 0.5,
                GAME_OVER_FONT_SIZE,
                RED,
            );
        }

        next_frame().await;
    }
}
