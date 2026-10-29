# 実行時の辞書編集

[マニュアル一覧](README.md) · [エンジンの仕様](specification.md) · [Rust API](rust-api.md) · [C API](c-api.md)

動詞・一般略語・ユーザー略語は、セッション内の「上書き辞書」で編集できます。組み込み済みの表は変更せず、同じキーに登録した内容を優先します。削除を指定すると、そのキーの組み込み項目も非表示になります。上書きを解除すると組み込み項目が復元します。

| 種類 | `RuleKind` / C の `kind` | キー | 値 |
| --- | --- | --- | --- |
| ユーザー略語 | `User` / 0 | 完全なストローク（例 `KA-`） | 表示する文字列 |
| 一般略語 | `General` / 1 | 助詞を除く左右のキー（例 `K-T`、必要なら末尾 `*`） | 語句と文法分類 |
| 一般略語の左部 | `GeneralLeft` / 2 | 左手の子音・母音（例 `K`。空キーも可） | 合成略語の前半 |
| 一般略語の右部 | `GeneralRight` / 3 | 右手の子音・母音（例 `KAU`） | 合成略語の後半 |
| 動詞 | `Verb` / 4 | 助詞と `*` を除くキー（例 `KA-KNA`） | 語幹と活用行 |

動詞の活用行は[エンジンの仕様](specification.md#動詞の定義)に従います。一般略語は `Context::Unknown`（Cでは0）を渡すと語句から分類を推定できます。明示的に分類を渡すと、後続の助詞選択にも反映されます。左部・右部の分類は合成後に推定します。ユーザー略語は文法分類を持ちません。
undoやカーソル移動など、制御・コマンドに使うストロークはユーザー略語のキーに登録できません。

Rustでは `Session::set_rule`、`hide_rule`、`clear_rule`、`clear_rules` を使います。`Session::rules().entries()` で現在の上書き一覧を取得できます。`Session::preview` と `Session::process` は同じ上書き辞書を参照します。セッションを使わず試すときは `translate_with_rules` に `RuleStore` を渡します。既存の `translate` は組み込み表だけを参照します。

```rust
use mejiro_engine::{Context, RuleKind, Session};

let mut session = Session::<20>::new();
session.set_rule(RuleKind::Verb, "KA-KNA", "よ", "m", Context::Unknown)?;
let result = session.preview("KA-KNA*", true)?;
assert!(result.output.text.as_str().starts_with("よ"));
session.hide_rule(RuleKind::Verb, "KA-KNA")?;
session.clear_rule(RuleKind::Verb, "KA-KNA")?;
# Ok::<(), mejiro_engine::Error>(())
```

Cでは `mejiro_core_session_rule_set/hide/clear/count/get/clear_all` を使います。`mejiro_rule` のキー・値・活用行は長さ付きUTF-8で、終端NULを含めません。`rule_get` は追加・編集・非表示にした項目だけを返し、組み込み表全体は列挙しません。項目がない索引は `-3`、上限超過は `-2`、不正な入力は `-1` です。失敗した編集は以前の設定を維持します。

上書きは24件まで、キー32バイト、値96バイト、活用行8バイトまでです。編集が成功すると古い文法文脈と取り消し履歴を消去します。マクロと出力モードは維持されます。USB・画面・フラッシュ保存は接続層の役割です。接続層は `rule_get` で一覧を書き出し、起動時に登録・非表示を再適用してください。

QMKの現行出力層はローマ字・JISかなと一部の記号をキー入力に変換します。漢字などの任意のUnicode文字を登録できても、そのままキーボードから送信する機能はありません。キーボード用の編集画面では、実際に送信できる文字列だけを許可してください。

QMKの「First up chords send」は物理キーの離し方によるストローク確定設定です。Rustの変換辞書には含めず、QMK側の設定として扱います。
