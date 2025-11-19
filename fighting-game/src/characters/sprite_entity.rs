use super::*;

#[derive(Clone)]
pub struct SpriteEntity<const FRAMES: usize> {
    position: Vector2,
    facing_right: bool,
    id: EntityID,

    velocity: Vector2,
    frame: usize,
    frames: [(Box<str>, usize, Vector2); FRAMES],
    draw_behind_players: bool,
}

impl<const FRAMES: usize> SpriteEntity<FRAMES> {
    pub fn new(
        frames: [(Box<str>, usize, Vector2); FRAMES],
        position: Vector2,
        id: EntityID,
        facing_right: bool,
        velocity: Vector2,
    ) -> Self {
        Self {
            position,
            facing_right,
            id,

            velocity,
            frame: 0,
            frames,
            draw_behind_players: false,
        }
    }
    pub fn draw_behind_players(self) -> Self {
        Self {
            draw_behind_players: true,
            ..self
        }
    }
}

impl<const FRAMES: usize> NonPlayerEntity for SpriteEntity<FRAMES> {
    fn update(&mut self, _: &mut World, _: Option<&InputHandler>) -> EntityUpdateResult {
        self.frame += 1;

        self.position += self.velocity / 60.0;

        if self.frame >= self.frames.iter().map(|(_, l, _)| *l).sum() {
            EntityUpdateResult::Remove
        } else {
            EntityUpdateResult::Continue
        }
    }
    fn dir(&self) -> bool {
        self.facing_right
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut f = self.frame;

        for (sprite, length, offset) in &self.frames {
            f = f.saturating_sub(*length);
            if f == 0 {
                return Some((sprite.clone(), *offset));
            }
        }
        None
    }
    fn draw_behind_players(&self) -> bool {
        self.draw_behind_players
    }
}

impl<const FRAMES: usize> Position for SpriteEntity<FRAMES> {
    fn position(&self) -> Vector2 {
        self.position
    }
    fn move_by(&mut self, distance: Vector2) {
        self.position += distance
    }
    fn set_position(&mut self, position: Vector2) {
        self.position = position
    }
}

impl<const FRAMES: usize> HasID for SpriteEntity<FRAMES> {
    fn id(&self) -> EntityID {
        self.id
    }
}

impl<const FRAMES: usize> OnHit for SpriteEntity<FRAMES> {
    fn on_hit(&mut self, _: HitConnectionStatus) {}
}

impl<const FRAMES: usize> NonPlayerClone for SpriteEntity<FRAMES> {
    fn clone(&self) -> Box<dyn NonPlayerEntity> {
        Box::new(Clone::clone(self))
    }
}
