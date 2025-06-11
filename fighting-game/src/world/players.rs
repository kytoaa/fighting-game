use super::World;
use crate::datatypes::MoveTowards;

pub(super) struct TrackedPlayerData {
    pub(super) max_health: u32,
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
    pub const CANCEL_COST: u32 = 3333;
    pub const BURST_MAX: u32 = 10000;

    pub const fn new(max_health: u32, burst_meter: u32) -> Self {
        Self {
            max_health,
            health: max_health,
            meter: 0,
            meter_gain: 1200,
            burst_meter,
            scaling: 0,
            damage_boost: 0,
            defense_boost: 0,
            frames_since_scaling_set: 0,
        }
    }
    pub fn add_meter(&mut self, meter: u32) {
        self.meter += meter * self.meter_gain / 1000;
        self.meter = self.meter.clamp(0, 10000);
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
    pub fn add_burst(&mut self, burst: u32) {
        self.burst_meter = (self.burst_meter + burst).min(Self::BURST_MAX);
        if burst > 5 {
            println!("adding {} burst, burst at {}", burst, self.burst_meter);
        }
    }
    pub fn damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        if self.health <= 0 {
            println!("dead");
        }
    }
}

pub fn burst_gain(damage: u32, hit_number: usize) -> u32 {
    damage
        * (10
            + match hit_number as u32 {
                h @ 0..15 => h * 2,
                h @ 15..25 => h * 4,
                h @ 25.. => h * 8,
            })
        / 10
}

impl World {
    pub fn update_player_meters(&mut self) {
        for (i, player_data) in self.player_data.iter_mut().enumerate() {
            if self.combo.is_none() && player_data.scaling > 0 {
                player_data.scaling = 0;
            }
            if player_data.frames_since_scaling_set >= 60 && player_data.scaling < 0 {
                player_data.scaling = player_data.scaling.move_towards(0, 3000 / 60);
            }
            player_data.frames_since_scaling_set += 1;

            player_data.add_meter(1);

            player_data.add_burst(calculate_burst_gain(&player_data));
            if player_data.burst_meter % 1000 == 0 && player_data.burst_meter != 10000 {
                println!("player {} burst: {}", i, player_data.burst_meter);
            }
        }
    }
}

pub fn calculate_burst_gain(data: &TrackedPlayerData) -> u32 {
    const BASE_GAIN_PER_FRAME: u32 = 1; // takes roughly 167 seconds to reach full

    let health_percent = data.health * 100 / data.max_health;

    if health_percent < 20 {
        BASE_GAIN_PER_FRAME * 2
    } else {
        BASE_GAIN_PER_FRAME
    }
}
