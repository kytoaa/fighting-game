use super::{EntityID, World};
use crate::collision::{HitConnectionStatus, HitData, Proration};

pub(super) struct ComboInfo {
    hits: usize,
    total_damage: u32,
    proration: Proration,
    target: EntityID,

    /// combo scaling, when 0 no scaling is applied, when > 0 damage is reduced, when < 0 no
    /// scaling is applied, allowing for unscaled combos
    /// represents the additive scaling of the combo, does not include previous scaling
    /// min is -10000
    scaling: i32,
}
impl ComboInfo {
    pub const fn target(&self) -> &EntityID {
        &self.target
    }
}

pub(super) fn hit_player(
    hit_player: &mut Option<Box<dyn crate::characters::Entity>>,
    player_data: &mut super::players::TrackedPlayerData,
    combo: &mut Option<ComboInfo>,
    hit_player_id: EntityID,
    hit_data: &HitData,
) -> HitConnectionStatus {
    let player = hit_player.take().unwrap();

    let combo_info = combo.take();
    if combo_info
        .as_ref()
        .map(|c| c.target != hit_player_id)
        .unwrap_or(false)
    {
        panic!("combo was not reset");
    }

    let (grounded, counterhit) = (player.is_grounded(), player.counterhit());

    let on_hit_hitdata = hit_data.as_on_hit_hitdata(grounded, counterhit, |damage| {
        total_damage_scaling(
            damage,
            combo_info
                .as_ref()
                .map(|c| c.proration.clone())
                .unwrap_or(Proration::percent(100)),
            player_data.scaling,
            player_data.damage_boost,
            player_data.defense_boost,
            hit_data.minimum_damage,
            player.counterhit(),
        )
    });

    let on_hit_hitdata_damage = on_hit_hitdata.damage;
    let (player, hit_status) = player.hit(on_hit_hitdata);

    let combo_info = match hit_status {
        crate::collision::HitConnectionStatus::Hit => {
            let mut combo_info = combo_info.unwrap_or_else(|| ComboInfo {
                hits: 0,
                total_damage: 0,
                proration: hit_data.proration.clone(),
                target: hit_player_id,
                scaling: 0,
            });

            combo_info.hits += 1;
            combo_info.total_damage += on_hit_hitdata_damage;
            combo_info.scaling += hit_data.scaling;

            player_data.scaling += hit_data.scaling;
            player_data.health = player_data.health.saturating_sub(on_hit_hitdata_damage);
            player_data.burst_meter += super::players::burst_gain(hit_data.damage, combo_info.hits);
            player_data.meter_gain =
                (player_data.meter_gain as i32 + hit_data.meter_gain_modifier) as u32;
            player_data.add_meter(hit_data.meter_gain);

            Some(combo_info)
        }
        crate::collision::HitConnectionStatus::Blocked => {
            player_data.scaling -= hit_data.scaling * if grounded { 1 } else { 2 };
            player_data.add_meter(hit_data.meter_gain / 2);

            None
        }
        crate::collision::HitConnectionStatus::Invuln => None,
    };

    *combo = combo_info;

    hit_player.insert(player);

    hit_status
}

pub const fn scale_damage(damage: u32, scaling: i32) -> u32 {
    const SCALING_TABLE: [u16; 11] = [1000, 950, 750, 560, 420, 300, 210, 140, 140, 60, 40];
    if scaling < 0 {
        damage
    } else {
        // lerp between table value and value above
        damage * {
            let scale_index = scaling / 1000;
            if scale_index as usize >= SCALING_TABLE.len() - 1 {
                *SCALING_TABLE.last().unwrap() as u32
            } else {
                let lower = SCALING_TABLE[scale_index as usize];
                let upper = SCALING_TABLE[(scale_index + 1) as usize];
                let dif = lower - upper;
                lower as u32 + (dif as u32 * (scaling as u32 - (scale_index as u32 * 1000)) / 1000)
            }
        } / 1000
    }
}

pub const fn total_damage_scaling(
    damage: u32,
    proration: Proration,
    scaling: i32,
    damage_boost: i32,
    enemy_defense_boost: i32,
    min_damage: u32,
    counterhit: bool,
) -> u32 {
    // proration -> combo scaling -> boosts -> counterhit -> min
    max(
        (scale_damage(proration.scale_damage(damage), scaling) as i32
            * (100 + damage_boost - enemy_defense_boost)
            / 100)
            * if counterhit { 11 } else { 10 }
            / 10,
        min_damage as i32,
    ) as u32
}

const fn max(a: i32, b: i32) -> i32 {
    if b > a {
        b
    } else {
        a
    }
}
