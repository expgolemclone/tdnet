# TDNet Viewer

TDNet の適時開示 PDF をブラウザで読むための Rust 製ビューアです。

## Requirements

- Rust toolchain
- `cargo`
- Linux/macOS の場合、`--kill-existing` による既存プロセスの port 解放には `lsof` が必要です

## Run

Python 版の `main.py` はありません。Rust 版は `cargo run --` の後ろにアプリの引数を書きます。

```sh
cargo run -- serve --date 20260515 --period day --all
```

起動すると既定で `http://127.0.0.1:8080` を開きます。

別の port を使う場合:

```sh
cargo run -- serve --date 20260515 --period day --all --port 18080
```

今日の日付で起動する場合:

```sh
cargo run -- serve --period day --all
```

## Build

release binary を作る場合:

```sh
cargo build --release
```

作成した binary を直接実行する場合:

```sh
./target/release/tdnet-viewer serve --date 20260515 --period day --all
```

## Commands

- `serve`: TDNet の開示PDFビューアを起動します。
- `fetch-segments`: JPX の上場会社データを取得し、`segments.json` を作成します。

## Serve Options

- `--date yyyymmdd`: 取得する終了日。省略時は今日。
- `--period day|week|month|year`: 取得期間。平日のみを対象にします。
- `--all`: ETF/ETN を含めて全件表示します。filter 系オプションとは併用できません。
- `--port PORT`: 起動 port。省略時は `8080`。
- `--kill-existing`: 指定 port をListenしている既存プロセスを終了してから起動します。
- `--ticker-csv PATH`: CSV の 1 列目を ticker code として読み込みます。
- `-t, --tickers CODE...`: ticker code を直接指定します。
- `--segment NAME...`: `segments.json` の市場区分で絞り込みます。

例:

```sh
cargo run -- fetch-segments
cargo run -- serve --period week --segment プライム
cargo run -- serve --period month --ticker-csv ticker.csv
cargo run -- serve --date 20260515 -t 7203 6758
```

## Test

```sh
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

## UI

ブラウザでは次のキー操作が使えます。

- `j` / `k`: PDF ページ移動
- `h` / `l`: 開示ファイル切り替え
- `o`: 番号ジャンプ
- `gg` / `G`: 先頭 / 末尾へ移動
- `yy`: 現在ページを引用形式でコピー
- `i`: PDF 表示テーマを切り替え
- `:tree`: 開示一覧を表示。開示一覧では `j` / `k`、`Enter`、`/`、`q` が使えます。
