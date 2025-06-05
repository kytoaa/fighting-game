use super::EntityID;
use crate::collision::{AttackID, HitConnectionStatus, HitData, HitDataExtension, Proration};
use crate::datatypes::BoundingShape;

#[derive(Debug)]
pub(super) struct ComboInfo {
    hits: usize,
    total_damage: u32,
    proration: Proration,
    target: EntityID,
    attacks_used: Vec<AttackID>,

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
    pub fn add_attack(&mut self, attack: AttackID) {
        self.attacks_used.push(attack);
    }

    pub const fn hits(&self) -> usize {
        self.hits
    }
    pub const fn total_damage(&self) -> u32 {
        self.total_damage
    }
}

pub(super) fn hit_player(
    hit_player: &mut Option<Box<dyn crate::characters::Player>>,
    player_data: &mut [super::players::TrackedPlayerData; 2],
    combo: &mut Option<ComboInfo>,
    hit_data: &HitData,
    attack_id: AttackID,
) -> HitConnectionStatus {
    let player = hit_player.take().unwrap();
    let hit_player_id = player.id();

    let mut combo_info = combo.take();
    if combo_info
        .as_ref()
        .map(|c| c.target != hit_player_id)
        .unwrap_or(false)
    {
        panic!("combo was not reset");
    }

    for extension in &hit_data.extensions {
        match extension {
            HitDataExtension::SetScaling(scaling) => {
                player_data[hit_player_id.id()].scaling = *scaling
            }
            HitDataExtension::SetProration(proration) => {
                combo_info = combo_info.map(|mut combo| {
                    combo.proration = *proration;
                    combo
                })
            }
            _ => (),
        }
    }

    let (grounded, counterhit) = (player.is_grounded(), player.counterhit());

    let mut on_hit_hitdata = hit_data.as_on_hit_hitdata(grounded, counterhit, |damage| {
        total_damage_scaling(
            damage,
            combo_info
                .as_ref()
                .map(|c| c.proration.clone())
                .unwrap_or(Proration::percent(100)),
            player_data[hit_player_id.id()].scaling,
            player_data[hit_player_id.id()].damage_boost,
            player_data[hit_player_id.id()].defense_boost,
            hit_data.minimum_damage,
            player.counterhit(),
        )
    });
    on_hit_hitdata.hit_effect = hit_data
        .extensions
        .iter()
        .find_map(|extension| {
            if let HitDataExtension::CleanHit(bb, effect) = extension {
                if bb.intersects(&player.position()) {
                    println!("clean hit");
                    Some(effect.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or(on_hit_hitdata.hit_effect);

    let on_hit_hitdata = on_hit_hitdata;
    let on_hit_hitdata_damage = on_hit_hitdata.damage;

    let (player, hit_status) = player.hit(on_hit_hitdata);

    let combo_info = match hit_status {
        crate::collision::HitConnectionStatus::Hit => {
            if counterhit {
                println!("counterhit");
            }
            let mut combo_info = combo_info.unwrap_or_else(|| ComboInfo {
                hits: 0,
                total_damage: 0,
                proration: hit_data.proration.clone(),
                target: hit_player_id,
                attacks_used: vec![],
                scaling: 0,
            });

            let uses_before_scaling = hit_data
                .extensions
                .iter()
                .find_map(|e| match e {
                    HitDataExtension::UsagesBeforeScaling(u) => Some(*u),
                    _ => None,
                })
                .unwrap_or(2);

            combo_info.hits += 1;
            combo_info.total_damage += on_hit_hitdata_damage;
            combo_info.scaling += hit_data.scaling
                * if combo_info
                    .attacks_used
                    .iter()
                    .filter(|a| **a == attack_id)
                    .count()
                    >= uses_before_scaling
                {
                    2
                } else {
                    1
                };
            combo_info.add_attack(attack_id);

            player_data[hit_player_id.id()].scaling += hit_data.scaling;
            player_data[hit_player_id.id()].health = player_data[hit_player_id.id()]
                .health
                .saturating_sub(on_hit_hitdata_damage);

            player_data[hit_player_id.id()]
                .add_burst(super::players::burst_gain(hit_data.damage, combo_info.hits));

            player_data[hit_player_id.other_player().id()].meter_gain =
                (player_data[hit_player_id.id()].meter_gain as i32 + hit_data.meter_gain_modifier)
                    as u32;

            player_data[hit_player_id.other_player().id()].add_meter(hit_data.meter_gain);

            Some(combo_info)
        }
        crate::collision::HitConnectionStatus::Blocked => {
            let block_scaling = hit_data
                .extensions
                .iter()
                .find_map(|extension| {
                    if let HitDataExtension::SetBlockScaling(scaling) = extension {
                        Some(*scaling)
                    } else {
                        None
                    }
                })
                .unwrap_or(hit_data.scaling)
                * if grounded { 1 } else { 2 }
                * hit_data.scaling_on_block_mult as i32
                / 100;

            player_data[hit_player_id.id()].add_meter(hit_data.meter_gain / 4);
            player_data[hit_player_id.other_player().id()].add_meter(hit_data.meter_gain / 2);
            player_data[hit_player_id.other_player().id()].add_scaling(-block_scaling);

            None
        }
        crate::collision::HitConnectionStatus::Invuln => None,
    };

    *combo = combo_info;

    _ = hit_player.insert(player);

    hit_status
}

pub fn total_damage_scaling(
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

pub fn scale_damage(damage: u32, scaling: i32) -> u32 {
    const SCALING_TABLE: [u16; 11] = [1000, 950, 750, 560, 420, 300, 210, 140, 140, 60, 40];
    if scaling < 0 {
        damage
    } else {
        // lerp between table value and value above
        let scale_factor = {
            let scale_index = scaling / 2000;
            if scale_index as usize >= SCALING_TABLE.len() - 1 {
                *SCALING_TABLE.last().unwrap() as u32
            } else {
                let lower = SCALING_TABLE[scale_index as usize];
                let upper = SCALING_TABLE[(scale_index + 1) as usize];
                let dif = lower - upper;
                lower as u32 + (dif as u32 * (scaling as u32 - (scale_index as u32 * 2000)) / 2000)
            }
        };
        damage * scale_factor / 1000
    }
}

const fn max(a: i32, b: i32) -> i32 {
    if b > a {
        b
    } else {
        a
    }
}
