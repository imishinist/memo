use std::collections::BTreeMap;

use crate::error::MemoError;
use crate::memo::{MemoDocument, MemoFile};
use crate::search::{SearchResult, japanese_tokenizer::JapaneseTokenizer};

use tantivy::TantivyDocument;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::schema::*;
use tantivy::*;

use std::path::{Path, PathBuf};

/// Tantivy-based search index for memo documents
pub struct SearchIndex {
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    pub index_dir: PathBuf,
    index: Index,
    writer: IndexWriter,
    reader: IndexReader,

    // fields
    id_field: Field,
    path_field: Field,

    content_field: Field,
    title_field: Field,
    tags_field: Field,
    tags_facet_field: Field,
    created_at_field: Field,

    metadata_field: Field,
}

impl SearchIndex {
    pub fn create<P: AsRef<Path>>(
        data_dir: P,
        index_dir: P,
    ) -> std::result::Result<Self, MemoError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let index_dir = index_dir.as_ref().to_path_buf();

        // japanese text options
        let ja_fi = TextFieldIndexing::default()
            .set_tokenizer("lang_ja") // 日本語トークナイザーを指定
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(ja_fi.clone())
            .set_stored();

        // schema building
        let mut schema_builder = Schema::builder();
        // required fields
        let id_field = schema_builder.add_text_field("id", TEXT | STORED);
        let path_field = schema_builder.add_text_field("path", STORED);
        let content_field = schema_builder.add_text_field("content", text_options.clone());

        // optional fields
        let title_field = schema_builder.add_text_field("title", text_options.clone());
        let tags_field = schema_builder.add_text_field("tags", text_options.clone());
        let tags_facet_field = schema_builder.add_facet_field("tags.facet", INDEXED);
        let created_at_field = schema_builder.add_date_field("created_at", INDEXED | STORED);

        let json_options = JsonObjectOptions::default()
            .set_stored()
            .set_expand_dots_enabled()
            .set_indexing_options(ja_fi.clone());
        let metadata_field = schema_builder.add_json_field("metadata", json_options);
        let schema = schema_builder.build();

        let index = Index::create_in_dir(&index_dir, schema)?;
        let japanese_tokenizer = JapaneseTokenizer::new();
        if !japanese_tokenizer.is_available() {
            eprintln!(
                "Warning: Japanese tokenizer is not available. Falling back to simple tokenization."
            );
        }
        index.tokenizers().register("lang_ja", japanese_tokenizer);

        let writer = index.writer(50_000_000)?;
        let reader = index.reader()?;
        Ok(Self {
            data_dir,
            index_dir,
            index,
            writer,
            reader,
            id_field,
            path_field,
            content_field,
            title_field,
            tags_field,
            tags_facet_field,
            created_at_field,
            metadata_field,
        })
    }

    pub fn open<P: AsRef<Path>>(data_dir: P, index_dir: P) -> std::result::Result<Self, MemoError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let index_dir = index_dir.as_ref().to_path_buf();

        let index = Index::open_in_dir(&index_dir)?;
        let japanese_tokenizer = JapaneseTokenizer::new();
        if !japanese_tokenizer.is_available() {
            eprintln!(
                "Warning: Japanese tokenizer is not available. Falling back to simple tokenization."
            );
        }
        index.tokenizers().register("lang_ja", japanese_tokenizer);

        let schema = index.schema();
        let id_field = schema.get_field("id")?;
        let path_field = schema.get_field("path")?;
        let content_field = schema.get_field("content")?;
        let title_field = schema.get_field("title")?;
        let tags_field = schema.get_field("tags")?;
        let tags_facet_field = schema.get_field("tags.facet")?;
        let created_at_field = schema.get_field("created_at")?;
        let metadata_field = schema.get_field("metadata")?;

        let writer = index.writer(50_000_000)?;
        let reader = index.reader()?;

        Ok(Self {
            data_dir,
            index_dir,
            index,
            writer,
            reader,
            id_field,
            path_field,
            content_field,
            title_field,
            tags_field,
            tags_facet_field,
            created_at_field,
            metadata_field,
        })
    }

    pub fn add_memo(&mut self, memo: &MemoDocument) -> std::result::Result<(), MemoError> {
        let mut doc = doc!(
            self.id_field => memo.id.to_string(),
            self.path_field => memo.path.clone(),
            self.content_field => memo.content.clone(),
            self.created_at_field => DateTime::from_timestamp_secs(memo.created_at.timestamp())
        );

        // optional fields
        if let Some(front_matter) = &memo.metadata {
            // title
            if let Some(title) = front_matter.get("title").and_then(|v| v.as_str()) {
                doc.add_text(self.title_field, title);
            }

            // tags
            if let Some(tags) = front_matter.get("tags").and_then(|v| v.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        doc.add_text(self.tags_field, tag_str);
                        doc.add_facet(self.tags_facet_field, Facet::from(&format!("/{}", tag_str)));
                    }
                }
            }

            doc.add_object(self.metadata_field, convert_map(front_matter.clone()));
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    pub fn remove_memo(&mut self, memo: &MemoDocument) -> std::result::Result<(), MemoError> {
        let term = Term::from_field_text(self.id_field, &memo.id.as_str());
        self.writer.delete_term(term);
        Ok(())
    }

    pub fn commit(&mut self) -> std::result::Result<(), MemoError> {
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str) -> std::result::Result<Vec<SearchResult>, MemoError> {
        let searcher = self.reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.content_field, self.title_field]);
        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(100))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let path = retrieved_doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // get real data from the path
            let memo = MemoFile::from_path(&path)?;
            let memo = MemoDocument::from_memo_file(&memo);
            results.push(SearchResult { memo, score });
        }

        Ok(results)
    }
}

