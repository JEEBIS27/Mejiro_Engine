# ひらがなからストロークを探す

[マニュアル一覧](README.md) · [Rust API](rust-api.md) · [実行時の辞書編集](runtime-rules.md)

逆引きはひらがなの文字列から入力可能なストローク列を探します。複数ストロークは `/` で区切ります。候補の各ストロークは、通常の順方向変換で初期状態から再生して確認します。文脈、母音の引き継ぎ、辞書編集も同じ変換規則を使います。

## 二つの候補群

`ReverseResult` には `all` と `basic_and_particle` があり、それぞれ最大10件です。後者に候補がなければ両方空になります。まず後者の最短ストローク数を求め、前者の探索上限にします。

| 候補群 | 使用する入力 |
| --- | --- |
| `all` | 基本入力、助詞、動詞、一般略語、ユーザー略語、マイナー母音・例外かな |
| `basic_and_particle` | 基本入力と助詞 |

候補はストローク数と、出力文字列内の各部分がどの入力種別で作られたかを持ちます。たとえば `Knt-Tt` が「こととは」を作る場合、「こと」は語の部分、「とは」は助詞の部分として記録します。ストロークの末尾に保留した「っ」が残る候補や、キー操作を含む候補は返しません。

## 文法情報は任意

サイトは解析した文字列の範囲を `GrammarSpan { start, end, kind, lemma }` として渡せます。範囲はUnicode文字数で、`end` は範囲外の最初の位置です。動詞の `lemma` は「いう」のような辞書形で、省略もできます。エンジンは構文解析をしません。渡された種類・辞書形と候補の出力範囲が一致すれば、その候補を優先します。一致しなくても候補は残します。文法情報がなければ短いストローク列を優先し、同数なら専用入力を基本入力より優先します。最後の同順位は装飾キーが少ない順、ストローク文字列順で固定します。

```rust
use mejiro_engine::reverse::{self, GrammarSpan, Kind};
use mejiro_engine::RuleStore;

let grammar = [GrammarSpan { start: 2, end: 5, kind: Kind::Verb, lemma: Some("いう".into()) }];
let result = reverse::lookup("とはいっても", &grammar, &RuleStore::new()).unwrap();
assert!(result.all.iter().any(|c| c.strokes == "nt-t/I-SKntk/-tk"));
assert!(result.basic_and_particle.iter().any(|c| c.strokes == "nt-t/Itk-TIA/-tk"));
```

逆引きはPC・WebAssembly用の `reverse` Cargo featureで有効にします。ファームウェア向けの標準ビルドには含まれません。Ploverの `reverse_lookup` は文法情報なしで呼び、Mejiro31サイトはブラウザー内のRust WebAssemblyに文法情報と現在の辞書編集を渡します。サイトの文法解析器は検索を実行したときに読み込みます。

検索対象は32文字までです。長すぎる入力は `InputTooLong`、探索上限に達して基本候補が見つからなかった場合は `SearchLimit` を返します。探索上限までに候補が見つかった場合は `truncated` にその旨を示します。長い文章は語句ごとに逆引きしてください。
