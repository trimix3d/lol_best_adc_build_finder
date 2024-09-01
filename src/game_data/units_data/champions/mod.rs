use super::{Unit, UnitProperties};

//list every champion file
mod aphelios;
mod ashe;
mod caitlyn;
mod draven;
mod ezreal;
mod jinx;
mod kaisa;
mod lucian;
mod sivir;
mod varus;
mod xayah;

impl Unit {
    pub const ALL_CHAMPIONS: [&'static UnitProperties; 11] = [
        &Unit::APHELIOS_PROPERTIES,
        &Unit::ASHE_PROPERTIES,
        &Unit::CAITLYN_PROPERTIES,
        &Unit::DRAVEN_PROPERTIES,
        &Unit::EZREAL_PROPERTIES,
        &Unit::JINX_PROPERTIES,
        &Unit::KAISA_PROPERTIES,
        &Unit::LUCIAN_PROPERTIES,
        &Unit::SIVIR_PROPERTIES,
        &Unit::VARUS_PROPERTIES,
        &Unit::XAYAH_PROPERTIES,
    ];
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    pub fn test_empty_fight_scenarios() {
        for properties in Unit::ALL_CHAMPIONS.iter() {
            if properties.fight_scenarios.is_empty() {
                panic!("The champion '{}' has no fight scenarios", properties.name);
            }
        }
    }

    #[test]
    pub fn test_champions_names_collisions() {
        //get champions names and sort them
        let mut names: Vec<&str> = Unit::ALL_CHAMPIONS
            .iter()
            .map(|properties| properties.name)
            .collect();
        names.sort_unstable();

        //compare adjacent elements of sorted vec to find names collisions
        for window in names.windows(2) {
            if window[0] == window[1] {
                panic!("champion name collision encountered: {:?}", window[0])
            }
        }
    }
}