fn convert_map(value: serde_json::Value) -> BTreeMap<String, OwnedValue> {
    use serde_json::Value;

    let mut tmp = BTreeMap::new();
    match value {
        Value::Object(t) => {
            for (k, v) in t.into_iter() {
                tmp.insert(k.clone(), convert_value(v));
            }
        }
        _ => {}
    }
    tmp
}

fn convert_value(value: serde_json::Value) -> OwnedValue {
    use serde_json::Value;
    match value {
        Value::Null => OwnedValue::Null,
        Value::Bool(b) => OwnedValue::from(b),
        Value::Number(n) if n.is_u64() => OwnedValue::U64(n.as_u64().unwrap()),
        Value::Number(n) if n.is_i64() => OwnedValue::I64(n.as_i64().unwrap()),
        Value::Number(n) if n.is_f64() => OwnedValue::F64(n.as_f64().unwrap()),
        Value::Number(n) => OwnedValue::Str(n.to_string()),
        Value::String(s) => OwnedValue::from(s.to_string()),
        Value::Array(values) => OwnedValue::Array(values.into_iter().map(convert_value).collect()),
        Value::Object(values) => OwnedValue::Object(
            values
                .into_iter()
                .map(|(k, v)| (k, convert_value(v)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    macro_rules! btree_map {
        ($($key:expr => $value:expr),* $(,)?) => {
            {
                let mut map = BTreeMap::new();
                $(map.insert($key.to_string(), $value);)*
                map
            }
        };
    }

    #[test]
    fn test_convert_value_basic_types() {
        // null
        assert_eq!(convert_value(json!(null)), OwnedValue::Null);

        // boolean
        assert_eq!(convert_value(json!(true)), OwnedValue::Bool(true));
        assert_eq!(convert_value(json!(false)), OwnedValue::Bool(false));

        // string
        assert_eq!(
            convert_value(json!("test string")),
            OwnedValue::Str("test string".to_string())
        );

        // numbers
        assert_eq!(convert_value(json!(42u64)), OwnedValue::U64(42));
        assert_eq!(convert_value(json!(-42i64)), OwnedValue::I64(-42));
        assert!(
            matches!(convert_value(json!(3.14f64)), OwnedValue::F64(f) if (f - 3.14).abs() < f64::EPSILON)
        );

        // large numbers
        assert!(
            matches!(convert_value(json!(1.23e20)), OwnedValue::F64(_))
                || matches!(convert_value(json!(1.23e20)), OwnedValue::Str(_))
        );
    }

    #[test]
    fn test_convert_value_arrays() {
        assert!(matches!(convert_value(json!([])), OwnedValue::Array(arr) if arr.is_empty()));

        assert_eq!(
            convert_value(json!([1, 2, 3])),
            OwnedValue::Array(vec![
                OwnedValue::U64(1),
                OwnedValue::U64(2),
                OwnedValue::U64(3),
            ])
        );

        assert_eq!(
            convert_value(json!([1, "test", true, null])),
            OwnedValue::Array(vec![
                OwnedValue::U64(1),
                OwnedValue::Str("test".to_string()),
                OwnedValue::Bool(true),
                OwnedValue::Null,
            ])
        );

        assert_eq!(
            convert_value(json!([[1, 2], [3, 4]])),
            OwnedValue::Array(vec![
                OwnedValue::Array(vec![OwnedValue::U64(1), OwnedValue::U64(2)]),
                OwnedValue::Array(vec![OwnedValue::U64(3), OwnedValue::U64(4)]),
            ])
        );
    }

    #[test]
    fn test_convert_value_nested_objects() {
        // Simple object
        assert_eq!(
            convert_map(json!({
                "name": "test",
                "value": 42
            })),
            btree_map! {
                "name" => OwnedValue::Str("test".to_string()),
                "value" => OwnedValue::U64(42),
            }
        );

        assert_eq!(
            convert_map(json!({
                "user": {
                    "name": "John",
                    "age": 30,
                    "active": true
                },
                "tags": ["tag1", "tag2"]
            })),
            btree_map! {
                "user" => OwnedValue::Object(btree_map !{
                    "name" => OwnedValue::Str("John".to_string()),
                    "age" => OwnedValue::U64(30),
                    "active" => OwnedValue::Bool(true),
                }.into_iter().collect()),
                "tags" => OwnedValue::Array(vec![
                    OwnedValue::Str("tag1".to_string()),
                    OwnedValue::Str("tag2".to_string()),
                ]),
            }
        );

        // complex nested structure
        assert_eq!(
            convert_map(json!({
                "title": "Test Memo",
                "tags": ["@important", "@work"],
                "priority": 1,
                "created_at": "2025-01-30T15:15:45Z",
                "author": {
                    "name": "John Doe",
                    "email": "john@example.com"
                },
                "settings": {
                    "public": false,
                    "archived": false,
                    "categories": ["personal", "notes"]
                }
            })),
            btree_map! {
                "title" => OwnedValue::Str("Test Memo".to_string()),
                "tags" => OwnedValue::Array(vec![
                    OwnedValue::Str("@important".to_string()),
                    OwnedValue::Str("@work".to_string()),
                ]),
                "priority" => OwnedValue::U64(1),
                "created_at" => OwnedValue::Str("2025-01-30T15:15:45Z".to_string()),
                "author" => OwnedValue::Object(btree_map! {
                    "name" => OwnedValue::Str("John Doe".to_string()),
                    "email" => OwnedValue::Str("john@example.com".to_string()),
                }.into_iter().collect()),
                "settings" => OwnedValue::Object(btree_map! {
                    "public" => OwnedValue::Bool(false),
                    "archived" => OwnedValue::Bool(false),
                    "categories" => OwnedValue::Array(vec![
                        OwnedValue::Str("personal".to_string()),
                        OwnedValue::Str("notes".to_string()),
                    ]),
                }.into_iter().collect()),
            }
        );
    }

    #[test]
    fn test_convert_map_empty_and_null() {
        // Empty object
        assert!(convert_map(json!({})).is_empty());

        // Object with null values
        let object_with_nulls = json!({
            "null_field": null,
            "string_field": "test"
        });
        let converted = convert_map(object_with_nulls);
        assert_eq!(converted.len(), 2);
        assert!(matches!(
            converted.get("null_field").unwrap(),
            OwnedValue::Null
        ));
        assert!(matches!(
            converted.get("string_field").unwrap(),
            OwnedValue::Str(_)
        ));

        // Non-object value (should return empty map)
        let non_object = json!("not an object");
        let converted = convert_map(non_object);
        assert!(converted.is_empty());

        let array_value = json!([1, 2, 3]);
        let converted = convert_map(array_value);
        assert!(converted.is_empty());
    }
}
