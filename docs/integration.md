# エンジンの組み込み

[マニュアル一覧](README.md) · [Rust API](rust-api.md) · [C API](c-api.md)

この文書は、プラグインやファームウェアを作る方向けです。
メジロ式を使い始める場合は、[Ploverへの導入](installation-plover.md)または[キーボードへの導入](installation-keyboard.md)を参照してください。

## 接続方法と配布するファイル

| 利用先 | エンジンの取り込み方 | 利用者が導入するもの |
| --- | --- | --- |
| Plover | PCのOS・CPU向けにビルドした共有ライブラリをプラグインから呼ぶ | ライブラリを含むプラグイン |
| QMK | MCU向けの静的ライブラリと `mejiro.h` をCの接続層から使う | エンジンをリンクしたキーボード用ファームウェア |
| RMK | `mejiro-engine` をCargo依存としてソースからビルドする | エンジンを含むキーボード用ファームウェア |

規則はエンジンで管理し、各接続先にはキー入力・出力・設定の橋渡しを持たせます。
QMKへはMCU向けの静的ライブラリとC ABIをリンクし、RMKへはRustクレートを参照します。Mejiro31のQMK定義では、ビルド時にCargoが `.a` を生成して自動リンクします。
PC向けライブラリとMCU向けライブラリは、それぞれの対象に合わせてビルドします。

## 共通の接続手順

```text
物理キー → 同時押しの確定 → ストローク文字列
                              ↓
                       メジロエンジン
                              ↓
                 文字列 + キー操作 + 次の状態
                              ↓
                      接続先で実際に出力
```

