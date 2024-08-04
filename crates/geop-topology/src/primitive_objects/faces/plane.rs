use std::rc::Rc;

use geop_geometry::{
    points::point::Point,
    surfaces::{plane::Plane, sphere::Sphere, surface::Surface},
};

use crate::topology::face::Face;

pub fn primitive_plane(basis: Point, u_slope: Point, v_slope: Point) -> Face {
    let plane = Plane::new(basis, u_slope, v_slope);
    Face::new(None, vec![], Rc::new(Surface::Plane(plane)))
}