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
        }
    }
    pub fn add_meter(&mut self, meter: u32) {
        self.meter += meter * self.meter_gain / 1000
    }
}

pub fn burst_gain(damage: u32, hit_number: usize) -> u32 {
    damage * (10 + hit_number as u32 * 2) / 10
}
