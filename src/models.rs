use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordDefinition {
    pub word: String,
    #[serde(rename = "pronunciation")]
    pub phonetic: Option<String>,
    #[serde(default)]
    pub forms: Option<HashMap<String, Value>>,
    #[serde(rename = "concise_definition")]
    pub concise_definition: Option<String>,
    #[serde(rename = "definitions", default)]
    pub meanings: Option<Vec<Meaning>>,
    #[serde(rename = "comparison", default)]
    pub comparisons: Option<Vec<Comparison>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meaning {
    #[serde(rename = "pos")]
    pub part_of_speech: String,
    pub explanation_en: String,
    pub explanation_cn: Option<String>,
    #[serde(rename = "example_en")]
    pub example_en: Option<String>,
    #[serde(rename = "example_cn")]
    pub example_cn: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comparison {
    #[serde(rename = "word_to_compare")]
    pub word: String,
    pub analysis: Option<String>,
}
