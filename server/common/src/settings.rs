use std::collections::BTreeMap;

pub struct Settings {
    btree: BTreeMap<String, BTreeMap<String, String>>,
}

impl Settings {
    pub fn new(cfg_path: &str) -> Settings {
        let file = std::fs::File::open(cfg_path).unwrap();
        let t: BTreeMap<String, BTreeMap<String, String>> = serde_yaml::from_reader(&file).unwrap();
        Settings { btree: t }
    }

    pub fn get(&self, section: &str, key: &str) -> String {
        self.btree[section][key].to_string()
    }
}
