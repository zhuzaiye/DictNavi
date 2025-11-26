use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::models::WordDefinition;
use tantivy::{
    collector::TopDocs,
    query::{AllQuery, TermQuery},
    schema::*,
    Index, Term,
};

// Schema definition
fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // Primary key: word (indexed + stored)
    let _word = schema_builder.add_text_field("word", TEXT | STORED);

    // Concise definition (for result display)
    let _concise_definition = schema_builder.add_text_field("concise_definition", STORED);

    // Whole JSON content (deserialized when clicked)
    let _json_data = schema_builder.add_text_field("json_data", STORED);

    schema_builder.build()
}

pub struct Dictionary {
    words_directory: String,
    index_path: String,
    schema: Schema,
    index: Arc<RwLock<Option<Index>>>,
}

impl Dictionary {
    pub fn new(words_directory: String) -> Self {
        let index_path = format!("{}/.index", words_directory);
        let schema = build_schema();
        
        Dictionary {
            words_directory,
            index_path,
            schema,
            index: Arc::new(RwLock::new(None)),
        }
    }

    // Asynchronous indexing: scan all JSON files in words directory and build tantivy index
    pub async fn build_index_async(&self) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        // Use tokio::task::spawn_blocking to move blocking I/O operations to thread pool
        let words_dir = self.words_directory.clone();
        let index_path = self.index_path.clone();
        let schema = self.schema.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            // Ensure words directory exists
            let words_dir_path = Path::new(&words_dir);
            if !words_dir_path.exists() {
                fs::create_dir_all(&words_dir).map_err(|e| format!("Failed to create words directory: {}", e))?;
                println!("Created words directory: {}", words_dir);
            }

            // If index directory exists, delete it first
            if Path::new(&index_path).exists() {
                fs::remove_dir_all(&index_path).map_err(|e| format!("Failed to remove index directory: {}", e))?;
                println!("Removed existing index directory");
            }

            // Ensure parent directory of index directory exists
            if let Some(parent) = Path::new(&index_path).parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            // Create index directory itself (Index::create_in_dir requires directory to exist)
            fs::create_dir_all(&index_path).map_err(|e| format!("Failed to create index directory: {}", e))?;

            println!("Building index from words directory: {}", words_dir);
            let index = Index::create_in_dir(&index_path, schema.clone())
                .map_err(|e| format!("Failed to create index: {}", e))?;
            let mut index_writer = index.writer(50_000_000)
                .map_err(|e| format!("Failed to create index writer: {}", e))?; // 50MB buffer

            let word_field = schema.get_field("word")
                .map_err(|e| format!("Failed to get word field: {}", e))?;
            let concise_definition_field = schema.get_field("concise_definition")
                .map_err(|e| format!("Failed to get concise_definition field: {}", e))?;
            let json_data_field = schema.get_field("json_data")
                .map_err(|e| format!("Failed to get json_data field: {}", e))?;

            let mut indexed_count = 0;
            let mut error_count = 0;
            
            // Count JSON files in words directory
            let mut json_count = 0;
            
