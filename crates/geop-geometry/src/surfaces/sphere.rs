use crate::{points::point::Point, EQ_THRESHOLD};

use super::surface::Surface;

#[derive(Clone, Debug)]
pub struct Sphere {
    pub basis: Point,
    pub radius: f64,
}

impl Sphere {
    pub fn new(basis: Point, radius: f64) -> Sphere {
        Sphere {
            basis,
            radius,
        }
    }
}

impl Surface for Sphere {
    fn point_at(&self, u: f64, v: f64) -> Point {
        let x = self.basis.x + self.radius * u.cos() * v.sin();
        let y = self.basis.y + self.radius * u.sin() * v.sin();
        let z = self.basis.z + self.radius * v.cos();
        Point::new(x, y, z)
    }

    fn project(&self, p: &Point) -> (f64, f64) {
        let v = *p - self.basis;
        let u = v.dot(Point::new(1.0, 0.0, 0.0));
        let v = v.dot(Point::new(0.0, 1.0, 0.0));
        (u, v)
    }

    fn derivative_u(&self, u: f64, v: f64) -> Point {
        let x = -self.radius * u.sin() * v.sin();
        let y = self.radius * u.cos() * v.sin();
        let z = 0.0;
        Point::new(x, y, z)
    }

    fn derivative_v(&self, u: f64, v: f64) -> Point {
        let x = self.radius * u.cos() * v.cos();
        let y = self.radius * u.sin() * v.cos();
        let z = -self.radius * v.sin();
        Point::new(x, y, z)
    }

    fn normal(&self, p: Point) -> Point {
        (p - self.basis).normalize()
    }

    fn normalize(&mut self) {
        // Use this to make redundant representations of surfaces unique
        self.radius = self.radius.abs();
    }

    fn is_normalized(&self) -> bool {
        self.radius >= 0.0
    }
}

impl PartialEq for Sphere {
    fn eq(&self, other: &Sphere) -> bool {
        self.basis == other.basis && (self.radius - other.radius).abs() < EQ_THRESHOLD
    }
}