# C APIの使い方

[マニュアル一覧](README.md) · [エンジンの仕様](specification.md) · [組み込み](integration.md)

宣言は [native/mejiro.h](../native/mejiro.h) にあります。
ヘッダーとライブラリは同じエンジンの版を使用してください。
PC上で呼び出す場合と、QMKへ組み込む場合でライブラリのビルド対象が異なります。

## PC上で変換を試す

エンジンリポジトリのルートでビルドします。

```sh
cargo build --release -p mejiro-native
```

以下を `example.c` として保存します。

```c
#include <assert.h>
#include <stdio.h>
#include "mejiro.h"

int main(void) {
    assert(mejiro_abi_version() == 1);
    const uint8_t stroke[] = "Kntk-Tk";
    const mejiro_state initial = {0};
    mejiro_translation result;
    int32_t status = mejiro_core_translate(
        stroke, sizeof(stroke) - 1, &initial, 1, &result);
    if (status != 0) {
        return 1;
    }
    assert(result.request == 0);
    assert(result.operation_count == 0);
    fwrite(result.text, 1, result.text_len, stdout);
    putchar('\n');
    return 0;
}
```

Linuxでは、エンジンリポジトリのルートで次のようにリンク・実行できます。

```sh
cc -std=c11 -Wall -Wextra -Werror -I native example.c \
  -L target/release -lmejiro_native \
  -Wl,-rpath,"$PWD/target/release" -o example
./example
```

`ことへの` と表示されます。この例はプレビューのみを実行します。

## プレビューの入出力

`mejiro_core_translate` は `0` で成功です。`include_commands` は0または1を渡します。
ストロークにはバイト長を渡し、終端NULは長さに含めません。
出力の `text` も長さ付きのUTF-8で、NUL終端ではありません。

| `mejiro_translation` のフィールド | 意味 |
| --- | --- |
| `next` | 次の状態 |
| `text` / `text_len` | 文字列と有効なバイト数 |
| `operations` / `operation_count` | キー操作と有効な件数 |
| `output_context` | 出力の文法分類 |
| `request` | 下表の要求種別 |
| `argument` | マクロ操作の枠番号（0〜6） |
| `dynamic_particle` | 文脈依存の単独助詞構文なら1 |

| `request` | 意味 |
| --- | --- |
| 0 | 通常出力 |
| 1 | undo |
| 2 | 繰り返し |
| 3 | タイピング出力モードの切り替え |
| 4 | マクロ記録開始／停止 |
| 5 | 最後に開始したマクロ記録の停止 |
| 6 | マクロ再生 |

プレビューは制御要求を実行しません。共通の履歴・マクロ処理にはセッションAPIを使います。

## セッションのメモリを用意する

必要なサイズとアライメントは `mejiro_core_session_size()` と `mejiro_core_session_align()` で取得します。
Cセッションは `Session<20>` に対応し、内部レイアウトを直接読み書きしません。

以下はPCで動かす、C11の `aligned_alloc` を使った独立した例です。
メモリ上の「か」1文字を1単位として保持・削除する、簡単なホストを模擬しています。

```c
#include <assert.h>
#include <stdlib.h>
#include "mejiro.h"

int main(void) {
    size_t alignment = mejiro_core_session_align();
    if (alignment < sizeof(void *)) {
        alignment = sizeof(void *);
    }
    size_t size = mejiro_core_session_size();
    size_t rounded = (size + alignment - 1) / alignment * alignment;
    uint8_t *session = aligned_alloc(alignment, rounded);
    if (session == NULL) {
        return 1;
    }
    assert(mejiro_core_session_init(session, rounded) == 0);

    const uint8_t stroke[] = "KA-";
    mejiro_event event;
    assert(mejiro_core_session_process(
        session, stroke, sizeof(stroke) - 1, &event) == 0);
    assert(event.kind == 0 && event.has_ticket == 1);
    assert(event.translation.operation_count == 0);
    assert(event.translation.text_len == 3);
    unsigned document_units = 1;
    assert(mejiro_core_session_ack(session, event.ticket, document_units) == 0);

    const uint8_t undo[] = "-U";
    assert(mejiro_core_session_process(
        session, undo, sizeof(undo) - 1, &event) == 0);
    assert(event.kind == 1 && event.erase_units == 1);
    document_units -= event.erase_units;
    assert(document_units == 0);
    free(session);
    return 0;
}
```

