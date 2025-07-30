use crate::error::MemoError;
use lindera::dictionary::{DictionaryKind, load_dictionary_from_kind};
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer as LinderaTokenizer;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer as TantivyTokenizer};

/// Linderaを使った日本語トークナイザー
#[derive(Clone)]
pub struct JapaneseTokenizer {
    tokenizer: Option<LinderaTokenizer>,
}

impl Default for JapaneseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl JapaneseTokenizer {
    pub fn new() -> Self {
        let tokenizer = Self::create_tokenizer().ok();
        Self { tokenizer }
    }

    /// トークナイザーを作成（エラーハンドリング付き）
    fn create_tokenizer() -> Result<LinderaTokenizer, MemoError> {
        let dict = load_dictionary_from_kind(DictionaryKind::IPADIC)
            .map_err(|e| MemoError::Tokenizer(format!("Failed to load dictionary: {}", e)))?;

        let segmenter = Segmenter::new(Mode::Normal, dict, None);
        Ok(LinderaTokenizer::new(segmenter))
    }

    /// トークナイザーが利用可能かチェック
    pub fn is_available(&self) -> bool {
        self.tokenizer.is_some()
    }
}

impl TantivyTokenizer for JapaneseTokenizer {
    type TokenStream<'a> = JapaneseTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        JapaneseTokenStream::new(text, self.tokenizer.as_mut())
    }
}

/// Linderaを使った日本語トークンストリーム
pub struct JapaneseTokenStream<'a> {
    tokens: Vec<Token>,
    index: usize,
    _text: &'a str,
}

impl<'a> JapaneseTokenStream<'a> {
    fn new(text: &'a str, tokenizer: Option<&mut LinderaTokenizer>) -> Self {
        let mut tokens = Vec::new();
        let mut position = 0;

        // トークナイザーが利用可能な場合のみLinderaでトークン化
        if let Some(tokenizer) = tokenizer {
            match tokenizer.tokenize(text) {
                Ok(lindera_tokens) => {
                    for lindera_token in lindera_tokens {
                        let surface = lindera_token.text;
                        let features = lindera_token.details.as_ref();

                        // 意味のあるトークンのみを追加
                        if should_include_token(&surface, features) {
                            let mut token = Token::default();

                            // 基本形があれば基本形を、なければ表層形を使用
                            token.text = if let Some(features_vec) = features {
                                if features_vec.len() > 6
                                    && !features_vec[6].is_empty()
                                    && features_vec[6] != "*"
                                {
                                    features_vec[6].to_lowercase()
                                } else {
                                    surface.to_lowercase()
                                }
                            } else {
                                surface.to_lowercase()
                            };

                            // オフセット情報を設定
                            token.offset_from = lindera_token.byte_start;
                            token.offset_to = lindera_token.byte_end;
                            token.position = position;

                            tokens.push(token);
                            position += 1;
                        }
                    }
                }
                Err(_) => {
                    // トークン化に失敗した場合は、単純な空白分割にフォールバック
                    Self::fallback_tokenize(text, &mut tokens, &mut position);
                }
            }
        } else {
            // トークナイザーが利用できない場合は、単純な空白分割にフォールバック
            Self::fallback_tokenize(text, &mut tokens, &mut position);
        }

        Self {
            tokens,
            index: 0,
            _text: text,
        }
    }

    /// フォールバック用の単純なトークン化
    fn fallback_tokenize(text: &str, tokens: &mut Vec<Token>, position: &mut usize) {
        let mut byte_offset = 0;

        for word in text.split_whitespace() {
            // 空白をスキップして実際の単語の開始位置を見つける
            while byte_offset < text.len() && text.as_bytes()[byte_offset].is_ascii_whitespace() {
                byte_offset += 1;
            }

            if !word.is_empty() && should_include_simple_token(word) {
                let mut token = Token::default();
                token.text = word.to_lowercase();
                token.offset_from = byte_offset;
                token.offset_to = byte_offset + word.len();
                token.position = *position;

                tokens.push(token);
                *position += 1;
            }

            byte_offset += word.len();
        }
    }
}

impl<'a> TokenStream for JapaneseTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

fn should_include_token(surface: &str, features: Option<&Vec<std::borrow::Cow<str>>>) -> bool {
    // 空文字や空白のみは除外
    if surface.trim().is_empty() {
        return false;
    }

    // 品詞情報を確認（features[0]が品詞）
    if let Some(features_vec) = features {
        if !features_vec.is_empty() {
            let pos = &features_vec[0];

            // 記号、助詞、助動詞は除外
            if pos.starts_with("記号") || pos.starts_with("助詞") || pos.starts_with("助動詞")
            {
                return false;
            }

            // 一文字の名詞（「の」「が」など）は除外
            if pos.starts_with("名詞") && surface.chars().count() == 1 {
                return false;
            }
        }
    }

    // 記号のみは除外
    if surface.chars().all(|c| {
        c.is_ascii_punctuation()
            || matches!(
                c,
                '、' | '。' | '！' | '？' | '（' | '）' | '「' | '」' | '・'
            )
    }) {
        return false;
    }

    // 日本語文字を含む場合は含める
    if surface.chars().any(|c| {
        (c >= 'あ' && c <= 'ん') ||  // ひらがな
        (c >= 'ア' && c <= 'ン') ||  // カタカナ
        (c >= '一' && c <= '龯') // 漢字
    }) {
        return true;
    }

    // 英数字の場合、2文字以上なら含める
    surface.len() >= 2
}

/// フォールバック用の単純なトークン判定
fn should_include_simple_token(word: &str) -> bool {
    // 空文字や空白のみは除外
    if word.trim().is_empty() {
        return false;
    }

    // 記号のみは除外
    if word.chars().all(|c| c.is_ascii_punctuation()) {
        return false;
    }

    // 1文字以上なら含める（フォールバック時は緩い条件）
    !word.is_empty()
}
