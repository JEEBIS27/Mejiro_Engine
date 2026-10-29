set shell := ["bash", "-euc"]
set positional-arguments

# コマンド一覧
default:
    @"{{just_executable()}}" --list

# リリースビルド
build:
    cargo build --locked --release --workspace

# 全クレートのテスト
test *args:
    cargo test --locked --workspace "$@"

# Rustソースを整形
fmt:
    cargo fmt --all

# 整形を変更せず検査
fmt-check:
    cargo fmt --all -- --check

# 静的検査
lint:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# 通常の開発チェック
check: fmt-check lint test

# 隣のPloverへ現在の作業ツリーのEngineをコピー（コミット不要）
sync-plover destination='../Plover_Mejiro':
    python3 tools/sync_plover_engine.py "$1"

# 現在のEngineをPloverとQMK Vialの両方へ反映
sync: sync-plover sync-qmk

# 隣のQMK Vial作業領域を現在のEngineで準備
sync-qmk destination='../qmk_firmware':
    "{{just_executable()}}" --justfile "$1/justfile" sync "{{justfile_directory()}}"
