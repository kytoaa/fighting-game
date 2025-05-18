use super::World;
use crate::datatypes::MoveTowards;

pub(super) struct TrackedPlayerData {
    pub(super) health: u32,

    /// max of 10000
    pub(super) meter: u32,
    /// rate of gain of meter, 1000 being default
    pub(super) meter_gain: u32,

    /// player burst meter, 10000 being full
    pub(super) burst_meter: u32,

    /// the current scaling for this player, getting hit increases scaling and blocking decreases it
    pub(super) scaling: i32,

    /// player deals n% more damage
    pub(super) damage_boost: i32,
    /// player takes n% less damage
    pub(super) defense_boost: i32,

    pub(super) frames_since_scaling_set: usize,
}
impl TrackedPlayerData {
    pub const fn new(max_health: u32) -> Self {
        Self {
            health: max_health,
            meter: 0,
            meter_gain: 1000,
            burst_meter: 10000,
            scaling: 0,
            damage_boost: 0,
            defense_boost: 0,
            frames_since_scaling_set: 0,
        }
    }
    pub fn add_meter(&mut self, meter: u32) {
        self.meter += meter * self.meter_gain / 1000
    }
    pub fn add_scaling(&mut self, scaling: i32) {
        self.scaling += scaling;
        if scaling < 0 {
            self.frames_since_scaling_set = 0;
        }
        if self.scaling <= -10000 {
            self.scaling = -10000;
        }
    }
}

pub fn burst_gain(damage: u32, hit_number: usize) -> u32 {
    damage * (10 + hit_number as u32 * 2) / 10
}

impl World {
    pub fn update_player_meters(&mut self) {
        for player_data in self.player_data.iter_mut() {
            if self.combo.is_none() && player_data.scaling > 0 {
                player_data.scaling = 0;
            }
            if player_data.frames_since_scaling_set >= 60 && player_data.scaling < 0 {
                player_data.scaling = player_data.scaling.move_towards(0, 3000 / 60);
            }
            player_data.frames_since_scaling_set += 1;
        }
    }
}
