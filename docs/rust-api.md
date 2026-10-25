# Rust APIの使い方

[マニュアル一覧](README.md) · [エンジンの仕様](specification.md) · [組み込み](integration.md)

## 依存関係を追加する

Rust 1.88以降を使用します。次は利用側のプロジェクトと `Mejiro_Engine` が隣り合っている場合の `Cargo.toml` です。

```toml
[dependencies]
mejiro-engine = { path = "../Mejiro_Engine" }
```

クレート名は `mejiro-engine`、Rustコード内の名前は `mejiro_engine` です。
コアは `no_std` なので、RMKなどの組み込み側でも同じAPIを使用できます。

## 変換を試す

次を `src/main.rs` に保存し、`cargo run` で実行します。

```rust
use mejiro_engine::{State, translate};

fn main() -> mejiro_engine::Result<()> {
    let initial = State::default();
    let first = translate("n-*", initial, true)?;
    assert_eq!(first.output.text.as_str(), "している");

    let second = translate("k-k", first.next, true)?;
    assert_eq!(second.output.text.as_str(), "のが");
    assert_eq!(initial, State::default());
    Ok(())
}
```

`translate(stroke, state, include_commands)` はプレビューです。引数の状態を変更せず、`Translation` を返します。
確定した入力に対してのみ `next` を次の入力の状態として採用します。
`include_commands = false` は、コマンド辞書を別に扱う辞書検索向けです。undo・マクロなどの制御要求は、このフラグにかかわらず `request` に分類されます。

| `Translation` のフィールド | 内容 |
| --- | --- |
| `output` | 文字列・キー操作・出力の文法分類 |
| `next` | 入力を確定した後の状態 |
| `request` | 通常出力、undo、繰り返し、モード変更、マクロ操作の要求 |
| `dynamic_particle` | 文脈で変化する単独助詞の構文かどうか |

`request` が `Output` 以外の場合は、接続層で要求を処理します。
確定入力の履歴もエンジンに任せる場合は、次の `Session` を使います。

## セッションで入力とundoを扱う

次は、かなを直接保存する簡単な文字列ホストです。このホストではUnicodeスカラー値1個を1単位として削除できます。
IMEの削除方式を模擬するコードではありません。

```rust
use mejiro_engine::{EventKind, Session};

fn main() -> mejiro_engine::Result<()> {
    let mut session = Session::<20>::new();
    let mut document = String::new();

    let event = session.process("KA-")?;
    assert_eq!(event.kind, EventKind::Emit);
    assert!(event.output.operations().is_empty());
    document.push_str(event.output.text.as_str());
    if !event.output.is_empty() {
        if let Some(ticket) = event.ticket {
            let units = event.output.text.as_str().chars().count() as u32;
            session.acknowledge(ticket, units)?;
        }
    }
    assert_eq!(document, "か");

    let undo = session.process("-U")?;
    assert_eq!(undo.kind, EventKind::Undo);
    for _ in 0..undo.erase_units {
        document.pop();
    }
    assert!(document.is_empty());
    Ok(())
}
```

実際の接続層では、次の順番で処理します。

1. 確定したストロークに対して1回だけ `process` を呼びます。
2. `Emit` なら文字とキー操作を順番どおり出力します。
3. 空でない出力に `ticket` が付いていれば、実際に取り消せる単位を `acknowledge` に渡します。次の入力を処理する前に登録します。
4. `Undo` なら `erase_units` に従って削除します。`output` は再送しません。
5. `Control` なら文字を出力しません。

`ticket` がないイベントには登録しません。空の出力に付いた `ticket` はエンジン内で登録済みです。
IMEでは、送ったローマ字の文字数とBackspaceで消せる回数が一致するとは限りません。
外部編集や送信失敗で履歴の対応が失われたら `session.reset()` を呼びます。

## 文字とキー操作を順番に処理する

次は送信順を表示する実行例です。実際の接続層では `println!` を文字送信・キー送信に置き換えます。

```rust
use mejiro_engine::{State, translate};

fn main() -> mejiro_engine::Result<()> {
    let translated = translate("-TYI", State::default(), true)?;
    let output = translated.output;
    let text = output.text.as_str();
    let mut from = 0;
    for operation in output.operations() {
        let part = &text[from..operation.byte_offset];
        if !part.is_empty() {
            println!("text: {part}");
        }
        println!("key: {:?}", operation.key);
        from = operation.byte_offset;
    }
    if from < text.len() {
        println!("text: {}", &text[from..]);
    }
    Ok(())
}
```

出力は `text: （）`、続いて `key: Left` です。
同じバイト位置に複数の操作があれば、配列順に処理します。

## モードを選ぶ

```rust
use mejiro_engine::{Mode, Session};

fn main() -> mejiro_engine::Result<()> {
    let mut session = Session::<20>::new();
    session.set_typing_mode(Mode::Romaji)?;
    session.process("#-n")?;
    assert_eq!(session.state().mode, Mode::Romaji);
    assert_eq!(session.process("KA-")?.output.text.as_str(), "ka");
    Ok(())
}
```

`Mode::Jis` を選ぶ場合も同じです。`set_typing_mode(Mode::Kana)` はエラーになります。
上記は変換結果を調べる例です。実際に出力して履歴を使う場合は、送信後に削除単位も登録します。

## そのほかの操作

| API | 用途 |
| --- | --- |
| `session.state()` | 現在の文法・母音・モード・促音を取得 |
| `state.last_vowel()` / `state.pending_sokuon()` | 母音と促音の状態を取得 |
| `session.history_len()` | 保持している履歴件数を取得 |
| `session.process_override(text)` | 接続先の辞書で得た文字列を出力モード変換し、履歴・記録に含める |
| `session.recording(index)` | 指定したマクロ枠が記録中かを取得 |
| `session.recorded(index)` | 記録済みの文字・キー操作を参照 |
| `session.reset()` | 文脈と履歴をリセット。モードとマクロは保持 |

`-U` は履歴がある場合は登録済みの削除単位を返し、履歴が空の場合は2文字分の削除を返します。接続先で `EventKind::Undo` の `erase_units` を実際の削除操作へ変換してください。

`process_override` は文字列を受け取り、Ploverの書式命令は解釈しません。
出力の文法分類を `Unknown` として扱います。undoで置換前の文法状態に戻れます。
マクロ枠の番号は `n, t, k, nt, nk, tk, ntk` の順に0〜6です。

[容量・エラーの仕様](specification.md#容量とエラー) · [仕様・使用例](examples.md) · [C API](c-api.md)