履歴が空の状態でさらに `-U` を処理すると、`kind == 1` のundoイベントと `erase_units == 2` が返ります。接続層はBackspaceなどで2文字分を削除します。

前の例と同じ方法でコンパイルできます。`assert` で呼び出しも確認するサンプルなので、`NDEBUG` を定義せずに実行してください。
組み込みでは、対象MCUで取得したサイズ・アライメントに合う静的領域を接続層で用意します。
エンジン自体はヒープ確保を要求しません。PCで照会したサイズを別のMCUへそのまま転用しないでください。

## セッションAPI

| 関数（`mejiro_core_` に続く名前） | 役割 |
| --- | --- |
| `session_size` / `session_align` | 必要な領域と整列を照会 |
| `session_init` | メモリを初期化。再実行するとモード・マクロも初期化 |
| `session_process` | 確定ストロークを1回処理 |
| `session_ack` | 最新の非空出力について実際の削除単位を登録 |
| `session_reset` | 文脈・履歴をリセットし、モード・マクロを保持 |
| `session_set_typing_mode` | 切り替え先のモードを1（Romaji）または2（Jis）に設定 |

`mejiro_event.kind` は0＝出力、1＝undo、2＝制御です。
`session_process` の結果は **`kind`** で分岐します。`translation.request` はプレビューと同じ制御分類を保持するフィールドではありません。
文字と操作は `translation` に入り、undoの削除量は `erase_units` に入ります。
`has_ticket` が1で、文字またはキー操作を出力した場合にだけackを行います。
undoの `translation` は取り消す出力で、再送してはいけません。

同じセッションへの呼び出しは直列に行います。ポインターには必要な長さと整列を持つ有効な領域を渡し、入力・出力・セッションを重ねません。
`session_init` を済ませた領域だけを各セッション関数へ渡します。
専用の破棄関数は不要です。利用終了後、呼び出し元で領域を解放できます。

## 状態と戻り値

`mejiro_state` をすべて0で初期化すると、母音A・文法不明・Kana・保留促音なしになります。

| フィールド | 値 |
| --- | --- |
| `vowel` | `A, I, U, IA, AU, YA, YU, YAU, Y, YI, YIA, YIU, YIAU, IU, IAU` の順で0〜14 |
| `context` | [文法分類](specification.md#状態)の記載順で0〜8 |
| `mode` | 0＝Kana、1＝Romaji、2＝Jis |
| `pending_sokuon` | 0または1 |

| 戻り値 | 意味 |
| --- | --- |
| 0 | 成功 |
| -1 | 不正な入力・状態・引数・ack |
| -2 | 容量不足（変換・セッション処理） |
| -3 | 制御要求（PC向け `mejiro_translate` のみ） |
| -4 | undo対象の削除単位が未登録（セッション処理） |

失敗時は結果バッファを使用せず、戻り値を処理します。変換・セッション処理の失敗時に結果バッファは書き換えられません。

## Plover書式を返すPC向けAPI

`mejiro_translate` は、型付き出力をPlover書式の本文へ変換するプレビューです。
出力バッファには終端NULを含む容量が必要です。返された本文の書式への組み込みと状態の確定はプラグイン側で行います。
`mejiro_abi_version()` の値は1です。

`mejiro_translate` は制御要求に `-3` を返すため、undoやマクロを処理する関数として使いません。
Ploverの確定履歴との対応は[組み込みガイド](integration.md#ploverへの接続)を参照してください。

[エンジンの仕様](specification.md) · [Rust API](rust-api.md) · [仕様・使用例](examples.md)
