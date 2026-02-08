//! Parser for classes.json and generated registration code.

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ClassesJson {
    pub blocks: Vec<BlockEntry>,
    pub items: Vec<ItemEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BlockEntry {
    pub name: String,
    pub class: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemEntry {
    pub name: String,
    pub class: String,
}

impl ClassesJson {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let classes: ClassesJson = serde_json::from_str(&content)?;
        Ok(classes)
    }
}

/// Parse generated blocks.rs to extract registry_id -> steel_behavior mapping
pub fn parse_generated_blocks(path: &str) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut map = HashMap::new();

    // Match patterns like: vanilla_blocks :: BARREL , Box :: new (BarrelBlock :: new
    let re =
        Regex::new(r"vanilla_blocks\s*::\s*(\w+)\s*,\s*Box\s*::\s*new\s*\(\s*(\w+)\s*::").unwrap();

    for cap in re.captures_iter(&content) {
        let registry_id = cap[1].to_lowercase(); // BARREL -> barrel
        let steel_behavior = cap[2].to_string(); // BarrelBlock
        map.insert(registry_id, steel_behavior);
    }

    map
}

/// Parse generated items.rs to extract registry_id -> steel_behavior mapping
pub fn parse_generated_items(path: &str) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut map = HashMap::new();

    // Match patterns like: vanilla_items :: ITEMS . stone , Box :: new (BlockItemBehavior :: new
    let re =
        Regex::new(r"vanilla_items\s*::\s*ITEMS\s*\.\s*(\w+)\s*,\s*Box\s*::\s*new\s*\(\s*(\w+)")
            .unwrap();

    for cap in re.captures_iter(&content) {
        let registry_id = cap[1].to_string(); // stone (already lowercase)
        let steel_behavior = cap[2].to_string(); // BlockItemBehavior
        map.insert(registry_id, steel_behavior);
    }

    map
}

/// Combined mapping: vanilla_class -> steel_behavior (derived from registry IDs)
pub fn build_class_mapping(
    classes_json: &ClassesJson,
    steel_blocks: &HashMap<String, String>,
    steel_items: &HashMap<String, String>,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut block_mapping: HashMap<String, String> = HashMap::new();
    let mut item_mapping: HashMap<String, String> = HashMap::new();

    // For blocks: classes.json tells us registry_id -> vanilla_class
    // steel_blocks tells us registry_id -> steel_behavior
    // So we can derive vanilla_class -> steel_behavior
    for block in &classes_json.blocks {
        if let Some(steel_behavior) = steel_blocks.get(&block.name) {
            block_mapping.insert(block.class.clone(), steel_behavior.clone());
        }
    }

    // Same for items
    for item in &classes_json.items {
        if let Some(steel_behavior) = steel_items.get(&item.name) {
            item_mapping.insert(item.class.clone(), steel_behavior.clone());
        }
    }

    (block_mapping, item_mapping)
}
