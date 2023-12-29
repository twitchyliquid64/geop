use crate::topology::{face::Face, intersections::face_face::face_face_same_surface_intersection};

pub fn face_face_difference(face_self: &Face, face_other: &Face) -> Face {
    assert!(face_self.surface == face_other.surface);
    return face_face_same_surface_intersection(&face_self, &face_other.neg());
}
