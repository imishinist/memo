use tantivy::tokenizer::{Token, TokenStream, Tokenizer as TantivyTokenizer};

/// シンプルなUnicodeベース日本語トークナイザー
#[derive(Clone)]
pub struct JapaneseTokenizer;

impl Default for JapaneseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl JapaneseTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl TantivyTokenizer for JapaneseTokenizer {
    type TokenStream<'a> = JapaneseTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        JapaneseTokenStream::new(text)
    }
}

/// 日本語トークンストリーム
pub struct JapaneseTokenStream<'a> {
    tokens: Vec<Token>,
    index: usize,
    _text: &'a str,
}

impl<'a> JapaneseTokenStream<'a> {
    fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut position = 0;

        // 文字単位でトークン化（日本語対応）
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let mut token_text = String::new();
            let start_byte = text.char_indices().nth(i).map(|(pos, _)| pos).unwrap_or(0);
            let mut end_byte = start_byte;

            // 連続する同じ種類の文字をグループ化
            let char_type = get_char_type(chars[i]);
            let mut j = i;

            while j < chars.len() && get_char_type(chars[j]) == char_type {
                token_text.push(chars[j]);
                end_byte = text
                    .char_indices()
                    .nth(j + 1)
                    .map(|(pos, _)| pos)
                    .unwrap_or(text.len());
                j += 1;
            }

            // 意味のあるトークンのみを追加
            if should_include_token(&token_text) {
                let mut token = Token::default();
                token.text = token_text.to_lowercase();
                token.offset_from = start_byte;
                token.offset_to = end_byte;
                token.position = position;

                tokens.push(token);
                position += 1;
            }

            i = j;
        }

        Self {
            tokens,
            index: 0,
            _text: text,
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

#[derive(PartialEq)]
enum CharType {
    Hiragana,
    Katakana,
    Kanji,
    Latin,
    Number,
    Punctuation,
    Whitespace,
}

fn get_char_type(ch: char) -> CharType {
    if ch.is_whitespace() {
        CharType::Whitespace
    } else if ch >= 'あ' && ch <= 'ん' {
        CharType::Hiragana
    } else if ch >= 'ア' && ch <= 'ン' {
        CharType::Katakana
    } else if ch >= '一' && ch <= '龯' {
        CharType::Kanji
    } else if ch.is_ascii_alphabetic() {
        CharType::Latin
    } else if ch.is_ascii_digit() {
        CharType::Number
    } else {
        CharType::Punctuation
    }
}

fn should_include_token(text: &str) -> bool {
    // 空文字や空白のみは除外
    if text.trim().is_empty() {
        return false;
    }

    // 記号のみは除外
    if text.chars().all(|c| {
        c.is_ascii_punctuation()
            || matches!(c, '、' | '。' | '！' | '？' | '（' | '）' | '「' | '」')
    }) {
        return false;
    }

    // 日本語文字を含む場合は含める
    if text.chars().any(|c| {
        (c >= 'あ' && c <= 'ん') ||  // ひらがな
        (c >= 'ア' && c <= 'ン') ||  // カタカナ
        (c >= '一' && c <= '龯') // 漢字
    }) {
        return true;
    }

    // 英数字の場合、2文字以上なら含める
    text.len() >= 2
}
