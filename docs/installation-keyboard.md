# キーボードへの導入

[マニュアル一覧](README.md) · [Ploverへの導入](installation-plover.md)

キーボード内でメジロ式を変換し、日本語入力に必要なキー操作をパソコンへ送る方法です。
使用するキーボード用に作られたファームウェアを導入します。

## 1. ファームウェアを用意する

[Mejiro31の案内](https://github.com/JEEBIS27/Mejiro31)から、基板とキーマップに合うファームウェアを選びます。
配布されたファームウェアを使う場合は、エンジン単体をビルドする必要はありません。

Mejiro31のRP2040版では `.uf2` ファイルを使用します。
エンジンの `.a` や `.so` は組み込み用ライブラリなので、キーボードへコピーするファイルはファームウェアの `.uf2` を選んでください。

### 自分でQMKファームウェアをビルドする場合

[QMK公式の環境構築手順](https://docs.qmk.fm/newbs_getting_started)でビルド環境を準備します。
Mejiro31のキーボード定義を含む[QMKリポジトリ](https://github.com/JEEBIS27/qmk_firmware)と、[メジロエンジン](../README.md)を同じ親ディレクトリに取得します。Rustのターゲットを追加し、QMKリポジトリのルートでビルドします。Cargoがエンジンを自動的に静的ライブラリへビルドしてリンクします。

```sh
git clone --recurse-submodules https://github.com/JEEBIS27/qmk_firmware.git
git clone https://github.com/JEEBIS27/Mejiro_Engine.git
rustup target add thumbv6m-none-eabi
cd qmk_firmware
qmk compile -kb jeebis/mejiro31 -km jp_default
```

ビルドが成功したら、生成された `jeebis_mejiro31_jp_default.uf2` を使用します。

配列プリセットを選ぶ場合は、同じQMKリポジトリのルートで候補を表示します。

```sh
python3 keyboards/jeebis/mejiro31/keymaps/jp_default/build_firmware_presets.py --list
```

表示された名前を指定してビルドします。次はQWERTYプリセットの例です。

```sh
python3 keyboards/jeebis/mejiro31/keymaps/jp_default/build_firmware_presets.py mejiro31_jp_qwerty
```

プリセットの成果物は `.build/export/` に出力されます。
ここでのビルドは、QMKのMejiro31定義と隣接する共通エンジンをファームウェアに組み込みます。
自分のキーボードへメジロエンジンを組み込む作業は、[エンジンの組み込み](integration.md)を参照してください。

## 2. Mejiro31へ書き込む

1. キーボードのUSBケーブルを抜きます。
2. PCB裏面にある、USB端子から離れた側の **BOOTボタン** を押したままUSBケーブルを接続します。
3. パソコンにRP2040のUF2ドライブが表示されたら、BOOTボタンを離します。
4. 用意した `.uf2` ファイルをそのドライブへコピーします。
5. 書き込み後、キーボードが再起動して通常の入力機器として接続されるのを確認します。

これはMejiro31のRP2040基板向けの手順です。他の基板では、その基板のブートローダーに合った手順を使います。
一般的な書き込み方法は[QMK公式の説明](https://docs.qmk.fm/flashing)でも確認できます。

## 3. 入力環境を合わせる

1. キーマップの配列表を開き、メジロ式モードと通常入力モードの切り替えキーを確認します。
2. キーボード内のメジロ式変換を有効にします。Mejiro31の `jp_default` では、配列表の `MJI_MODE` で速記モードへ切り替えます。速記モード内の変換方式は `MJI_MEJIRO` でメジロ式とGemini送信を切り替えます。物理位置は使用中の配列表で確認してください。
3. Mejiro31の現行QMK版では、パソコン側の日本語IMEをローマ字入力に設定します。他のファームウェアでは、その出力方式に合わせて設定します。
4. テキストエディターで `KA-` を入力し、`か` になることを確認します。
5. [仕様・使用例](examples.md)で助詞や取り消しを確認します。

## RMKを使う場合

メジロ式を組み込んだ、そのキーボード用のRMKファームウェアを使用します。
ファイル形式と書き込み方法は基板・ブートローダーごとに異なります。上記のUF2手順はRP2040のUF2ブートローダーを使う構成に適用します。
Rustソースから組み込む場合は[RMKへの接続](integration.md#rmkへの接続)へ進んでください。

## うまく入力できないとき

| 症状 | 確認すること |
| --- | --- |
| UF2ドライブが表示されない | BOOTボタンを押したまま接続したか、データ通信できるUSBケーブルか |
| 通常のキー入力になる | メジロ式モードと使用レイヤーが選ばれているか |
| ローマ字や別のかなが出る | ファームウェアの出力方式とIMEの入力方式が一致しているか |
| 記号の位置がずれる | OS側のキーボード配列がファームウェアの想定と一致しているか |
| 入力が二重になる | Ploverの出力も同時に有効になっていないか |

[仕様・使用例](examples.md) · [エンジンの組み込み](integration.md)
