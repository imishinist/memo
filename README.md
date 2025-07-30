# Memo - シンプルなメモ管理ツール

Rustで実装されたコマンドラインメモ管理ツールです。

## 特徴

- **低摩擦**: 最小限の操作でメモを開始
- **自動整理**: 日付・時刻による自動分類
- **XDG準拠**: XDG Base Directory仕様に準拠したデータ保存
- **タグ対応**: `@tag` 形式でのタグ付け
- **VSCode連携**: メモディレクトリを簡単に開ける

## インストール

```bash
# Cargoでインストール
cargo install --path .

# または手動ビルド
cargo build --release
cp target/release/memo ~/.local/bin/
```

## 使用方法

### 基本コマンド

#### 新規メモ作成
```bash
memo add
```
- 現在の日時に基づいてファイルを作成
- `$EDITOR` 環境変数で指定されたエディタで編集
- ファイルは `~/.local/share/memo/YYYY-MM/DD/HHMMSS.md` に保存

#### メモ編集
```bash
memo edit <id>
```
- IDは以下の形式をサポート:
  - 完全ID: `2025-01/30/143022`
  - 短縮ID: `0130143022` (月日時分秒)
  - さらに短縮: `30143022` (日時分秒、同月内)
  - 最短: `143022` (時分秒、同日内)

#### メモ一覧
```bash
memo list
```
- 最新の20件のメモを表示
- 作成日時とプレビューを表示

#### メモディレクトリ表示
```bash
memo dir
```
- メモが保存されているディレクトリパスを出力
- grep検索やVSCodeで開く際に便利

### 使用例

```bash
# 新規メモ作成
memo add

# 今日作成したメモを編集
memo edit 143022

# メモ一覧を表示
memo list

# grepでタグ検索
grep -r "@meeting" $(memo dir)

# VSCodeでメモディレクトリを開く
code $(memo dir)
```

## ファイル構造

```
~/.local/share/memo/
├── 2025-01/
│   ├── 30/
│   │   ├── 143022.md
│   │   └── 151545.md
│   └── 31/
│       └── 090000.md
```

## メモファイル形式

```markdown
# メモタイトル（任意）

メモ内容をここに書く

@tag1 @meeting @urgent

## 追記
後から追記した内容
```

## タグ機能

- タグは `@tag` 形式で記述
- Markdownの `#` と競合しないよう `@` を使用
- 手動でgrepによる検索が可能:
  ```bash
  grep -r "@meeting" $(memo dir)
  ```

## 環境変数

- `$EDITOR`: 使用するエディタ（デフォルト: vi）
- `$XDG_DATA_HOME`: データディレクトリ（デフォルト: ~/.local/share）

## 開発

```bash
# 開発用ビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release
```

## ライセンス

MIT License
