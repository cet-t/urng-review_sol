# urng 1.0.0 修正・検証レポート

## 作業状態と基点

- 作成日: 2026-10-16。修正、回帰テスト、feature 検証、性能比較を完了。未解決の依存先ビルド問題と未実行の検証は末尾に明記。
- 修正前: tag `1.0.0`、`9f8518e05b4c7580d8a291ba679e900fb8dba0af`。
- 対応コミット: **`2614fb6` — Fix RNG memory safety, reference algorithms and numeric sampling**。下表の全修正・回帰テスト・ベンチコード/CSVを含む。親が上記の1.0.0基点であることを確認済み。本報告書は後続の文書コミットに保存する（`git log -1 -- FIX_REPORT.md` で識別できる）。
- 作業開始前から存在する未追跡の `REVIEW.md` は利用者のファイルとして保持し、コミット対象から除外する。
- 優先順位は、メモリ/FFI/CPU 安全性 → 系列重複 → 参照アルゴリズム → 数値境界/分布 → 全構成検証 → 性能比較。正しさのための性能低下を許容した。

## 修正した問題・対応ファイル・回帰テスト

番号は作業開始時の `REVIEW.md` の確認済み欠陥に対応する。単一語の固定変換と、一様な範囲生成は契約を区別した。

| # / 優先度 | 修正内容・主なファイル | 欠陥を検出する検証 |
|---|---|---|
| 1 Critical | `src/prng/b64/xoshiro.rs`: `Xoshiro256Ssx2` の64バイト→16バイト書込みを廃止。2本の xoshiro256** を配列上で正しく更新する安全な経路へ変更。512-bit 型は格納形式として保持。 | 既知状態 `[1,2,3,4] → 11520`、5 seed×2 lane×256 step、C ABI の全 lane/端数/guard page。危険な修正前処理は実行していない。 |
| 2 High | `src/cabi/**/*.rs`, `src/cabi/_internal.rs`: 解放/出力を `pub unsafe extern "C" fn` に変更し所有権契約を明記。null、要素/状態の alignment、サイズ・アドレス overflow、状態との重複を参照生成前に拒否。未初期化出力には `MaybeUninit<T>` と `write` を使用。 | unsafe 関数型の検査、compile-fail doctest、null/0長/不整列/サイズ overflow/重複/未初期化バッファのテスト。 |
| 3 High | SIMD 各 core、`src/rngv.rs`, `src/lib.rs`: safe な入口で必要 CPU 機能を検査。AVX2 の rotate/blend を AVX2 命令で実装。`WordV` 変換を ISA 不要の配列計算へ変更。intrinsic 公開を x86_64 に限定。C SIMD constructor は未対応 CPU で null。 | 全 lane 比較、AVX2/AVX-512 明示構成の実行、128/256/512-bit 変換テスト。実機は必要 ISA を搭載しており、非対応 CPU での拒否動作は未実行。 |
| 4 High | C ABI の個別 NT-store kernel を除去し、通常ストアを使う共有経路に統一。ワーカーに fence のない経路は存在しなくなった。 | C ABI 出力・境界テスト、実 DLL の C guard-page 試験。既存の private NT helper/fence とそのテストは保持。メモリ順序異常を再現したという主張ではない。 |
| 5 High | `src/cabi/{prng,cbrng}/**`: チャンクごとの再シードを廃止し、所有する状態を進めて生成。`chunk_idx + lane_idx` による別チャンクでの系列再使用を除去。 | 3つの元の重複パターンの非一致、全56種の scalar/block 系列照合と旧チャンク境界前後の比較。 |
| 6 High | `src/wide/splitmix.rs`, `src/prng/b32/splitmix.rs`: 幅 N ごとに N×gamma 進め、全 lane の finalizer を scalar と一致させた。intrinsic 版にも同じ規約を適用。 | x4/x8/x16 を4 seed×128 blockで scalar の連続系列と完全比較。scalar 自体は別途 unsigned C 系列と照合。 |
| 7 High | `src/crng/b32/{chacha,salsa}.rs`: 標準定数/鍵/nonce/counter、正しいラウンド数と feed-forward、Salsa の rotate=13、counter carry を実装。256-bit 鍵 API `from_key_nonce` を追加。counter 枯渇後は再利用前に panic。 | RFC 8439 の ChaCha20 全16語、Salsa20 zero-key 64バイト、Salsa8/20 の libsodium 公式4 MiBハッシュ、counter carry/最終 block/枯渇。 |
| 8 High | SFC64 の c 更新、xoroshiro128++ の出力/49・21・28 遷移、xoroshiro64** SIMD の出力 scramble、Biski SIMD の旧状態使用、Philox32 の10 round/加算鍵スケジュールを修正。主に `src/prng/b{32,64}/*.rs`, `src/cbrng/b32/philox.rs`, `src/wide/{sfc,xoroshiro}.rs`。 | 独立 C 系列、Random123 KAT、各 SIMD/wide の全 lane 比較。C ABI は修正済み core を共有。 |
| 9 Medium | `src/lib.rs`: 実在する `README.md` を include し、大文字小文字の誤りを修正。README 自体を doctest に含める。 | Windows の build/doctest、実ファイル名と include パスの照合。Linux 実行は未実施。 |
| 10 Medium | `src/prng/b32/pcg.rs`, `src/wide/pcg.rs`: 増分を odd に限定。`src/prng/b{32,64}/xorshift.rs`, `src/wide/xorshift.rs`: seed mixer がゼロを返す lane を非ゼロへ補正。 | PCG 1024 seed の odd 検査、公開 PCG KAT。Xorshift32/64 のゼロ化 seed、wide 全幅/intrinsic の lane 生存テスト。 |
| 11 Medium | `src/cbrng/b64/{philox,threefish}.rs`: `Philox4x64`, `Threefish256` の初期 pos を block 長にして、初回から生成する。 | 最初の4語と最初の生成 block の一致。 |
| 12 Medium | `src/prng/b64/xoroshiro.rs`: `long_jump` を一回の状態走査・一回の状態代入へ修正。 | ++/** の jump/long_jump 後16語を C 参照系列と比較。 |
| 13 Medium | `src/wide/threefry.rs`: 各 lane の counter で c0→c1→c2→c3 の carry を伝播。 | 2/4-word×4/8/16 lane の全要素比較。全 counter 語が MAX の状態から wrap/carry を検査。 |
| 14 Medium | `src/shuffle.rs`, `src/choice.rs`: Fisher–Yates の j を `[i,len)` から選択。一様な index を利用。 | 長さ3の全6置換が重複なく到達し、`shuffle`/`shuffled` が一致。 |
| 15 Medium | `src/rng.rs`, `src/rngv.rs`, SIMD array API: 符号オフセットの加算を広い型で行い、全 i32/i64 範囲と 2^32 幅を扱う。 | scalar・WordV 全幅・SIMD array・全 C ABI numeric family で全符号範囲を比較。 |
| 16 Medium | `src/rng.rs`, `src/rngv.rs`, `src/wide/wrng.rs`, array API: `randi`/`randiv`/`index` を棄却法へ変更し、Choice/Shuffle/Alias が使用。u32 RNG でも長さ>2^32の index を扱う。`Word::to_randi/to_index`, `WordV::to_randi` は固定語変換として偏りを文書化。 | 低積の棄却を強制する入力、各 vector lane の棄却、幅全体、64-bit length のテスト。統計試験の合格だけに依存していない。 |
| 17 Medium | `src/rngv.rs`: 奇数 lane も正しい積の上位語とその lane の scale/min を使用。 | 128/256/512-bit の全 lane を個別 scalar 変換と比較。 |
| 18 Medium | `src/prng/b64/twisted_gfsr.rs`: 内部32-bit出力を2語結合して真の u64 を返す。SplitMix64 による seed 展開で上位 seed bit も使用。 | 上位32 bit、choice の到達範囲、上位 seed bit を変えた系列の非一致。 |
| 19 Medium | `src/sampler/b{32,64}/{alias,bst}.rs`: 有限・非負・正 total を検査。最大重みで正規化し MAX/tiny の overflow を回避。Alias の pop 条件、BST の厳密な上界選択を修正。 | MAX/MAX、最小 subnormal、zero weight、先頭 zero/乱数 zero、空/負/NaN/Inf/all-zero の全4 sampler。 |
| 20 Medium | `src/rand/b{32,64}/mod.rs`, `src/_internal.rs`: 既存64-bit SeedableRng 対象16種に TryRng を実装。next_u64/fill_bytes を native 語幅に対応させる。32-bit の不足アダプタも追加。Seed bytes を little-endian に固定。 | 全16種の64-bit/全25種の32-bit adapter、byte tail/canary、u32/u64 出力、明示 seed の endian。 |
| 21 Medium | `src/rng.rs` と array/WordV 変換: 丸めた上端を next_down で除外。max-min が Inf になる有限端点も扱う。 | 隣接 float の半開区間、±MAX、全 SIMD array/C ABI family の上端。 |
| 22 Low | `README.md`, `src/wide/mod.rs`: `WRng` を `RngW` に修正し Rust fence/feature 条件を訂正。誤った API/名前/entropy/C ABI 契約も修正。 | README を含む default/all-feature doctest、`cargo doc` 警告ゼロ。 |

追加の参照照合で、Threefry32x4 の独自 tweak/feed-forward と Threefish256 の counter XOR も標準仕様へ合わせた。内部予約フィールドは残した。Random123 の zero/all-ones/非ゼロ KAT、および Skein 1.3 の zero/非ゼロ鍵・tweak・plaintext KAT で検証した。Threefish の非ゼロ plaintext は、誤った最終 XOR を検出する。

前回「懸念」とした wide ベンチでは `examples/wide/test.rs` の出力に `black_box` を追加した。元の生成コードで計算消去が実際に起きていたとは断定しない。今回の性能比較は別の共通ハーネスで行った。

## 参照実装・期待値の独立性

- `src/core_regression.rs`: 公開 KAT と、`tests/reference/author_vectors.c` を GCC で実行した25系列×16出力の期待値を比較。保存済み期待値との再生成一致も確認。C はテスト専用でライブラリへリンクしていない。
- [Blackman/Vigna の参照実装](https://prng.di.unimi.it/)、[PCG の公開実装・例](https://www.pcg-random.org/download.html)、[JSF](https://burtleburtle.net/bob/rand/smallprng.html)、[Random123 KAT](https://github.com/DEShawResearch/random123/blob/main/tests/kat_vectors)、[Skein 1.3](https://www.schneier.com/wp-content/uploads/2015/01/skein.pdf) を参照。Threefish の非ゼロ KAT は [Bouncy Castle が公開する NIST CD の値](https://github.com/bcgit/bc-csharp/blob/master/crypto/test/src/crypto/test/Threefish256Test.cs) とも照合。
- ChaCha20 は [RFC 8439 §2.3.2](https://www.rfc-editor.org/rfc/rfc8439#section-2.3.2)。Salsa は [libsodium core3.c](https://github.com/jedisct1/libsodium/blob/master/test/default/core3.c) と [期待ハッシュ](https://github.com/jedisct1/libsodium/blob/master/test/default/core3.exp) を利用。`python tests/reference/verify_salsa.py` で各4,194,304バイトの SHA-256 が一致した。
- `src/simd_regression.rs`: 50件のテストで全 lane×256 step を比較。別途 `src/simd_extra_regression.rs` の3件と packed Xoshiro のテストを実行。scalar の参照照合を先に行い、scalar/SIMD が同じ誤りを持つだけの比較を避けた。生成元は `tests/reference/generate_lane_tests.py`。
- CET/Biski/SplitMix32 の C 期待値は、明示した漸化式の独立 unsigned-C 検算である。外部の公式仕様を発見・検証したという意味ではない。既存 MT/SFMT 全種について新たな公式 KAT を追加したわけでもない。

## 実行した検証

環境は Windows x86_64、rustc 1.98.1 (`48a229cea`, LLVM 22.1.8)。実機で AVX2 および利用した AVX-512 構成を実行した。

| コマンド / 構成 | 実行結果 |
|---|---|
| `cargo test --locked --offline` | unit **55成功**、cipher **4成功**、numeric **7成功**、doc **47成功/1 ignored**。失敗0。 |
| `cargo test --locked --offline --all-features` | unit **161**、C ABI **59**、cipher **4**、numeric **13**、sampler/rand **4**、doc **107**、unsafe API の compile-fail doc **1** が成功。合計 **349成功/1 ignored**、失敗0。 |
| `tests/reference/check_test_inventory.py` | 修正前48/default・101/all-featureの既存 unit test 名を全て保持。修正後55・161、消失0。基点の危険な処理は `--list` で列挙のみ、実行していない。 |
| `tests/reference/check_features.py` | rand/cabi/wide/simd/sampler/seedgen/暗黙thiserror の **2^7=128通り**を `cargo check --locked --offline --all-targets --no-default-features`、さらに `full` alias を確認。**129構成すべて成功**。 |
| `RUSTFLAGS='-C target-feature=+avx2'` / all-features | 全体テスト成功。unit **162**（ISA 条件付きの既存テストが追加される）。 |
| `RUSTFLAGS='-C target-feature=+avx512f,+avx512dq,+avx512vl,+avx512bw'` / all-features | 全体テスト成功。unit **170**。 |
| `RUSTFLAGS='-C target-feature=+avx512f,+avx512dq,+avx512vl'` / rand,cabi,simd,sampler,seedgen（wideなし） | unit **96成功**。 |
| 同 F/DQ/VL / all-features（wideあり） | **失敗**。`wide 1.5.0` 内部の AVX-512 条件不整合。下記の未解決事項を参照。 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --all-features --no-deps` | 成功。壊れた公開リンクも修正。 |
| `cargo build --locked --offline --release --all-features --lib --example fix-bench` | 成功。実 DLL と性能比較用 EXE を生成。 |
| `python tests/reference/verify_cabi.py` | GCC で C 呼出し側をビルド。実 DLL の **56種すべて成功/未対応CPU 0**。書込み末尾を PAGE_NOACCESS の直前に置き、0/1/7/8/15/16/17/ページ境界前後を検証。前方 canary も保持。 |
| `python tests/reference/verify_salsa.py` | Salsa20: `662b9d0e3463029156069b12f918691a98f7dfb2ca0393c96bbfc6b1fbd630a2`、Salsa8: `017421baa9959cbe894bd003ec87938254f47c1e757eb66cf89c353d0c2b68de`、いずれも一致。 |
| `git diff --check` / 変更Rustファイルの rustfmt | 成功。Git の LF→CRLF 通知はある。 |

ignored は元から README にあった外部 crate `cribler` の使用例。今回 README を doctest に含めたことで集計されるようになった。問題を隠すための削除・ignore 追加はしていない。既存の暗号 doctest も保持した。SIMD は CPU 非対応時のみ実行を避ける条件があり、今回の実機では必要 ISA を満たした。

C ABI の Rust 回帰は、39 scalar +17 SIMD の全 numeric family、各2 seed、0長、block前後、旧チャンク境界 `131071/131072/131073`、連続呼出し、前後 canary を含む。scalar は呼出しの区切りに依存せず同じ Rust 系列となる。SIMD は末尾の部分 block を全て消費する契約をテストしている。反復回数が変わる棄却サンプリングでは、余った lane を消費/破棄する。

詳細ログは作業ディレクトリの `target/fix-validation/` と `target/*-errors.log` に残した（生成物のため Git 対象外）。C oracle の再生成は `gcc -std=c11 -O2 -Wall -Wextra tests/reference/author_vectors.c -o target/author-vectors.exe` を実行し、stdout と `tests/reference/author_vectors.txt` を比較する。Windows PowerShell の出力リダイレクトでは UTF-16 化に注意し、ASCII/UTF-8 で保存する。

## 互換性への影響

1. **Rust の C ABI 呼出しはソース非互換**: free/fill に `unsafe` が必要。C の関数名・引数型・呼出し規約は保持し、既存の `new` で取得した handle を対応する `free` へ返す。安全性のため必要な変更。`examples/cabi/test.rs` も対応した。
2. **乱数系列は非互換**: 誤ったアルゴリズム、seed/zero-state、初期ゼロ、wide の重複、C の再シード、数値変換を修正したため、同じ seed でも修正前1.0.0の結果と異なる。新実装内での明示 seed の再現性を検証した。既存保存状態の binary interchange は保証しない。
3. **C bulk と SIMD の消費規則**: scalar は逐次系列へ統一。SIMD は lane 順に出力し、呼出し末尾の未使用 lane は破棄する。端数で区切ると一括呼出しと消費語数が変わる。バッファ長に応じた並列再シードは行わない。
4. **数値/重み入力の契約**: Rust は不正範囲・不正重みで panic、C bulk は不正範囲を no-op とする。一様な整数/index は棄却により可変数の語を消費する。低レベル Word の1語変換には偏りがあり、その契約を明示した。
5. **機能/entropy**: 既存 feature 名と public 型名を保持。x86_64 専用 intrinsic 型は他 architecture では公開しない。ChaCha/Salsa の `new(u32)`/`Default` は便利な決定的/time seed API として残すが、秘密鍵 entropy としては扱えない。full-key API でも鍵生成と key/nonce の一意性は呼出し側の責任。既存 rand adapter の4バイト Seed 型は保持し、byte order は little-endian に固定した。
6. crate の version は作業対象の1.0.0のまま。上記の非互換を伴うため、この変更を既存1.0.0と同一の系列互換更新として配布しないこと。

## 性能比較

測定コードは同一の `examples/fix-bench.rs`。基点を `target/fix-baseline` に隔離し、ベンチだけ同じものをコピーして比較した。raw CSV、コード SHA-256、再現手順、条件は [benchmarks/README.md](benchmarks/README.md) を参照。

同一 CPU（AMD64 Family 26 Model 68 Stepping 0、16 logical processors）、同一 compiler/dependencies、release/all-features、opt3/fat-LTO/1 codegen unit、追加 RUSTFLAGS なし。2,000,000出力×9回の中央値、1回 warmup、seed 123。wide は全 lane を数え、C ABI は16 MB出力。出力を black_box で観測。コンパイルやテストを同時実行していない。

| 対象 | 修正前 ns/出力 | 修正後 ns/出力 | 後/前（時間比） |
|---|---:|---:|---:|
| SplitMix64 | 0.567650 | 0.576300 | 1.015 |
| Xoshiro256Pp | 0.768050 | 0.676900 | 0.881 |
| Sfc32 | 0.567250 | 0.565650 | 0.997 |
| Sfc64 | 0.535600 | 0.600400 | 1.121 |
| Philox32 | 3.990900 | 1.684150 | 0.422 |
| Pcg32 | 0.765600 | 0.766050 | 1.001 |
| ChaCha20 | 6.594700 | 4.026800 | 0.611 |
| Salsa20 | 6.858550 | 3.662950 | 0.534 |
| Sfc32 randi(0,1e9) | 0.765750 | 1.357800 | 1.773 |
| wide SplitMix32x16 | 0.296800 | 0.645200 | 2.174 |
| C ABI SplitMix64 /16 MB | 0.142650 | 0.758900 | 5.320 |

- C ABI の約5.32倍の時間は、並列再シード/NT-store kernel を外し、状態を正しく逐次更新する経路へ変更したコスト。この1ケースを全 C ABI 型の性能差へ一般化しない。
- wide SplitMix の約2.17倍は、正しい scalar finalizer と重ならない block 系列を優先した結果。一様整数の約1.77倍も、偏りのある固定語変換を棄却法へ置き換えたコストとして受け入れる。
- Philox/ChaCha/Salsa は元の計算が仕様と異なり、round 数や状態処理も変わる。時間短縮を「同じ正しい処理の高速化」とは解釈しない。変更していない Xoshiro256Pp の差もあり、LTO/配置/測定環境の影響は分離できない。
- CPU affinity/周波数は固定しておらず、信頼区間は算出していない。小差を性能改善/悪化と断定しない。元の危険な Ssx2 は測定しなかった。元READMEのベンチ全項目/他マシンは再測定していない。

## 修正できなかった問題・検証限界・残存リスク

- **未解決のビルド構成**: `wide 1.5.0` を有効にし、`RUSTFLAGS=-C target-feature=+avx512f,+avx512dq,+avx512vl`（BWなし）とすると依存先で E0560/E0609。`i32x16` が `avx512` 表現になる一方、`abs`/`unsigned_abs`/`i16x32::dot` などの BW 条件の fallback が `a/b` フィールドへアクセスする。依存先を変更・vendor してはいない。**この構成を成功/修正済みとは扱わない**。通常 target flags の129 Cargo feature構成、および BWを含めた all-feature/AVX2 は成功。限定 ISA ビルドを使う場合は、対応 CPU で BW も有効にするか wide を外す。未対応 CPU に BW を指定する対処は不可。
- **未実行**: Linux、aarch64/32-bit/wasm、big-endian、Rust 1.92 MSRV、ISA 非対応実機/エミュレータ、Miri、Rust sanitizer、C#/GC からの利用。インストール済み標準 target は Windows x86_64 のみ。Linux の README パス問題は静的に修正したが、Linux build 成功と報告しない。
- **乱数品質/暗号安全性**: PractRand/TestU01/大規模な lane 間相関試験、暗号監査、side-channel/鍵消去の検証は未実施。特に CET/Biski/custom SplitMix32 の数学的品質や period を今回の回帰結果から証明できない。既知解一致は独立した暗号安全性評価の代わりではない。
- **FFI の限界**: null/alignment/overflow を検査しても、任意の非null pointer の provenance、確保範囲、use-after-free、double-free、同時アクセスの健全性は判定できない。契約違反は呼出し側の責任であり、`unsafe` 化はそれを明示するもの。guard page/canary は全ての UB を証明的に排除する検証ではない。
- **性能/移植性**: C ABI の並列・NT 最適化を復活させる場合は、正しい stream 分割、全 lane/boundary、fence、同じ条件の測定が必要。CPU 機能検査と scalar fallback を速度だけの理由で外さない。no_std は追加しておらず std を必要とする。

## 引き継ぎ用の進捗

- [x] 基点・既存変更を確認し、利用者の REVIEW.md を保持。
- [x] 修正前スナップショット、共通ベンチ、修正前測定を保存。
- [x] メモリ/FFI/ISA、系列重複、core/暗号、初期化/carry、数値/sampler/rand、README を順に修正し、関連テストを実行。
- [x] 独立 C/公開 KAT、全 lane 比較、実 DLL C guard-page 試験。
- [x] 全体/doctest/129 feature構成、AVX2/AVX-512 検証（上記の依存先の失敗構成は区別）。
- [x] 同条件の前後性能比較・互換性・未検証範囲を記録。
- [x] ローカルコミットの親が1.0.0であること、対象差分、既存テストの保持を確認。権限切替時に参照できなくなった Git 基点は公式リポジトリから再取得し、作業ファイルに変更を加えず履歴情報を復旧済み。