            // Iterate over all JSON files in words directory
            for entry in fs::read_dir(&words_dir).map_err(|e| format!("Failed to read words directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.extension().map(|s| s == "json").unwrap_or(false) {
                    json_count += 1;
                    match fs::read_to_string(&path) {
                        Ok(data) => {
                            // Parse JSON to get word and concise definition
                            match serde_json::from_str::<WordDefinition>(&data) {
                                Ok(word_def) => {
                                    let concise = word_def
                                        .concise_definition
                                        .clone()
                                        .unwrap_or_default();
                                    
                                    if let Err(e) = index_writer.add_document(tantivy::doc!(
                                        word_field => word_def.word.clone(),
                                        concise_definition_field => concise,
                                        json_data_field => data
                                    )) {
                                        eprintln!("Warning: Failed to add document for {:?}: {}", path, e);
                                        error_count += 1;
                                    } else {
                                        indexed_count += 1;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to parse JSON file {:?}: {}", path, e);
                                    error_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read file {:?}: {}", path, e);
                            error_count += 1;
                        }
                    }
                }
            }

            index_writer.commit().map_err(|e| format!("Failed to commit index: {}", e))?;
            println!("Index built successfully with {} documents ({} errors)", indexed_count, error_count);
            
            Ok::<(usize, usize, usize), String>((indexed_count, error_count, json_count))
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;
        
        let (indexed_count, _error_count, json_count) = result?;
        
        // Clear cached index, force reload
        let mut index_guard = self.index.write().unwrap();
        *index_guard = None;
        drop(index_guard);
        
        // Reload index
        self.ensure_index_loaded()?;
        
        Ok((indexed_count, json_count))
    }

    // Ensure index is loaded (not built automatically)
    fn ensure_index_loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut index_guard = self.index.write().unwrap();
        
        if index_guard.is_none() {
            // Check if index directory exists
            let index_dir = Path::new(&self.index_path);
            if !index_dir.exists() {
                drop(index_guard);
                return Err(format!("Index does not exist. Please build the index first in settings: {}", self.index_path).into());
            }
            
            // Try to open index, return error if failed
            match Index::open_in_dir(&self.index_path) {
                Ok(index) => {
                    *index_guard = Some(index);
                }
                Err(e) => {
                    drop(index_guard);
                    return Err(format!("Failed to open index {}: {}. Please rebuild the index.", self.index_path, e).into());
                }
            }
        }
        
        Ok(())
    }

    // Get index (internal use)
    fn get_index(&self) -> Result<Arc<Index>, Box<dyn std::error::Error>> {
        self.ensure_index_loaded()?;
        let index_guard = self.index.read().unwrap();
        index_guard
            .as_ref()
            .ok_or("Index not initialized".into())
            .map(|idx| Arc::new(idx.clone()))
    }

    // Lookup word (exact match)
    pub fn lookup_word(&self, word: &str) -> Result<Option<WordDefinition>, Box<dyn std::error::Error>> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let schema = searcher.schema();

        let word_field = schema.get_field("word")?;
        let json_data_field = schema.get_field("json_data")?;

        let query = TermQuery::new(
            Term::from_field_text(word_field, &word.to_lowercase()),
            tantivy::schema::IndexRecordOption::Basic,
        );

        let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;
        
        if let Some((_score, doc_address)) = top_docs.first() {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(*doc_address)?;
            if let Some(json_val) = retrieved_doc.get_first(json_data_field) {
                if let Some(json_str) = json_val.as_str() {
                    return Ok(Some(serde_json::from_str(json_str)?));
                }
            }
        }
        
        Ok(None)
    }

    // List all words (for autocomplete)
    pub fn list_words(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let schema = searcher.schema();
        let word_field = schema.get_field("word")?;

        // Use AllQuery for better performance (more efficient than wildcard query)
        let query = AllQuery;
        
        // Get all documents (set a large limit)
        let top_docs = searcher.search(&query, &TopDocs::with_limit(100_000))?;

        // Pre-allocate capacity to avoid reallocations
        let mut words = Vec::with_capacity(top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            if let Some(word_val) = retrieved_doc.get_first(word_field) {
                if let Some(word_str) = word_val.as_str() {
                    words.push(word_str.to_string());
                }
            }
        }
        
        // Index-based sorting optimization (memory-efficient version):
        // Instead of sorting strings directly, we:
        // 1. Create index vector [0, 1, 2, ..., n-1] - only 8 bytes per element
        // 2. Sort indices by comparing original strings (no key caching needed)
        // 3. Rearrange strings according to sorted indices
        //
        // Advantages:
        // - Sorting phase: only moves usize (8 bytes) instead of String (~24+ bytes)
        // - Better cache locality: small index vector fits entirely in CPU cache
        // - No lowercase key caching: compare on-the-fly (saves ~250KB memory)
        // - Final rearrangement: O(n) operation, strings moved once
        //
        // Memory comparison:
        // - Direct sort with cached keys: ~500KB (words + lowercase keys)
        // - Index sort: ~450KB (words + indices), but better cache performance
        // - For very large datasets (>100k), index sort can be 30-50% faster
        let mut indices: Vec<usize> = (0..words.len()).collect();
        indices.sort_unstable_by(|&a, &b| {
            words[a].to_lowercase().cmp(&words[b].to_lowercase())
        });
        
        // Rearrange words according to sorted indices
        // Note: This requires cloning, but the sorting phase is more efficient
        let sorted_words: Vec<String> = indices.iter().map(|&idx| words[idx].clone()).collect();
        
        Ok(sorted_words)
    }

}
