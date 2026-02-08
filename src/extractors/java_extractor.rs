use crate::{ClassInfo, ClassMethods};
use std::collections::HashMap;
use tree_sitter::{Parser, StreamingIterator};

pub fn parse_java_files_unfiltered(path: &str) -> Vec<ClassMethods> {
    let mut parser = Parser::new();

    let language = tree_sitter_java::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading Java grammar");

    let query_str = r#"
       (class_declaration
         name: (identifier) @class_name
         (superclass (type_identifier) @extends)?
         (super_interfaces (type_list (type_identifier) @implements))*
         body: (class_body
           (method_declaration
             name: (identifier) @method_name)))
    "#;

    let query = tree_sitter::Query::new(&language.into(), query_str).unwrap();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut class_info_map = HashMap::<String, ClassInfo>::new();

    crate::utils::walk_files(path, "java", |_path, content| {
        let tree = parser.parse(&content, None).unwrap();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        while let Some(m) = matches.next() {
            let mut class_name = String::new();
            let mut method_name = String::new();
            let mut extends = None;
            let mut implements = Vec::new();

            for capture in m.captures {
                let text = capture.node.utf8_text(content.as_bytes()).unwrap();
                match capture.index {
                    0 => class_name = text.to_string(),
                    1 => extends = Some(text.to_string()),
                    2 => implements.push(text.to_string()),
                    3 => method_name = text.to_string(),
                    _ => {}
                }
            }

            if !class_name.is_empty() {
                let class_info =
                    class_info_map
                        .entry(class_name.clone())
                        .or_insert_with(|| ClassInfo {
                            name: class_name.clone(),
                            methods: Vec::new(),
                            extends: extends.clone(),
                            implements: implements.clone(),
                        });
                if extends.is_some() {
                    class_info.extends = extends;
                }
                if !implements.is_empty() {
                    for interface in implements {
                        if !class_info.implements.contains(&interface) {
                            class_info.implements.push(interface);
                        }
                    }
                }

                if !method_name.is_empty() && !class_info.methods.contains(&method_name) {
                    class_info.methods.push(method_name);
                }
            }
        }
    });

    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for class_info in class_info_map.values() {
        if let Some(parent) = &class_info.extends {
            children_map
                .entry(parent.clone())
                .or_default()
                .push(class_info.name.clone());
        }
        for interface in &class_info.implements {
            children_map
                .entry(interface.clone())
                .or_default()
                .push(class_info.name.clone());
        }
    }

    class_info_map
        .into_iter()
        .map(|(class_name, class_info)| {
            let is_real_class = !children_map.contains_key(&class_name);
            ClassMethods {
                class_name,
                class_type: String::new(),
                methods: class_info.methods,
                is_real_class,
            }
        })
        .collect()
}
