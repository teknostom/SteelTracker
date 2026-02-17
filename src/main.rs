use std::collections::HashMap;
use std::fs;
use steel_tracker::extractors::{parse_java_files_unfiltered, parse_rust_files_unfiltered};
use steel_tracker::registry_parser::{self, ClassesJson};
use steel_tracker::types::{
    AnalysisResult, ClassMethods, ClassTracking, ImplementationStatus, MethodTracking,
};

fn main() {
    fs::create_dir_all("outputs").expect("Failed to create outputs directory");

    // Load classes.json for registry-based mapping
    let classes_json = ClassesJson::load("sources/SteelMC/steel-core/build/classes.json")
        .expect("Failed to load classes.json");

    // Parse generated registration code
    let steel_blocks = registry_parser::parse_generated_blocks(
        "sources/SteelMC/steel-core/src/behavior/generated/blocks.rs",
    );
    let steel_items = registry_parser::parse_generated_items(
        "sources/SteelMC/steel-core/src/behavior/generated/items.rs",
    );

    // Build vanilla_class -> steel_behavior mapping
    let (block_class_map, item_class_map) =
        registry_parser::build_class_mapping(&classes_json, &steel_blocks, &steel_items);

    // Parse all Java classes from entity, block, and item folders
    let mut all_java_classes: Vec<ClassMethods> = Vec::new();

    // Blocks
    let block_classes =
        parse_java_files_unfiltered("sources/yarn/build/namedSrc/net/minecraft/block");
    for mut c in block_classes {
        c.class_type = "block".to_string();
        all_java_classes.push(c);
    }

    // Items
    let item_classes =
        parse_java_files_unfiltered("sources/yarn/build/namedSrc/net/minecraft/item");
    for mut c in item_classes {
        c.class_type = "item".to_string();
        all_java_classes.push(c);
    }

    // Entities and AI (with sub-type detection)
    let entity_classes =
        parse_java_files_unfiltered("sources/yarn/build/namedSrc/net/minecraft/entity");
    for mut c in entity_classes {
        c.class_type = detect_entity_subtype(&c.class_name);
        all_java_classes.push(c);
    }

    // Write combined java.json
    let java_json = serde_json::to_string_pretty(&all_java_classes).unwrap();
    fs::write("outputs/java.json", java_json).unwrap();
    println!(
        "Wrote outputs/java.json ({} classes)",
        all_java_classes.len()
    );

    // Parse all Rust classes
    let mut all_rust_classes: Vec<ClassMethods> = Vec::new();

    let block_rust = parse_rust_files_unfiltered("sources/SteelMC/steel-core/src/behavior/blocks");
    for mut c in block_rust {
        c.class_type = "block".to_string();
        all_rust_classes.push(c);
    }

    let item_rust = parse_rust_files_unfiltered("sources/SteelMC/steel-core/src/behavior/items");
    for mut c in item_rust {
        c.class_type = "item".to_string();
        all_rust_classes.push(c);
    }

    let entity_rust = parse_rust_files_unfiltered("sources/SteelMC/steel-core/src/entity");
    for mut c in entity_rust {
        c.class_type = "entity".to_string();
        all_rust_classes.push(c);
    }

    // Build Rust lookup maps
    let rust_map: HashMap<String, Vec<String>> = all_rust_classes
        .iter()
        .map(|c| (c.class_name.to_lowercase(), c.methods.clone()))
        .collect();

    // Method mappings: Java method -> Option<Rust method>
    // None means Steel doesn't have this method yet (needs implementation)
    let block_methods: HashMap<&str, Option<&str>> = [
        // Steel has these
        ("getStateForNeighborUpdate", Some("update_shape")),
        ("getPlacementState", Some("get_state_for_placement")),
        ("onBlockAdded", Some("on_place")),
        ("onUseWithItem", Some("use_item_on")),
        ("onUse", Some("use_without_item")),
        ("neighborUpdate", Some("handle_neighbor_changed")),
        ("randomTick", Some("random_tick")),
        ("getPickStack", Some("get_clone_item_stack")),
        ("hasRandomTicks", Some("is_randomly_ticking")),
        ("createBlockEntity", Some("new_block_entity")),
        ("hasComparatorOutput", Some("has_analog_output_signal")),
        ("getComparatorOutput", Some("get_analog_output_signal")),
        ("getFluidState", Some("get_fluid_state")),
        // Steel doesn't have these yet
        ("onStateReplaced", None),
        ("onBlockBreakStart", None),
        ("onBroken", None),
        ("onDestroyedByExplosion", None),
        ("onEntityCollision", None),
        ("onProjectileHit", None),
        ("onSteppedOn", None),
        ("onLandedUpon", None),
        ("scheduledTick", None),
        ("getOutlineShape", None),
        ("getCollisionShape", None),
        ("getRaycastShape", None),
        ("canPlaceAt", None),
        ("canReplace", None),
        ("rotate", None),
        ("mirror", None),
    ]
    .into_iter()
    .collect();

    let item_methods: HashMap<&str, Option<&str>> = [
        // Steel has these
        ("useOnBlock", Some("use_on")),
        // Steel doesn't have these yet
        ("use", None),
        ("usageTick", None),
        ("finishUsing", None),
        ("postHit", None),
        ("postMine", None),
        ("postDamageEntity", None),
        ("useOnEntity", None),
        ("inventoryTick", None),
        ("onStoppedUsing", None),
        ("getMaxUseTime", None),
        ("getMiningSpeed", None),
        ("onCraft", None),
        ("onCraftByPlayer", None),
    ]
    .into_iter()
    .collect();

    let entity_methods: HashMap<&str, Option<&str>> = [
        // Steel has these
        ("tick", Some("tick")),
        ("writeCustomDataToNbt", Some("save_additional")),
        ("readCustomDataFromNbt", Some("load_additional")),
        // Steel doesn't have these yet
        ("damage", None),
        ("onDeath", None),
        ("onKilledOther", None),
        ("interact", None),
        ("interactAt", None),
        ("onSpawnPacket", None),
        ("onStruckByLightning", None),
        ("pushAwayFrom", None),
        ("travel", None),
        ("tickMovement", None),
        ("mobTick", None),
        ("getActiveEyeHeight", None),
    ]
    .into_iter()
    .collect();

    let goal_methods: HashMap<&str, Option<&str>> = [
        // Steel has these (placeholder - Steel doesn't have AI yet)
        ("canStart", None),
        ("shouldContinue", None),
        ("start", None),
        ("stop", None),
        ("tick", None),
        ("canStop", None),
        ("shouldRunEveryTick", None),
    ]
    .into_iter()
    .collect();

    // Warn about methods Steel doesn't have yet
    println!("\n=== Methods Steel Needs to Implement ===");
    let mut missing_methods: Vec<(&str, &str)> = Vec::new();
    for (java_method, rust_method) in &block_methods {
        if rust_method.is_none() {
            missing_methods.push(("Block", java_method));
        }
    }
    for (java_method, rust_method) in &item_methods {
        if rust_method.is_none() {
            missing_methods.push(("Item", java_method));
        }
    }
    for (java_method, rust_method) in &entity_methods {
        if rust_method.is_none() {
            missing_methods.push(("Entity", java_method));
        }
    }
    for (java_method, rust_method) in &goal_methods {
        if rust_method.is_none() {
            missing_methods.push(("Goal", java_method));
        }
    }
    for (type_name, method) in &missing_methods {
        println!("  {}: {} (not in Steel yet)", type_name, method);
    }

    // Analyze each Java class
    let mut tracking: Vec<ClassTracking> = Vec::new();

    for java_class in &all_java_classes {
        if !java_class.is_real_class || java_class.methods.is_empty() {
            continue;
        }

        // Get method mapping based on type
        let method_map: &HashMap<&str, Option<&str>> = match java_class.class_type.as_str() {
            "block" => &block_methods,
            "item" => &item_methods,
            "entity" => &entity_methods,
            "ai_goal" => &goal_methods,
            "ai_brain" => &goal_methods,
            _ => &entity_methods,
        };

        // Only include classes that have at least one tracked method
        let has_tracked_method = java_class
            .methods
            .iter()
            .any(|m| method_map.contains_key(m.as_str()));
        if !has_tracked_method {
            continue;
        }

        // Find corresponding Rust class
        let rust_methods = if java_class.class_type == "block" {
            block_class_map
                .get(&java_class.class_name)
                .and_then(|steel| rust_map.get(&steel.to_lowercase()))
                .cloned()
                .unwrap_or_default()
        } else if java_class.class_type == "item" {
            item_class_map
                .get(&java_class.class_name)
                .and_then(|steel| rust_map.get(&steel.to_lowercase()))
                .cloned()
                .unwrap_or_default()
        } else {
            // Direct name matching for entities/goals
            rust_map
                .get(&java_class.class_name.to_lowercase())
                .cloned()
                .unwrap_or_default()
        };

        // Track method implementation status (only for tracked methods)
        let mut method_tracking = Vec::new();
        for java_method in &java_class.methods {
            // Only track methods that are in our mapping
            if let Some(rust_equiv_opt) = method_map.get(java_method.as_str()) {
                let status = if let Some(rust_method) = rust_equiv_opt {
                    // Steel has this method in its trait
                    if rust_methods.contains(&rust_method.to_string()) {
                        ImplementationStatus::Implemented
                    } else {
                        ImplementationStatus::NotImplemented
                    }
                } else {
                    // Steel doesn't have this method yet
                    ImplementationStatus::NotImplemented
                };

                method_tracking.push(MethodTracking {
                    method_name: java_method.clone(),
                    status,
                });
            }
        }

        let implemented = method_tracking
            .iter()
            .filter(|m| m.status == ImplementationStatus::Implemented)
            .count();
        let total = method_tracking.len();

        tracking.push(ClassTracking {
            class_name: java_class.class_name.clone(),
            class_type: java_class.class_type.clone(),
            methods: method_tracking,
            percentage_implemented: if total > 0 {
                (implemented as f32 / total as f32) * 100.0
            } else {
                0.0
            },
        });
    }

    tracking.sort_by(|a, b| a.class_name.cmp(&b.class_name));

    let result = AnalysisResult { classes: tracking };

    let analysis_json = serde_json::to_string_pretty(&result).unwrap();
    fs::write("outputs/analysis.json", analysis_json).unwrap();
    println!(
        "Wrote outputs/analysis.json ({} classes)",
        result.classes.len()
    );

    // Summary by type
    println!("\n=== Summary by Type ===");
    for type_name in [
        "block",
        "item",
        "entity",
        "ai_goal",
        "ai_brain",
        "ai_control",
        "ai_pathing",
        "other",
    ] {
        let type_classes: Vec<_> = result
            .classes
            .iter()
            .filter(|c| c.class_type == type_name)
            .collect();
        if !type_classes.is_empty() {
            let total_methods: usize = type_classes.iter().map(|c| c.methods.len()).sum();
            let impl_methods: usize = type_classes
                .iter()
                .map(|c| {
                    c.methods
                        .iter()
                        .filter(|m| m.status == ImplementationStatus::Implemented)
                        .count()
                })
                .sum();
            let pct = if total_methods > 0 {
                (impl_methods as f32 / total_methods as f32) * 100.0
            } else {
                0.0
            };
            println!(
                "{}: {} classes, {:.1}% implemented",
                type_name,
                type_classes.len(),
                pct
            );
        }
    }
}

fn detect_entity_subtype(class_name: &str) -> String {
    if class_name.ends_with("Goal") {
        "ai_goal".to_string()
    } else if class_name.ends_with("Task")
        || class_name.ends_with("Sensor")
        || class_name.ends_with("Memory")
    {
        "ai_brain".to_string()
    } else if class_name.ends_with("Control")
        || class_name.ends_with("LookControl")
        || class_name.ends_with("MoveControl")
    {
        "ai_control".to_string()
    } else if class_name.ends_with("Navigation") || class_name.ends_with("PathNodeMaker") {
        "ai_pathing".to_string()
    } else if class_name.ends_with("Entity") {
        "entity".to_string()
    } else {
        "other".to_string()
    }
}
