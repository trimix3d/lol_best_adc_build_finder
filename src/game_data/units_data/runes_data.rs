use super::{ap_formula, runes_hp_by_lvl, Unit};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum RuneShard {
    Left,
    Middle,
    Right,
}

/// Represents the runes page of a champion.
/// For now, only implements runes shards.
#[derive(Debug, Clone)]
pub struct RunesPage {
    pub shard1: RuneShard,
    pub shard2: RuneShard,
    pub shard3: RuneShard,
}

impl Unit {
    /// Sets the Unit runes, returns Ok if success or Err if failure (depending on the validity of the given runes page).
    /// In case of a failure, the unit is not modified.
    /// In the current state, this function will always succeed because all possible runes pages are valid (but it may change in the future).
    pub fn set_runes(&mut self, runes_page: RunesPage) -> Result<(), String> {
        self.runes_page = runes_page;
        Ok(())
    }

    /// Updates unit runes stats (stats only coming from runes).
    ///
    /// Because of runes hp by lvl and adaptive force, runes stats actually depend on lvl and items as well.
    /// For this reason, this function must be ran after being sure that `Unit.lvl_stats` and `Unit.items_stats` are up to date.
    /// This also means that runes stats might become out of date after changing lvl/items.
    pub fn update_runes_stats(&mut self) {
        self.runes_stats.put_to_zero();

        //adaptive force doesn't count in champions passives, so it only depends on items stats in practise
        let runes_adaptive_bonus_ad: f32;
        let runes_adaptive_ap: f32;
        if self.items_stats.bonus_ad
            >= ap_formula(self.items_stats.ap_flat, self.items_stats.ap_coef)
        {
            runes_adaptive_bonus_ad = 5.4;
            runes_adaptive_ap = 0.;
        } else {
            runes_adaptive_bonus_ad = 0.;
            runes_adaptive_ap = 9.;
        }

        match self.runes_page.shard1 {
            RuneShard::Left => {
                self.runes_stats.bonus_ad += runes_adaptive_bonus_ad;
                self.runes_stats.ap_flat += runes_adaptive_ap;
            }
            RuneShard::Middle => self.runes_stats.bonus_as += 0.10,
            RuneShard::Right => self.runes_stats.ability_haste += 8.,
        }

        match self.runes_page.shard2 {
            RuneShard::Left => {
                self.runes_stats.bonus_ad += runes_adaptive_bonus_ad;
                self.runes_stats.ap_flat += runes_adaptive_ap;
            }
            RuneShard::Middle => self.runes_stats.ms_percent += 0.02,
            RuneShard::Right => self.runes_stats.hp += runes_hp_by_lvl(self.lvl),
        }

        match self.runes_page.shard3 {
            RuneShard::Left => self.runes_stats.hp += 65.,
            RuneShard::Middle => (), //tenacity & slow resist: do nothing (not implemented)
            RuneShard::Right => self.runes_stats.hp += runes_hp_by_lvl(self.lvl),
        }
    }
}
