# Ploverへの導入

[マニュアル一覧](README.md) · [キーボードへの導入](installation-keyboard.md)

パソコン上でPloverを使い、メジロ式のストロークを日本語へ変換する方法です。

## 用意するもの

- Ploverと、プラグインを導入できる環境。
- Ploverで入力を受け取れるキーボード。
- [Plover版メジロ式](https://github.com/JEEBIS27/Plover_Mejiro)の配列表。

## インストール

1. Ploverを起動し、`Tools` → `Plugins Manager` を開きます。
2. `GIT` → `Install from Git repo` を開き、次のURLを入力します。

   ```text
   https://github.com/JEEBIS27/Plover_Mejiro
   ```

3. インストールが完了したらPloverを再起動します。
4. `Configure` → `System` で `Mejiro` を選びます。
5. `Machine` で使用するキーボードの入力方式を選び、キー配置を設定します。具体的な割り当ては[Plover版の説明](https://github.com/JEEBIS27/Plover_Mejiro#readme)を参照してください。

## 入力を確認する

1. テキストエディターを開き、Ploverの出力を有効にします。
2. 配列表の左手 `K` と `A` を同時押しします。ストローク表記は `KA-` です。
3. `か` が入力されることを確認します。
4. [仕様・使用例](examples.md)の基本入力と助詞の例を試します。
5. `-U` で取り消しを試します。Ploverの履歴が空でも2文字分のBackspaceを送ります。

キーボード側で日本語を生成する機能とPloverの変換を同時に有効にすると、入力経路が重なります。
Mejiro31からストロークを送る場合は、キーマップの案内に従ってキーボード内のメジロ式変換を切り替え、Ploverにストロークを送る設定にします。

## うまく入力できないとき

| 症状 | 確認すること |
| --- | --- |
| `Mejiro` を選択できない | プラグインのインストール結果とPloverの再起動 |
| キーを押しても変換されない | Ploverの出力が有効か、`Machine` とキー割り当てがキーボードに合っているか |
| 入力が二重になる | キーボード内変換とPlover側変換の両方が有効になっていないか |
| 例と異なる助詞になる | 直前の入力や上書き辞書の有無。[助詞の例](examples.md)は指定された順番で試す |

エンジンライブラリをプラグインへ組み込む方法は[エンジンの組み込み](integration.md)に記載しています。
