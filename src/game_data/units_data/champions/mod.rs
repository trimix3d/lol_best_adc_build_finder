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
        Unit::APHELIOS_PROPERTIES_REF,
        Unit::ASHE_PROPERTIES_REF,
        Unit::CAITLYN_PROPERTIES_REF,
        Unit::DRAVEN_PROPERTIES_REF,
        Unit::EZREAL_PROPERTIES_REF,
        Unit::JINX_PROPERTIES_REF,
        Unit::KAISA_PROPERTIES_REF,
        Unit::LUCIAN_PROPERTIES_REF,
        Unit::SIVIR_PROPERTIES_REF,
        Unit::VARUS_PROPERTIES_REF,
        Unit::XAYAH_PROPERTIES_REF,
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
                panic!("Champion name collision encountered: {:?}", window[0])
            }
        }
    }
}
