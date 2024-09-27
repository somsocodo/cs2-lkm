use std::collections::HashMap;

pub struct IconResolver {
    icons: HashMap<&'static str, &'static str>,
}

impl IconResolver {
    pub fn new() -> Self {
        let mut icons = HashMap::new();
        icons.insert("knife_ct", "]");
        icons.insert("knife_t", "[");
        icons.insert("knife", "[");
        icons.insert("weapon_deagle", "A");
        icons.insert("weapon_elite", "B");
        icons.insert("weapon_fiveseven", "C");
        icons.insert("weapon_five-seven", "C");
        icons.insert("weapon_glock", "D");
        icons.insert("weapon_hkp2000", "E");
        icons.insert("weapon_p2000", "E");
        icons.insert("weapon_p250", "F");
        icons.insert("weapon_usp_silencer", "G");
        icons.insert("weapon_tec9", "H");
        icons.insert("weapon_cz75a", "I");
        icons.insert("weapon_revolver", "J");
        icons.insert("weapon_mac10", "K");
        icons.insert("weapon_mac-10", "K");
        icons.insert("weapon_ump45", "L");
        icons.insert("weapon_bizon", "M");
        icons.insert("weapon_mp7", "N");
        icons.insert("weapon_mp9", "O");
        icons.insert("weapon_mp5sd", "x");
        icons.insert("weapon_p90", "P");
        icons.insert("weapon_galilar", "Q");
        icons.insert("weapon_galil", "Q");
        icons.insert("weapon_famas", "R");
        icons.insert("weapon_m4a4", "S");
        icons.insert("weapon_m4a1_silencer", "T");
        icons.insert("weapon_m4a1", "T");
        icons.insert("weapon_aug", "U");
        icons.insert("weapon_sg556", "V");
        icons.insert("weapon_ak47", "W");
        icons.insert("weapon_g3sg1", "X");
        icons.insert("weapon_scar20", "Y");
        icons.insert("weapon_awp", "Z");
        icons.insert("weapon_ssg08", "a");
        icons.insert("weapon_ssg-08", "a");
        icons.insert("weapon_xm1014", "b");
        icons.insert("weapon_sawedoff", "c");
        icons.insert("weapon_mag7", "d");
        icons.insert("weapon_nova", "e");
        icons.insert("weapon_negev", "f");
        icons.insert("weapon_m249", "g");
        icons.insert("weapon_taser", "h");
        icons.insert("flashbang_projectile", "i");
        icons.insert("weapon_flashbang", "i");
        icons.insert("hegrenade_projectile", "j");
        icons.insert("weapon_hegrenade", "j");
        icons.insert("smokegrenade_projectile", "k");
        icons.insert("weapon_smokegrenade", "k");
        icons.insert("molotov_projectile", "l");
        icons.insert("weapon_molotov", "l");
        icons.insert("decoy_projectile", "m");
        icons.insert("weapon_decoy", "m");
        icons.insert("incgrenade_projectile", "n");
        icons.insert("weapon_incgrenade", "n");
        icons.insert("weapon_c4", "o");
        icons.insert("defuse kit", "r");

        IconResolver { icons }
    }

    pub fn resolve_icon(&self, weapon: &str) -> Option<&str> {
        self.icons.get(weapon).copied()
    }
}