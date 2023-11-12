use std::rc::Rc;

use geop_geometry::{transforms::Transform, points::point::Point, curves::line::Line};

use crate::topology::{face::FaceContainsPoint};

use super::{contour::Contour, face::Face, edge::{Edge, EdgeIntersection, edge_curve::EdgeCurve}};

pub struct Volume {
    pub faces: Vec<Rc<Face>>,
}


pub enum VolumeContainsPoint {
    Inside,
    OnFace(Rc<Face>),
    OnEdge(Rc<Edge>),
    OnPoint(Rc<Point>),
    Outside,
}

pub enum VolumeNormal {
    OnFace(Point),
    OnEdge(Point, Point),
    OnPoint(Vec<Point>),
}


#[derive(Debug)]
pub enum FaceSplit {
    AinB(Rc<Face>),
    AonBSameSide(Rc<Face>),
    AonBOpSide(Rc<Face>),
    AoutB(Rc<Face>),
    BinA(Rc<Face>),
    BonASameSide(Rc<Face>),
    BonAOpSide(Rc<Face>),
    BoutA(Rc<Face>),
}

impl VolumeNormal {
    pub fn is_from_inside(&self, point: Point) -> bool {{
        match self {
            VolumeNormal::OnFace(normal) => normal.dot(point) > 0.0,
            VolumeNormal::OnEdge(normal1, normal2) => normal1.dot(point) > 0.0 && normal2.dot(point) > 0.0,
            VolumeNormal::OnPoint(normals) => {
                for normal in normals {
                    if normal.dot(point) < 0.0 {
                        return false;
                    }
                }
                true
            }
        }
    }}
}

#[derive(Debug)]
pub enum VolumeShellIntersection {
    Contour(Contour),
    Point(Point),
}

impl Volume {
    pub fn new(faces: Vec<Rc<Face>>) -> Volume {
        assert!(faces.len() > 0, "Volume must have at least one face");
        Volume { faces }
    }
    
    pub fn transform(&self, transform: Transform) -> Volume {
        Volume { faces: self.faces.iter().map(|f| Rc::new(f.transform(transform))).collect() }
    }

    pub fn normal(&self, point: Point) -> VolumeNormal {
        let mut relevant_normals = Vec::<Point>::new();
        for face in self.faces.iter() {
            match face.contains_point(point) {
                FaceContainsPoint::Inside | FaceContainsPoint::OnEdge(_) | FaceContainsPoint::OnPoint(_) => {
                    relevant_normals.push(face.normal(point));
                },
                FaceContainsPoint::Outside => {}
            }
        }
        match relevant_normals.len() {
            0 => panic!("Point is not inside volume"),
            1 => VolumeNormal::OnFace(relevant_normals[0]),
            2 => {
                VolumeNormal::OnEdge(relevant_normals[0], relevant_normals[1])
            }
            _ => {
                VolumeNormal::OnPoint(relevant_normals)
            }
        }
    }

    pub fn contains_point(&self, other: Point) -> VolumeContainsPoint {
        // first check if point is on any other face
        for face in self.faces.iter() {
            match face.contains_point(other) {
                FaceContainsPoint::Inside => return VolumeContainsPoint::OnFace(face.clone()),
                FaceContainsPoint::OnEdge(edge) => return VolumeContainsPoint::OnEdge(edge),
                FaceContainsPoint::OnPoint(point) => return VolumeContainsPoint::OnPoint(point),
                FaceContainsPoint::Outside => {}
            }
        }

        // choose a random point on a face
        let q = self.faces[0].inner_point();
        let curve = Edge::new(
            Rc::new(other.clone()), 
            Rc::new(q.clone()),
            Rc::new(EdgeCurve::Line(Line::new(other, q - other))));

        // Find the closest intersection point with any other face and use the normal to determine if the point is inside or outside
        for face in self.faces.iter() {
            let intersections = face.intersect_edge(&curve);
        }
        let mut closest_distance = (other - q).norm();
        let curve_dir = q - other;
        let normal = self.normal(q);
        let mut closest_intersect_from_inside = normal.is_from_inside(curve_dir);
        for face in self.faces.iter() {
            let edge_intersections = face.intersect_edge(&curve);
            let mut intersections = Vec::<Point>::new();
            for intersection in edge_intersections {
                match intersection {
                    EdgeIntersection::Point(point) => {
                        intersections.push(*point);
                    },
                    EdgeIntersection::Edge(edge) => {
                        intersections.push(*edge.start);
                        intersections.push(*edge.end);
                    }
                }
            }
            for point in intersections {
                let distance = (other - point).norm();
                if distance < closest_distance {
                    let curve_dir = curve.tangent(point);
                    let normal = self.normal(point);
                    closest_distance = distance;
                    closest_intersect_from_inside = normal.is_from_inside(curve_dir);
                }
            }
        }

        match closest_intersect_from_inside {
            true => VolumeContainsPoint::Inside,
            false => VolumeContainsPoint::Outside,
        }
    }

