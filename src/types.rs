use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMethods {
    pub class_name: String,
    pub class_type: String,
    pub methods: Vec<String>,
    pub is_real_class: bool,
}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImplementationStatus {
    Implemented,
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodTracking {
    pub method_name: String,
    pub status: ImplementationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassTracking {
    pub class_name: String,
    pub class_type: String,
    pub methods: Vec<MethodTracking>,
    pub percentage_implemented: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub classes: Vec<ClassTracking>,
}
