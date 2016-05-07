use prelude::*;

#[derive(Clone, Debug)]
pub struct Pos(pub Vec3f);
impl specs::Component for Pos {
    type Storage = specs::VecStorage<Pos>;
}

#[derive(Clone, Debug)]
pub struct MoveTo(pub Vec3f, pub f32);
impl specs::Component for MoveTo {
    type Storage = specs::VecStorage<MoveTo>;
}
