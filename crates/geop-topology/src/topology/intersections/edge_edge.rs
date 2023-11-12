use std::rc::Rc;

use geop_geometry::{
    curve_curve_intersection::{
        circle_circle::{circle_circle_intersection, CircleCircleIntersection},
        line_line::{line_line_intersection, LineLineIntersection},
    },
    curves::curve::Curve, points::point::Point,
};

use crate::topology::{edge::{Edge, edge_curve::EdgeCurve}, contains::edge_point::{EdgeContains, edge_contains_point}};


#[derive(Clone, Debug)]
pub enum EdgeEdgeIntersection {
    Point(Rc<Point>),
    Edge(Edge),
}

impl Edge {
    // All intersections where it crosses other edge. The end points are not included. The list is sorted from start to end.
    pub fn intersections(&self, other: &Edge) -> Vec<EdgeEdgeIntersection> {
        match *self.curve {
            EdgeCurve::Circle(ref circle) => match *other.curve {
                EdgeCurve::Circle(ref other_circle) => {
                    match circle_circle_intersection(circle, other_circle) {
                        CircleCircleIntersection::TwoPoint(a, b) => {
                            let mut intersections = Vec::new();
                            let u_a = circle.project(a).0;
                            let u_b = circle.project(b).0;
                            if u_a < u_b {
                                intersections.push(Rc::new(a));
                                intersections.push(Rc::new(b));
                            } else {
                                intersections.push(Rc::new(b));
                                intersections.push(Rc::new(a));
                            }
                            intersections
                                .into_iter()
                                .filter(|intersection| {
                                    edge_contains_point(self, **intersection) == EdgeContains::Inside
                                        && edge_contains_point(other, **intersection) == EdgeContains::Inside
                                })
                                .map(|i| EdgeEdgeIntersection::Point(i))
                                .collect()
                        }
                        CircleCircleIntersection::OnePoint(a) => {
                            if edge_contains_point(self, a) == EdgeContains::Inside
                                && edge_contains_point(other, a) == EdgeContains::Inside
                            {
                                vec![EdgeEdgeIntersection::Point(Rc::new(a))]
                            } else {
                                vec![]
                            }
                        }
                        CircleCircleIntersection::None => {
                            vec![]
                        }
                        CircleCircleIntersection::Circle(_) => {
                            let mut self_start_u = circle.project(*self.start).0;
                            let mut self_end_u = circle.project(*self.end).0;
                            let mut other_start_u = other_circle.project(*other.start).0;
                            let mut other_end_u = other_circle.project(*other.end).0;

                            if self_end_u < self_start_u.max(other_start_u) {
                                self_start_u += 2.0 * std::f64::consts::PI;
                                self_end_u += 2.0 * std::f64::consts::PI;
                            }

                            if other_end_u < self_start_u.max(other_start_u) {
                                other_start_u += 2.0 * std::f64::consts::PI;
                                other_end_u += 2.0 * std::f64::consts::PI;
                            }

                            let start_u = self_start_u.max(other_start_u);
                            let end_u = self_end_u.min(other_end_u);

                            if self.start == other.start && self.end == other.end {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    self.start.clone(),
                                    self.end.clone(),
                                    self.curve.clone(),
                                ))]
                            } else if end_u < start_u {
                                vec![]
                            } else if self.start == other.start {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    self.start.clone(),
                                    Rc::new(circle.point_at(end_u)),
                                    self.curve.clone(),
                                ))]
                            } else if self.start == other.end {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    self.start.clone(),
                                    Rc::new(circle.point_at(end_u)),
                                    self.curve.clone(),
                                ))]
                            } else if self.end == other.start {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    Rc::new(circle.point_at(start_u)),
                                    self.end.clone(),
                                    self.curve.clone(),
                                ))]
                            } else if self.end == other.end {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    Rc::new(circle.point_at(start_u)),
                                    self.end.clone(),
                                    self.curve.clone(),
                                ))]
                            } else {
                                vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    Rc::new(circle.point_at(start_u)),
                                    Rc::new(circle.point_at(end_u)),
                                    self.curve.clone(),
                                ))]
                            }
                        }
                    }
                }
                EdgeCurve::Ellipse(ref _ellipse) => {
                    todo!("Implement circle-ellipse intersection")
                }
                EdgeCurve::Line(ref _line) => {
                    todo!("Implement circle-line intersection")
                }
            },
            EdgeCurve::Ellipse(ref _ellipse) => {
                todo!("Implement ellipse intersection")
            }
            EdgeCurve::Line(ref line) => {
                match *other.curve {
                    EdgeCurve::Circle(ref _circle) => {
                        todo!("Implement line-circle intersection")
                    }
                    EdgeCurve::Ellipse(ref _ellipse) => {
                        todo!("Implement line-ellipse intersection")
                    }
                    EdgeCurve::Line(ref other_line) => {
                        match line_line_intersection(line, other_line) {
                            LineLineIntersection::Point(a) => {
                                let mut intersections = Vec::new();
                                // Check if it is contained
                                if edge_contains_point(self, a) == EdgeContains::Inside
                                    && edge_contains_point(other, a) == EdgeContains::Inside
                                {
                                    intersections.push(EdgeEdgeIntersection::Point(Rc::new(a)));
                                }
                                intersections
                            }
                            LineLineIntersection::None => {
                                vec![]
                            }
                            LineLineIntersection::Line(_) => {
                                let start_u_other = self.curve.curve().project(*other.start).0;
                                let end_u_other = self.curve.curve().project(*other.end).0;

                                let (other_start, other_end) = match start_u_other < end_u_other {
                                    true => (&other.start, &other.end),
                                    false => (&other.end, &other.start),
                                };

                                // Now that we have the right order
                                let start_u_other = self.curve.curve().project(**other_start).0;
                                let end_u_other = self.curve.curve().project(**other_end).0;

                                if start_u_other > self.end_u || end_u_other < self.start_u {
                                    return vec![];
                                }

                                let start = match start_u_other < self.start_u {
                                    true => self.start.clone(),
                                    false => other_start.clone(),
                                };

                                let end = match self.end_u < end_u_other {
                                    true => self.end.clone(),
                                    false => other_end.clone(),
                                };

                                if start == end {
                                    assert!(
                                        edge_contains_point(self, *start) != EdgeContains::Outside
                                            && edge_contains_point(other, *start) != EdgeContains::Outside
                                    );
                                    return vec![EdgeEdgeIntersection::Point(start)];
                                }

                                // println!("Direction: {:?}", self.direction);
                                // println!("Start: {:?}", start);
                                // println!("End: {:?}", end);

                                // println!("Self start u: {:?}", self.start_u);
                                // println!("Self end u: {:?}", self.end_u);
                                // println!("Other start u: {:?}", start_u_other);
                                // println!("Other end u: {:?}", end_u_other);

                                // println!("Self: {:?}", self);
                                // println!("Other: {:?}", other);

                                assert!(edge_contains_point(self, *start) != EdgeContains::Outside);
                                assert!(edge_contains_point(other, *start) != EdgeContains::Outside);
                                assert!(edge_contains_point(self, *end) != EdgeContains::Outside);
                                assert!(edge_contains_point(other, *end) != EdgeContains::Outside);
                                return vec![EdgeEdgeIntersection::Edge(Edge::new(
                                    start,
                                    end,
                                    self.curve.clone(),
                                ))];
                            }
                        }
                    }
                }
            }
        }
    }
}