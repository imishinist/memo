use crate::error::MemoError;
use crate::memo::MemoDocument;
use crate::search::{SearchResult, japanese_tokenizer::JapaneseTokenizer};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use tantivy::TantivyDocument;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::Value;
use tantivy::schema::*;
use tantivy::*;

/// Tantivyベースの検索インデックス
pub struct SearchIndex {
    pub index_dir: PathBuf,
    index: Index,
    writer: IndexWriter,
    reader: IndexReader,
    // フィールド定義
    content_field: Field,
    title_field: Field,
    tags_field: Field,
    path_field: Field,
    created_at_field: Field,
}

impl SearchIndex {
    /// 新しいインデックスを作成
    pub fn create<P: AsRef<Path>>(index_dir: P) -> std::result::Result<Self, MemoError> {
        let index_dir = index_dir.as_ref().to_path_buf();

        // スキーマを定義
        let mut schema_builder = Schema::builder();

        // 日本語対応のテキストフィールドオプション
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("lang_ja") // 日本語トークナイザーを指定
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        // 必須フィールド
        let content_field = schema_builder.add_text_field("content", text_options.clone());

        // オプショナルフィールド
        let title_field = schema_builder.add_text_field("title", text_options);
        let tags_field = schema_builder.add_facet_field("tags", INDEXED);
        let path_field = schema_builder.add_text_field("path", STORED);
        let created_at_field = schema_builder.add_date_field("created_at", INDEXED | STORED);

        let schema = schema_builder.build();

        // インデックスを作成
        let index = Index::create_in_dir(&index_dir, schema)?;

        // 日本語トークナイザーを登録
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
            index_dir,
            index,
            writer,
            reader,
            content_field,
            title_field,
            tags_field,
            path_field,
            created_at_field,
        })
    }

    /// 既存のインデックスを開く
    pub fn open<P: AsRef<Path>>(index_dir: P) -> std::result::Result<Self, MemoError> {
        let index_dir = index_dir.as_ref().to_path_buf();

        let index = Index::open_in_dir(&index_dir)?;

        // 日本語トークナイザーを登録
        let japanese_tokenizer = JapaneseTokenizer::new();
        if !japanese_tokenizer.is_available() {
            eprintln!(
                "Warning: Japanese tokenizer is not available. Falling back to simple tokenization."
            );
        }
        index.tokenizers().register("lang_ja", japanese_tokenizer);

        let schema = index.schema();

        // フィールドを取得
        let content_field = schema.get_field("content")?;
        let title_field = schema.get_field("title")?;
        let tags_field = schema.get_field("tags")?;
        let path_field = schema.get_field("path")?;
        let created_at_field = schema.get_field("created_at")?;

        let writer = index.writer(50_000_000)?;

        let reader = index.reader()?;

        Ok(Self {
            index_dir,
            index,
            writer,
            reader,
            content_field,
            title_field,
            tags_field,
            path_field,
            created_at_field,
        })
    }

    /// メモをインデックスに追加
    pub fn add_memo(&mut self, memo: &MemoDocument) -> std::result::Result<(), MemoError> {
        let mut doc = doc!(
            self.content_field => memo.content.clone(),
            self.path_field => memo.path.clone(),
            self.created_at_field => DateTime::from_timestamp_secs(memo.created_at.timestamp())
        );

        // オプショナルフィールド
        if let Some(frontmatter) = &memo.frontmatter {
            // titleフィールド
            if let Some(title) = frontmatter.get("title").and_then(|v| v.as_str()) {
                doc.add_text(self.title_field, title);
            }

            // tagsフィールド
            if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        doc.add_facet(self.tags_field, Facet::from(&format!("/{}", tag_str)));
                    }
                }
            }

            // metadataフィールド（title, tags以外）
            // TODO: TantivyのJSONフィールドAPIを正しく実装
            /*
            let mut metadata = frontmatter.clone();
            if let JsonValue::Object(ref mut map) = metadata {
                map.remove("title");
                map.remove("tags");

                if !map.is_empty() {
                    doc.add_object(self.metadata_field, metadata);
                }
            }
            */
        }

        self.writer.add_document(doc)?;

        Ok(())
    }

    /// メモをインデックスから削除
    pub fn remove_memo(&mut self, path: &str) -> std::result::Result<(), MemoError> {
        let term = Term::from_field_text(self.path_field, path);
        self.writer.delete_term(term);
        Ok(())
    }

    /// 変更をコミット
    pub fn commit(&mut self) -> std::result::Result<(), MemoError> {
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    /// 検索実行
    pub fn search(&self, query_str: &str) -> std::result::Result<Vec<SearchResult>, MemoError> {
        let searcher = self.reader.searcher();

        // クエリパーサーを作成（metadataフィールドは除外）
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.content_field, self.title_field]);

        // クエリをパース
        let query = query_parser.parse_query(query_str)?;

        // 検索実行
        let top_docs = searcher.search(&query, &TopDocs::with_limit(100))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            // ドキュメントからMemoDocumentを復元
            let content = retrieved_doc
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let path = retrieved_doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let created_at = retrieved_doc
                .get_first(self.created_at_field)
                .and_then(|v| v.as_datetime())
                .and_then(|dt| chrono::DateTime::from_timestamp(dt.into_timestamp_secs(), 0))
                .unwrap_or_else(|| {
                    // デフォルトとしてUnixエポック時刻を使用
                    chrono::DateTime::from_timestamp(0, 0).expect("Unix epoch should be valid")
                });

            // frontmatterを復元（簡略化）
            let mut frontmatter = None;

            // titleを取得
            if let Some(title_value) = retrieved_doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
            {
                let mut fm = serde_json::Map::new();
                fm.insert(
                    "title".to_string(),
                    JsonValue::String(title_value.to_string()),
                );
                frontmatter = Some(JsonValue::Object(fm));
            }

            // metadataは一時的に無効化
            /*
            if let Some(metadata_value) = retrieved_doc.get_first(self.metadata_field).and_then(|v| v.as_json()) {
                if let Some(ref mut fm) = frontmatter {
                    if let (Some(fm_obj), Some(meta_obj)) = (fm.as_object_mut(), metadata_value.as_object()) {
                        for (k, v) in meta_obj {
                            fm_obj.insert(k.clone(), v.clone());
                        }
                    }
                } else {
                    frontmatter = Some(metadata_value.clone());
                }
            }
            */

            let memo = MemoDocument {
                content,
                path,
                created_at,
                frontmatter,
            };

            results.push(SearchResult { memo, score });
        }

        Ok(results)
    }
}
