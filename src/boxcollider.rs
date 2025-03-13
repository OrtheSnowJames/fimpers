// on collision, move the collider back out of the box using bevy 0.15.3
use bevy::prelude::*;

#[derive(Component)]
struct BoxCollider {
    pub size: Vec2,
}

impl BoxCollider {
    pub fn new(width: f32, height: f32) -> Self {
        BoxCollider { size: Vec2::new(width, height) }
    }


}