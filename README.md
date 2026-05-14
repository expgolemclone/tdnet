# TDNet Viewer

TDNet の適時開示 PDF をブラウザで読むための Rust 製ビューアです。

## Requirements

- Rust toolchain
- `cargo`
- Linux/macOS の場合、既存プロセスの port 解放には `lsof` があると便利です

## Run

Python 版の `main.py` はありません。Rust 版は `cargo run --` の後ろにアプリの引数を書きます。

```sh
cargo run -- --date 20260515 --period day --all
```

起動すると既定で `http://localhost:8080` を開きます。

別の port を使う場合:

```sh
cargo run -- --date 20260515 --period day --all --port 18080
```

今日の日付で起動する場合:

```sh
cargo run -- --period day --all
```

## Build

release binary を作る場合:

```sh
cargo build --release
```

作成した binary を直接実行する場合:

```sh
./target/release/tdnet-viewer --date 20260515 --period day --all
```

## Options

- `--date yyyymmdd`: 取得する終了日。省略時は今日。
- `--period day|week|month|year`: 取得期間。平日のみを対象にします。
- `--all`: ticker filter なしで全件表示します。
- `--port PORT`: 起動 port。省略時は `8080`。
- `--ticker-csv PATH`: CSV の 1 列目を ticker code として読み込みます。
- `-t, --tickers CODE...`: ticker code を直接指定します。
- `--segment NAME...`: `segments.json` の市場区分で絞り込みます。
- `--fetch-segments`: JPX の上場会社データを取得し、`segments.json` を作成します。

例:

```sh
cargo run -- --fetch-segments
cargo run -- --period week --segment プライム
cargo run -- --period month --ticker-csv ticker.csv
cargo run -- --date 20260515 -t 7203 6758
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
- `:tree`: 開示一覧を表示
