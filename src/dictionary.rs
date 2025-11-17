use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::models::WordDefinition;
use tantivy::{
    collector::TopDocs,
    query::{QueryParser, TermQuery},
    schema::*,
    Index, Term,
};

// Schema 设计
fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // 主键：单词（索引 + 存储）
    let _word = schema_builder.add_text_field("word", TEXT | STORED);

    // 简明释义（用于结果展示）
    let _concise_definition = schema_builder.add_text_field("concise_definition", STORED);

    // 整个 JSON 内容（点击时反序列化）
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

    // 检查索引是否存在且有效（不自动构建）
    fn check_index_exists(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let index_dir = Path::new(&self.index_path);
        
        if !index_dir.exists() {
            return Ok(false);
        }
        
        // 检查索引是否有效（是否有文档）
        self.is_index_valid()
    }

    // 检查索引是否有效（是否有文档）
    fn is_index_valid(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let index_dir = Path::new(&self.index_path);
        if !index_dir.exists() {
            return Ok(false);
        }

        match Index::open_in_dir(&self.index_path) {
            Ok(index) => {
                let reader = index.reader()?;
                let searcher = reader.searcher();
                Ok(searcher.num_docs() > 0)
            }
            Err(_) => Ok(false),
        }
    }

    // 检查是否需要重建索引（通过比较文件数量和修改时间）
    // 注意：此方法假设索引目录已存在
    fn needs_rebuild(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let words_dir = Path::new(&self.words_directory);
        if !words_dir.exists() {
            return Ok(true);
        }

        // 统计 words 目录中的 JSON 文件数量
        let mut json_count = 0;
        let mut latest_mtime = SystemTime::UNIX_EPOCH;
        
        for entry in fs::read_dir(&self.words_directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|s| s == "json").unwrap_or(false) {
                json_count += 1;
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(mtime) = metadata.modified() {
                        if mtime > latest_mtime {
                            latest_mtime = mtime;
                        }
                    }
                }
            }
        }

        // 检查索引中的文档数量是否与文件数量匹配
        match Index::open_in_dir(&self.index_path) {
            Ok(index) => {
                let reader = index.reader()?;
                let searcher = reader.searcher();
                let indexed_count = searcher.num_docs() as usize;
                
                // 如果文档数量不匹配，需要重建
                if indexed_count != json_count {
                    return Ok(true);
                }

                // 检查索引目录的修改时间
                if let Ok(index_metadata) = fs::metadata(&self.index_path) {
                    if let Ok(index_mtime) = index_metadata.modified() {
                        // 如果 words 目录中有文件比索引更新，需要重建
                        if latest_mtime > index_mtime {
                            return Ok(true);
                        }
                    }
                }
            }
            Err(_) => {
                // 如果无法打开索引，需要重建
                return Ok(true);
            }
        }

        Ok(false)
    }

    // 异步构建索引：扫描 words 目录下的所有 JSON 文件并建立 tantivy 索引
    pub async fn build_index_async(&self) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        // 使用 tokio::task::spawn_blocking 将阻塞的 I/O 操作移到线程池
        let words_dir = self.words_directory.clone();
        let index_path = self.index_path.clone();
        let schema = self.schema.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            // 确保 words 目录存在
            let words_dir_path = Path::new(&words_dir);
            if !words_dir_path.exists() {
                fs::create_dir_all(&words_dir).map_err(|e| format!("Failed to create words directory: {}", e))?;
                println!("Created words directory: {}", words_dir);
            }

            // 如果索引目录已存在，先删除
            if Path::new(&index_path).exists() {
                fs::remove_dir_all(&index_path).map_err(|e| format!("Failed to remove index directory: {}", e))?;
                println!("Removed existing index directory");
            }

            // 确保索引目录的父目录存在
            if let Some(parent) = Path::new(&index_path).parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            // 创建索引目录本身（Index::create_in_dir 需要目录已存在）
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
            
            // 统计words目录中的JSON文件数量
            let mut json_count = 0;
            
            // 遍历 words 目录下的所有 JSON 文件
            for entry in fs::read_dir(&words_dir).map_err(|e| format!("Failed to read words directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.extension().map(|s| s == "json").unwrap_or(false) {
                    json_count += 1;
                    match fs::read_to_string(&path) {
                        Ok(data) => {
                            // 解析 JSON 以获取单词和简明释义
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
        
        // 清空缓存的索引，强制重新加载
        let mut index_guard = self.index.write().unwrap();
        *index_guard = None;
        drop(index_guard);
        
        // 重新加载索引
        self.ensure_index_loaded()?;
        
        Ok((indexed_count, json_count))
    }

   // 确保 index 已加载（不自动构建）
    fn ensure_index_loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut index_guard = self.index.write().unwrap();
        
        if index_guard.is_none() {
            // 检查索引目录是否存在
            let index_dir = Path::new(&self.index_path);
            if !index_dir.exists() {
                drop(index_guard);
                return Err(format!("索引不存在，请先在设置中构建索引: {}", self.index_path).into());
            }
            
            // 尝试打开索引，如果失败则返回错误
            match Index::open_in_dir(&self.index_path) {
                Ok(index) => {
                    *index_guard = Some(index);
                }
                Err(e) => {
                    drop(index_guard);
                    return Err(format!("无法打开索引 {}: {}。请重新构建索引。", self.index_path, e).into());
                }
            }
        }
        
        Ok(())
    }

    // 获取 index（内部使用）
    fn get_index(&self) -> Result<Arc<Index>, Box<dyn std::error::Error>> {
        self.ensure_index_loaded()?;
        let index_guard = self.index.read().unwrap();
        index_guard
            .as_ref()
            .ok_or("索引未初始化".into())
            .map(|idx| Arc::new(idx.clone()))
    }

    // 查找单词（精确匹配）
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

    // 列出所有单词（用于自动完成）
    pub fn list_words(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let schema = searcher.schema();
        let word_field = schema.get_field("word")?;

        // 使用 QueryParser 进行前缀查询（使用通配符 "*" 匹配所有）
        let query_parser = QueryParser::for_index(&index, vec![word_field]);
        let query = query_parser.parse_query("*")?;
        
        // 获取所有文档（设置一个很大的限制）
        let top_docs = searcher.search(&query, &TopDocs::with_limit(100_000))?;

        let mut words = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            if let Some(word_val) = retrieved_doc.get_first(word_field) {
                if let Some(word_str) = word_val.as_str() {
                    words.push(word_str.to_string());
                }
            }
        }
        
        words.sort();
        Ok(words)
    }

    // 前缀搜索（模糊查询，用于更高效的自动完成）
    pub fn search_words(&self, prefix: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let index = self.get_index()?;
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let schema = searcher.schema();

        let word_field = schema.get_field("word")?;
        let concise_field = schema.get_field("concise_definition")?;

        // 使用 QueryParser 进行前缀查询
        let query_parser = QueryParser::for_index(&index, vec![word_field]);
        let query_str = format!("{}*", prefix.to_lowercase());
        let query = query_parser.parse_query(&query_str)?;
        
        let top_docs = searcher.search(&query, &TopDocs::with_limit(20))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            let word = retrieved_doc
                .get_first(word_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let concise = retrieved_doc
                .get_first(concise_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            results.push((word, concise));
        }
        
        Ok(results)
    }
}