1. 左右の論理キーを割り当て、[ストロークの順序](specification.md#ストロークの表記)で文字列を作ります。
2. 同時押しを確定するタイミングで、1ストロークを1回だけ処理します。
3. 文字列を `byte_offset` で区切り、キー操作を順番どおり挟んで送信します。
4. undoをどこで管理するかを決め、確定状態と実際の出力履歴を対応付けます。
5. 外部編集や入力モードの切り替えで対応が崩れる境界を検出し、文脈と履歴をリセットします。

キー操作のうち `SelectAlternateSystem` の接続先はホスト側で決めます。
`#-k` はこの操作の後に `ImeOff` を返します。切り替え先が未実装・未登録なら切り替え操作は何もせず、後続のIME OFFを必ず処理します。
Ploverの標準接続先は登録済みPhonetixです。QMKは `mejiro_steno_select_alternate_system()` フックで利用可能な別の速記エンジンを選びます。標準実装は何もせず `false` を返します。RMK等の接続層でも同じ順序と未対応時の動作を実装します。
エンジンが返し得る操作について、送信・無視・入力拒否のいずれで扱うかを接続前に決めておきます。
`Session::process` 成功時には状態と記録も進むため、送信できない操作を送信後のackだけで取り消すことはできません。

## Ploverへの接続

エンジンリポジトリのルートで実行します。

```sh
cargo build --release -p mejiro-native
```

| ビルド先 | 主な共有ライブラリ名 |
| --- | --- |
| Linux | `target/release/libmejiro_native.so` |
| macOS | `target/release/libmejiro_native.dylib` |
| Windows | `target/release/mejiro_native.dll` |

Ploverを実行するOS・CPUに合うファイルをプラグイン内に同梱します。
Pythonからは、同梱先を基準に絶対パスを組み立てて `ctypes` などで読み込み、引数型・戻り値型を[Cヘッダー](../native/mejiro.h)に合わせます。
読み込み時に `mejiro_abi_version()` を確認します。単にライブラリを配置するだけでなく、辞書・書式処理からの呼び出しを接続します。

Ploverの辞書検索は同じ候補を複数回参照する場合があるため、検索中にはセッションを進めません。
`translate` 相当のプレビューを使い、確定した書式アクションに次の `State` を保存します。
Ploverがundoしたときは、残った確定履歴に対応する状態を復元します。Ploverが担当する削除とエンジンの `Session` による削除を重ねないようにします。

上書き辞書を使ったときも、確定履歴側で文法分類を更新します。
Plover版では `mejiro_plover_preview` を使い、書式本文・次の状態・制御要求を一緒に取得します。
プラグインの `mejiro_engine` 拡張が確定直前に変換し、状態をPloverのTranslation履歴に保存します。
ソースの配布用コピーは `python tools/sync_plover_engine.py ../Plover_Mejiro` で生成します。プラグインのビルド時に共有ライブラリを生成するため、Python側の母音・活用・助詞表を編集する必要はありません。
既存の `mejiro_translate` と `mejiro_core_translate` も引き続き利用できます。
Pythonによる呼び出しの実例は[比較ツール](../tools/compare_plover.py)でも確認できます。

## QMKへの接続

QMKのCコードからは[CセッションAPI](c-api.md#セッションapi)を呼びます。

### MCU用ライブラリを準備する

1. キーボードのMCUに合うRustターゲットを選びます。RP2040のCortex-M0+には `thumbv6m-none-eabi` を使います。[Rustのターゲット説明](https://doc.rust-lang.org/rustc/platform-support/thumbv6m-none-eabi.html)を参照してください。
2. ファームウェア側に、コアを参照する `no_std` の `staticlib` ラッパークレートを作ります。
3. ラッパーにパニック時の処理と必要なC ABIのエクスポートを用意し、対象MCU用にビルドします。
4. 生成した `.a` と同じ版の `native/mejiro.h` を、QMKの接続層から参照できるようにします。Mejiro31のQMK定義は、隣接する本リポジトリのヘッダーを参照します。
5. QMKのリンク設定に静的ライブラリを追加し、必要なC ABIのシンボルが最終ファームウェアへ含まれることを確認します。

`native/` はPC用の `std` を使うラッパーです。QMK用には、上記のMCU向けラッパーを使用します。
ラッパー、メモリ配置、リンカー設定は使用するMCUとQMK構成に合わせて用意します。Mejiro31では `qmk_firmware` と `Mejiro_Engine` を同じ親ディレクトリに置き、キーボード定義内の `steno/rust/` ラッパーと `rules.mk` でビルド・リンクします。

コア自体のRP2040向けコンパイルは、ターゲットを導入した環境で次のように確認できます。

```sh
rustup target add thumbv6m-none-eabi
cargo build --release -p mejiro-engine --target thumbv6m-none-eabi
```

このコマンドの成果物はRustのコアライブラリです。Cからリンクする `.a` や、書き込み用 `.uf2` は、ラッパーとQMK側のビルドで生成します。

### 入出力を接続する

- キーの押下・解放を集約し、確定時に `mejiro_core_session_process` を呼びます。
- `kind` を判定し、通常出力なら文字とキー操作、undoなら削除を行います。
- 成功した通常出力に対して、実際の削除単位を `mejiro_core_session_ack` で登録します。
- カーソル移動やIMEとの対応が失われる外部操作では `mejiro_core_session_reset` を呼びます。

`Kana` のUTF-8文字列は、そのまま通常のHIDキー列になるわけではありません。
接続層で日本語IME向けのキー列に変換するか、ホストに合う文字送信方式を用意します。
`Romaji`／`Jis` を使う場合も、OSの配列・IME設定・修飾キーの扱いを一致させます。

ビルド後はセッション領域、Flash使用量、最悪時のスタック使用量を確認します。
書き込み手順は[キーボードへの導入](installation-keyboard.md)を参照してください。

## RMKへの接続

利用側の `Cargo.toml` にコアを追加します。次はリポジトリが隣り合う場合です。

```toml
[dependencies]
mejiro-engine = { path = "../Mejiro_Engine" }
```

1. メジロ式入力を処理する場所に `Session::<20>` を保持します。
2. RMK側で同時押しを確定し、`session.process(stroke)` を呼びます。
3. [Rust APIの出力処理](rust-api.md#文字とキー操作を順番に処理する)に従って、RMK側の文字・キー送信処理へ渡します。
4. 出力完了後に `acknowledge` を呼び、外部編集などの境界で `reset` を呼びます。
5. 対象キーボードの通常のRMKビルド・書き込み手順でファームウェアを生成します。

キーイベントのフックと送信APIは、使用するRMKの版に対応する[公式ドキュメント](https://rmk.rs/)を参照します。
同一セッションの更新は1か所に集め、非同期タスクから同時に変更しない構成にします。
Rust側から使うので、CヘッダーやPC用共有ライブラリは必要ありません。

## 更新と動作確認

利用側ではエンジンの参照リビジョンを固定し、ヘッダー・ライブラリ・接続層を対応する組み合わせで更新します。
QMKではライブラリを差し替えた後にファームウェアを再リンクし、RMKでは依存するコアを含めて再ビルドします。

[仕様・使用例](examples.md)の基本入力、文脈依存の助詞、undo、マクロを実際の出力先で確認します。
特にundoは送信したキーの数ではなく、ホスト上で元の状態へ戻せるかを確認します。

[エンジンの仕様](specification.md) · [Rust API](rust-api.md) · [C API](c-api.md)