    pub fn shell_intersect(&self, other: &Volume) -> Vec<VolumeShellIntersection> {
        let intersections = Vec::<EdgeIntersection>::new();
        for face in self.faces.iter() {
            for other_face in other.faces.iter() {
                // intersections.extend(face.intersect(&other_face));
            }
        }

        todo!("Volume::intersect")
    }

    pub fn split_parts<F>(&self, other: &Volume, filter: F) -> Face
    where
        F: Fn(&FaceSplit) -> bool,
    {
        let mut intersections = self.shell_intersect(other);
        for int in intersections.iter() {
            println!("Intersection: {:?}", int);
        }

        todo!("Volume::split_parts")
        // let mut contours_self = self.boundaries.clone();
        // let mut contours_other = other.boundaries.clone();

        // for vert in intersections {
        //     contours_self = contours_self
        //         .into_iter()
        //         .map(|contour| contour.split_if_necessary(&vert))
        //         .collect();
        //     contours_other = contours_other
        //         .into_iter()
        //         .map(|contour| contour.split_if_necessary(&vert))
        //         .collect();
        // }

        // let mut edges = contours_self
        //     .into_iter()
        //     .map(|contour| {
        //         return contour
        //             .edges
        //             .into_iter()
        //             .map(|edge| match other.contains_edge(&edge) {
        //                 FaceContainsEdge::Inside => EdgeSplit::AinB(edge),
        //                 FaceContainsEdge::OnBorderSameDir => EdgeSplit::AonBSameSide(edge),
        //                 FaceContainsEdge::OnBorderOppositeDir => EdgeSplit::AonBOpSide(edge),
        //                 FaceContainsEdge::Outside => EdgeSplit::AoutB(edge),
        //             })
        //             .collect::<Vec<EdgeSplit>>();
        //     })
        //     .chain(contours_other.into_iter().map(|contour| {
        //         contour
        //             .edges
        //             .into_iter()
        //             .map(|edge| match self.contains_edge(&edge) {
        //                 FaceContainsEdge::Inside => EdgeSplit::BinA(edge),
        //                 FaceContainsEdge::OnBorderSameDir => EdgeSplit::BonASameSide(edge),
        //                 FaceContainsEdge::OnBorderOppositeDir => EdgeSplit::BonAOpSide(edge),
        //                 FaceContainsEdge::Outside => EdgeSplit::BoutA(edge),
        //             })
        //             .collect::<Vec<EdgeSplit>>()
        //     }))
        //     .flatten()
        //     .filter(filter)
        //     .map(|e| match e {
        //         EdgeSplit::AinB(edge) => edge,
        //         EdgeSplit::AonBSameSide(edge) => edge,
        //         EdgeSplit::AonBOpSide(edge) => edge,
        //         EdgeSplit::AoutB(edge) => edge,
        //         EdgeSplit::BinA(edge) => edge,
        //         EdgeSplit::BonASameSide(edge) => edge,
        //         EdgeSplit::BonAOpSide(edge) => edge,
        //         EdgeSplit::BoutA(edge) => edge,
        //     })
        //     .collect::<Vec<Rc<Edge>>>();

        // for edge in edges.iter() {
        //     println!("Edge: {:?}", edge);
        // }

        // // Now find all the contours
        // let mut contours = Vec::<Contour>::new();
        // while let Some(current_edge) = edges.pop() {
        //     let mut new_contour = vec![current_edge];
        //     loop {
        //         let next_i = edges.iter().position(|edge| {
        //             edge.start == new_contour[new_contour.len() - 1].end
        //                 || edge.end == new_contour[new_contour.len() - 1].end
        //         });
        //         match next_i {
        //             Some(i) => {
        //                 if edges[i].start == new_contour[new_contour.len() - 1].end {
        //                     new_contour.push(edges.remove(i));
        //                 } else {
        //                     new_contour.push(Rc::new(edges.remove(i).neg()));
        //                 }
        //             }
        //             None => {
        //                 assert!(new_contour[0].start == new_contour[new_contour.len() - 1].end);
        //                 contours.push(Contour::new(new_contour));
        //                 break;
        //             }
        //         }
        //     }
        // }

        // return Face::new(contours, self.surface.clone());
    }
}
