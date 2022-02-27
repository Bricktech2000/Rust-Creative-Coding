use nannou::{
    noise::{NoiseFn, Perlin, Seedable},
    prelude::*,
};
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::io::{stdout, Write};
use std::ops;

const POINT_COUNT: u32 = 64; // number of starting points
const POINT_DELTA: f64 = 0.1; // randomized position delta for each point
const HEADING_NOISE_FACTOR: f64 = 15.0; // multiplies noise input (higher makes more frequent changes in heading)
const HEADING_NOISE_MULTIPLIER: f64 = 1.0; // multiplies noise output (higher makes larger changes in heading)
const COLOR_NOISE_FACTOR: f64 = 1.0; // multiplies noise input (higher makes more frequent changes in color)
const COLOR_NOISE_MULTIPLIER: f64 = 1.1; // multiplies noise output (higher makes larger changes in color)
const VELOCITY_MULTIPLIER: f64 = 0.25; // multiplies velocity (higher makes faster but coarser)
const POINT_SIZE: f64 = 1.0; // size of rendered points (1.0 is one pixel)
const SEED: u64 = 0; // seed for random number generator and noise functions (set to 0 for random seed)

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Copy, Clone)]
struct Vector2 {
    x: f64,
    y: f64,
}

impl ops::Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Div<f64> for Vector2 {
    type Output = Vector2;

    fn div(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl ops::Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, other: f64) -> Vector2 {
        Vector2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl ops::AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, other: Vector2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Vector2 {
    fn length(&self) -> f64 {
        (f64::powf(self.x, 2.0) + f64::powf(self.y, 2.0)).sqrt()
    }

    fn normalize(&mut self) -> &mut Self {
        self.x /= self.length();
        self.y /= self.length();

        self
    }
}

struct Flow {
    pos: Vector2,
    vel: Vector2,
}

struct Model {
    _window: window::Id,
    flow_field: Vec<Vec<Flow>>,
    noise: Perlin,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();

    let x_end: f64 = app.window_rect().x.end as f64;
    let y_end: f64 = app.window_rect().y.end as f64;

    let seed = if SEED == 0 {
        std::time::SystemTime::now().elapsed().unwrap().as_nanos() as u64
    } else {
        SEED
    };

    let mut rng = StdRng::seed_from_u64(seed);
    let mut flow_field = Vec::new();
    let noise: Perlin = Perlin::new().set_seed(seed as u32);

    for y in 0..POINT_COUNT as usize {
        let mut row = Vec::new();
        for x in 0..POINT_COUNT as usize {
            let flow = Flow {
                pos: Vector2 {
                    x: x_end
                        * ((x as f64) / (POINT_COUNT as f64) + rng.gen_range(0.0..POINT_DELTA)),
                    y: y_end
                        * ((y as f64) / (POINT_COUNT as f64) + rng.gen_range(0.0..POINT_DELTA)),
                },
                vel: Vector2 { x: 0.0, y: 0.0 },
            };
            row.push(flow);
        }
        flow_field.push(row);
    }

    Model {
        _window,
        flow_field,
        noise,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Start the timer
    use std::time::Instant;
    let now = Instant::now();

    let x_end: f64 = app.window_rect().x.end as f64;
    let y_end: f64 = app.window_rect().y.end as f64;

    for row in model.flow_field.iter_mut() {
        for flow in row.iter_mut() {
            // Get the noise value at the flow's position.
            let mut noise_value = model.noise.get([
                flow.pos.x as f64 / x_end * HEADING_NOISE_FACTOR,
                flow.pos.y as f64 / y_end * HEADING_NOISE_FACTOR,
            ]);

            noise_value *= HEADING_NOISE_MULTIPLIER * 2.0 * std::f64::consts::PI;

            // Change the velocity based on the noise
            flow.vel = Vector2 {
                x: noise_value.sin(),
                y: noise_value.cos(),
            };

            flow.pos += flow.vel * VELOCITY_MULTIPLIER;
        }
    }

    let elapsed = now.elapsed();
    let message = format!("\rUpdate: {:.2?}", elapsed);

    print!("{}{}", message, " ".repeat(20 - message.chars().count()));
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Start the timer
    use std::time::Instant;
    let now = Instant::now();

    // Prepare to draw.
    let draw = app.draw();

    let x_end: f64 = app.window_rect().x.end as f64;
    let y_end: f64 = app.window_rect().y.end as f64;

    for row in model.flow_field.iter() {
        for flow in row.iter() {
            let x: f64 = (flow.pos.x * 2.0 - x_end) as f64; // map to window
            let y: f64 = (flow.pos.y * 2.0 - y_end) as f64;
            let x2: f64 = x / x_end * COLOR_NOISE_FACTOR; // 0 to 1, times COLOR_NOISE_FACTOR
            let y2: f64 = y / y_end * COLOR_NOISE_FACTOR;

            // Choose a color based on screen position
            let color = nannou::color::hsl(
                ((model.noise.get([x2, y2]) * COLOR_NOISE_MULTIPLIER + 1.0) / 2.0) as f32,
                1.0,
                0.5,
            );

            // Draw circle based on perlin noise
            draw.ellipse()
                .x_y(x as f32, y as f32)
                .w_h((1.0 * POINT_SIZE) as f32, (1.0 * POINT_SIZE) as f32)
                .color(color);
        }
    }

    draw.to_frame(app, &frame).unwrap();

    let elapsed = now.elapsed();
    print!("    Draw: {:.2?}", elapsed);
    stdout().flush().unwrap();
}
