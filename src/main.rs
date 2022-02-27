use nannou::{
    noise::{NoiseFn, Perlin, Seedable},
    prelude::*,
};
use rand::Rng;
use std::ops;

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

    let mut init_flow_field = Vec::new();

    let size: f64 = 80.0;

    let x_end: f64 = app.window_rect().x.end as f64;
    let y_end: f64 = app.window_rect().y.end as f64;

    for y in 0..size as usize {
        let mut row = Vec::new();
        for x in 0..size as usize {
            let flow = Flow {
                pos: Vector2 {
                    x: x_end * ((x as f64) / size + rand::thread_rng().gen_range(0.0..0.1)),
                    y: y_end * ((y as f64) / size + rand::thread_rng().gen_range(0.0..0.1)),
                },
                vel: Vector2 { x: 0.0, y: 0.0 },
            };
            row.push(flow);
        }
        init_flow_field.push(row);
    }

    Model {
        _window,
        flow_field: init_flow_field,
        noise: Perlin::new()
            .set_seed(std::time::SystemTime::now().elapsed().unwrap().as_nanos() as u32),
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
                flow.pos.x as f64 / x_end * 5.0 * 2.0,
                flow.pos.y as f64 / y_end * 5.0 * 2.0,
            ]);

            noise_value *= 2.0 * 2.0 * std::f64::consts::PI;

            // Change the velocity based on the noise
            flow.vel = Vector2 {
                x: noise_value.sin(),
                y: noise_value.cos(),
            };

            flow.pos += flow.vel / 2.0 / 2.0;
        }
    }

    let elapsed = now.elapsed();
    println!("Update: {:.2?}", elapsed);
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
            let x: f64 = (flow.pos.x * 2.0 - x_end) as f64;
            let y: f64 = (flow.pos.y * 2.0 - y_end) as f64;
            let x2: f64 = x / x_end * 2.0 * 2.0;
            let y2: f64 = y / y_end * 2.0 * 2.0;

            // Choose a color based on screen position
            // let color = nannou::color::rgb(x % 100.0 + 30.0, y % 100.0 + 30.0, x % 100.0 + 30.0);

            let color = nannou::color::hsl(
                ((model.noise.get([x2, y2]) * 1.1 + 1.0) / 2.0) as f32,
                1.0,
                0.5,
            );

            // Draw circle based on perlin noise
            draw.ellipse()
                .x_y(x as f32, y as f32)
                .w_h(1.0 / 2.0, 1.0 / 2.0)
                .color(color);
        }
    }

    draw.to_frame(app, &frame).unwrap();

    let elapsed = now.elapsed();
    println!("Draw: {:.2?}", elapsed);
}
