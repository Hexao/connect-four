use iced::{Point, Vector};
use std::time::Instant;

#[repr(packed)]
pub struct Animation {
    start: Point,
    direction: Vector,

    control_point1: Point,
    control_point2: Point,

    start_animation: Instant,
    animation_duration: f32,
}

impl Animation {
    pub fn update_axis(&mut self, start: Point, direction: Vector) {
        self.direction = direction;
        self.start = start;
    }

    pub fn update_ctrl(&mut self, ctrl_point1: Point, ctrl_point2: Point) {
        self.control_point1 = ctrl_point1;
        self.control_point2 = ctrl_point2;
    }

    pub fn update_duration(&mut self, duration: f32) {
        self.animation_duration = duration;
    }

    pub fn restart(&mut self) {
        self.start_animation = Instant::now();
    }

    pub fn point_at(&self, time: Instant) -> Point {
        let elapsed = time.duration_since(self.start_animation).as_secs_f32();

        if elapsed <= 0.0 { return self.start; }
        if elapsed >= self.animation_duration { return self.start + self.direction; }

        let x = elapsed / self.animation_duration;
        let t = self.compute_t(x);
        let y = self.compute_y(t);

        self.start + self.direction * y
    }

    pub fn finished_at(&self, time: Instant) -> bool {
        let elapsed = time.duration_since(self.start_animation).as_secs_f32();

        elapsed > self.animation_duration
    }

    pub fn point_with(&self, x: f32) -> Point {
        if x <= 0.0 { return self.start; }
        if x >= 1.0 { return self.start + self.direction; }

        let t = self.compute_t(x);
        let y = self.compute_y(t);

        self.start + self.direction * y
    }

    pub fn start_point(&self) -> Point {
        self.start
    }

    fn compute_t(&self, x: f32) -> f32 {
        let Point { x: x1, .. } = self.control_point1;
        let Point { x: x2, .. } = self.control_point2;

        let a = 3.0*x1 - 3.0*x2 + 1.0;
        let b = 3.0*x2 - 6.0*x1;
        let c = 3.0*x1;
        let d = -x;

        let p = (3.0*a*c - b*b) / (3.0*a*a);
        let q = (2.0*b*b*b - 9.0*a*b*c + 27.0*a*a*d) / (27.0*a*a*a);
        let det = -4.0*p*p*p - 27.0*q*q;

        let z = match det.total_cmp(&0.0) {
            std::cmp::Ordering::Greater => {
                2.0 * (-p/3.0).sqrt() * ((1.0/3.0) * (((3.0*q)/(2.0*p)) * (-3.0/p).sqrt()).acos()).cos()
            },
            std::cmp::Ordering::Less => {
                let u = (-q + (-det / 27.0).sqrt()) / 2.0;
                let v = (-q - (-det / 27.0).sqrt()) / 2.0;

                u.cbrt() + v.cbrt()
            },
            std::cmp::Ordering::Equal => todo!(),
        };

        z - b / (3.0*a)
    }

    fn compute_y(&self, t: f32) -> f32 {
        let Point { y: y1, .. } = self.control_point1;
        let Point { y: y2, .. } = self.control_point2;

        let a = 3.0*y1 - 3.0*y2 + 1.0;
        let b = 3.0*y2 - 6.0*y1;
        let c = 3.0*y1;

        t*t*t*a + t*t*b + t*c
    }
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            start: Point::ORIGIN,
            direction: Vector::new(1.0, 0.0),

            control_point1: Point::new(0.5, 0.0),
            control_point2: Point::new(0.5, 1.0),

            start_animation: Instant::now(),
            animation_duration: 1.0
        }
    }
}

pub struct Builder {
    animation: Animation,
}

impl Builder {
    pub fn new() -> Self {
        Self { animation: Animation::default() }
    }

    pub fn move_curve(mut self, start: Point, direction: Vector) -> Self {
        self.animation.update_axis(start, direction);
        self
    }

    pub fn ctrl_curve(mut self, ctrl_point1: Point, ctrl_point2: Point) -> Self {
        self.animation.update_ctrl(ctrl_point1, ctrl_point2);
        self
    }

    pub fn anim_duration(mut self, duration: f32) -> Self {
        self.animation.update_duration(duration);
        self
    }

    pub fn build(self) -> Animation {
        // self.animation.start_animation = Instant::now();
        self.animation
    }
}
