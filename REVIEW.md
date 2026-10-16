## A. 総評

対象は urng 1.0.0、コミット 9f8518e05b4c7580d8a291ba679e900fb8dba0af です。追跡対象のコードは変更していません。

Rng・WordによるAPIの統一、スカラー・portable SIMD・C ABIの提供、生成処理から数値変換を分離する設計には利点があります。

一方、safe Rustから到達するメモリ破壊、並列ストリームの重複、名前付きアルゴリズムとの不一致を確認しました。現状を完成した安定版と評価するのは難しく、性能改善より先に安全性と系列の正確性を修正すべきです。

既存テストは標準構成・全feature構成とも通ります。ただし、多くは同じ実装を同じシードで比較するテストであり、以下の欠陥を検出できていません。

## B. 発見した問題

以下では、実行で再現したものと、コード・仕様の照合で確定したものを区別しています。

### 確認済みの欠陥

1. 16バイトの配列へ64バイトを書き込む

- 重要度：Critical／場所： src/prng/b64/xoshiro.rs:169 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/xoshiro.rs:169)／確信度：高
- 問題： Xoshiro256Ssx2::nextu()が [u64; 2] に _mm512_stream_si512 で書き込みます。
- 発生条件・影響： 対応CPUでこのsafeメソッドを呼ぶと、16バイトの領域に64バイトを書き込むため、領域外書き込みになります。配列には要求される64バイト境界へのアラインメントも保証されません。
- 根拠： 配列サイズと書き込み幅の静的照合。intrinsicの仕様でも512ビットの書き込みと64バイト境界が要求されています。危険な呼び出しは実行していません。
- 改善案： 正しいサイズの一時領域へ通常の非整列ストアを行い、必要な2レーンを取り出す。なお、この関数は状態更新にもidentity shuffleとのXORなどの誤りがあり、ストアだけ直して完了とはできません。

2. C ABIのポインタ操作がsafe Rust APIとして公開されている

- 重要度：High／場所： 例：src/cabi/prng/b32/xorshift.rs:13 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b32/xorshift.rs:13)、同ファイルの生成関数群／確信度：高
- 問題： pub extern "C" fn 内で、生ポインタの参照化、スライス生成、Box::from_rawを行います。
- 発生条件・影響： Rust利用者がunsafeを一切書かずに、不正ポインタの参照や二重解放を起こせます。
- 根拠： #![forbid(unsafe_code)]の検証コードで、同じポインタをxorshift32_freeへ2回渡すコードがコンパイルできました。二重解放自体は実行していません。
- 改善案： 呼び出し側にポインタの有効性などを要求する関数をpub unsafe extern "C" fnにし、所有権・領域サイズ・非エイリアス条件を記載する。Rust向けには所有権を管理するラッパーを用意する。

3. SIMDで必要なCPU機能を保証できていない

- 重要度：High／場所： src/prng/b32/jsf.rs:106 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b32/jsf.rs:106)、src/prng/b32/xoroshiro.rs:109 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b32/xoroshiro.rs:109)、src/rngv.rs、Sfc64x8::newなど／確信度：高
- 問題： AVX2版と説明する処理で_mm256_rol_epi32や_mm256_mask_blend_epi32を使っています。また、実行時検査もtarget_feature注釈もないsafe関数からAVX-512処理を呼ぶ箇所があります。
- 発生条件・影響： AVX2のみのCPUなどで、説明された条件を満たしていても未対応命令に到達します。
- 根拠： _mm256_rol_epi32はAVX-512FとAVX-512VLが必要です。未対応featureの実行はRustの仕様上も未定義動作です。未対応CPUでの実行再現はしていません。
- 改善案： AVX2版のrotate/blendをAVX2命令で実装する。公開safe APIでは必要な全featureを検査し、フォールバックするか、構築に成功した型が実行条件を保証する設計にする。

4. 特殊化したnon-temporal store経路に必要なfenceがない

- 重要度：High／場所： 例：src/cabi/prng/b64/xoshiro.rs:259 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b64/xoshiro.rs:259)、src/cabi/prng/b64/cet.rs:370 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b64/cet.rs:370)、biski.rs、b32のxoshiro/xoroshiroにも該当／確信度：高
- 問題： streaming storeを行うワーカーに_mm_sfence()がありません。
- 発生条件・影響： NT経路が選択される大きな整列済みバッファで、書き込み後のアクセスに必要な順序付けを保証できません。
- 根拠： 呼び出し経路を静的確認しました。Rustのintrinsic仕様は、変更したメモリへの次のアクセス前に、書き込んだスレッド自身でfenceを要求しています。可視性の異常を実測したわけではありません。
- 改善案： 各ワーカーのNT書き込み終了時にfenceを置く。既にfenceを実装している共通ヘルパーへの集約も有効です。

5. 隣接する並列チャンクで同じストリームを再利用する

- 重要度：High／場所： src/cabi/prng/b64/xoshiro.rs:167 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b64/xoshiro.rs:167)、src/cabi/prng/b64/cet.rs:269 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b64/cet.rs:269)、src/cabi/prng/b64/biski.rs:157 (D:/.repo/urng-review_gpt-6.1-sol/src/cabi/prng/b64/biski.rs:157)／確信度：高
- 問題： ストリームのシード入力が実質的にbase + (chunk_idx + lane_idx) * strideです。チャンク番号とレーン番号の組を区別できません。
- 発生条件・影響： 複数チャンクを生成すると、別のチャンクに同一系列が現れます。
- 根拠： 実行で次を確認しました。

  | C ABI関数 | 確認した重複 |
  |---|---|
  | xoshiro256ssx2_next_u64s | チャンク0の[1..32]とチャンク1の先頭31個が一致 |
  | cet64x8_next_u64s | 同様に63個が一致 |
  | biski64x8_next_u64s | 同様に47個が一致 |

- 改善案： chunk_idx * streams_per_chunk + lane_idxなど、組ごとに異なるIDを使う。大きなバッファでチャンク間の系列重複を検査する。

6. wide版SplitMixの連続ブロックが重なる

- 重要度：High／場所： src/wide/splitmix.rs:33 (D:/.repo/urng-review_gpt-6.1-sol/src/wide/splitmix.rs:33)、同:60 (D:/.repo/urng-review_gpt-6.1-sol/src/wide/splitmix.rs:60)／確信度：高
- 問題： レーンの初期状態をgamma間隔に配置し、毎回全レーンをgammaだけ進めます。
- 発生条件・影響： 幅Nの隣接ブロックでN−1個が重複します。
- 根拠： SplitMix32x4::new(1)の最初の2ブロックで、a[1..] == b[..3]を実行確認しました。x8/x16も同じマクロから生成されます。
- 改善案： 連続した単一系列をブロック化するなら、各回にN * gamma進める。出力順序も明文化する。

7. ChaCha/Salsaが出力から将来を計算できる構造になっている

- 重要度：High／場所： src/crng/b32/chacha.rs:55 (D:/.repo/urng-review_gpt-6.1-sol/src/crng/b32/chacha.rs:55)、src/crng/b32/salsa.rs:45 (D:/.repo/urng-review_gpt-6.1-sol/src/crng/b32/salsa.rs:45)／確信度：高
- 問題： 変換後の状態xをそのまま出力し、それを次の内部状態にします。標準の鍵・カウンタ構成やfeed-forwardがありません。ROUNDS回のループも、各回で2ラウンド実施しています。
- 発生条件・影響： 1ブロックの出力を得ると、シードなしで以後のブロックを計算できます。ChaCha/Salsaの暗号的性質を期待して使用できません。
- 根拠： ChaCha20の16語を観測し、その値だけから次の16語を完全に予測できることを実行確認しました。RFC 8439 §2.3.1の構成とも一致しません。
- 改善案： 標準の状態配置・カウンタ更新・ラウンド数・feed-forwardへ修正し、既知解で検証する。暗号用途を提供するなら、32ビット整数だけに限定しない鍵の入力APIも必要です。

8. 複数の名前付きアルゴリズムに実装上の相違がある

- 重要度：High／場所：下表／確信度：高
- 問題： 以下は単なるシード形式の違いではありません。urng側のシード展開を再現し、同じ初期状態から比較しました。
- 発生条件・影響： 該当ジェネレータを使用すると、標準アルゴリズム、または対応するスカラー版と異なる系列になります。標準アルゴリズムの評価結果をそのまま適用できません。

  | 場所 | 確認した誤り・根拠 |
  |---|---|
  | Sfc64:47 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/sfc.rs:47) | 本来のrotl(old_c, 24) + resultがrotl(result, 24)になっています。同じ初期状態で3語目から不一致。SIMD版にも同じ式があります。Rand側実装と照合しました。 |
  | Xoroshiro128Pp:86 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/xoroshiro.rs:86) | 出力が単なるs0 + s1で、++のrotateと加算がありません。状態遷移も作者の実装と異なります。seed=1の最初の値から不一致。 |
  | Philox32:68 (D:/.repo/urng-review_gpt-6.1-sol/src/cbrng/b32/philox.rs:68) | キー更新が加算ではなく乗算です。また1マクロで2ラウンド実施します。最初のブロックがRandom123の4x32-10と不一致。 |
  | Philox32x4x4:209 (D:/.repo/urng-review_gpt-6.1-sol/src/cbrng/b32/philox.rs:209) | _mm512_mul_epu32が参照する各64ビットレーンの下位32ビットに同じ乗数を配置しており、必要なM0/M1の配置になっていません。最初の4語を参照実装と比較して不一致を確認しました。 |
  | Xoroshiro64Ssx8/Ssx16:109 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b32/xoroshiro.rs:109) | 出力スクランブラのrotl(..., 5) * 5が欠落しています。作者の式と照合し、x16のレーン0で不一致を実行確認しました。 |
  | Biski64x8:126 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/biski.rs:126) | loop_mixを旧状態ではなく更新済みのfast_loop・mixから計算します。同じ初期状態のスカラー版と2語目から不一致。C ABIの特殊化経路にも同じ更新順があります。 |

- 改善案： 各式・レーン配置を修正し、初期化の違いを除いたスカラー対SIMD比較と、独立した参照実装による既知解テストを追加する。統計品質の低下量や実際の周期は、今回測定していません。

9. 大文字小文字を区別する環境ではビルドできない

- 重要度：High／場所： src/lib.rs:1 (D:/.repo/urng-review_gpt-6.1-sol/src/lib.rs:1)／確信度：高
- 問題： include_str!("../Readme.md")に対し、Gitに登録されたファイル名はREADME.mdです。
- 発生条件・影響： 通常のLinuxなど、大文字小文字を区別するファイルシステム上のビルドが失敗します。
- 根拠： include先とGit treeのファイル名を照合しました。Linux上での実行検証はしていません。
- 改善案： include先を../README.mdに修正し、LinuxでビルドするCIを追加する。

10. 初期化でアルゴリズム固有の状態条件を満たさない

- 重要度：Medium／場所：下表／確信度：高
- 問題・発生条件・影響・根拠：

  | 場所 | 確認結果と影響 |
  |---|---|
  | Pcg32::new:27 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b32/pcg.rs:27)、SIMD版、wide/pcg.rs | incが偶数になり得ます。seed=1では0x72619cb5a587ac1aでした。状態の偶奇が固定され、記載された2^64周期を満たせません。PCGの参照実装では増分を奇数にします。 |
  | Xorshift64::new (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/xorshift.rs) | seed=0x61c8864680b583ebで展開結果が0になり、以後も0に固定されます。実行確認済みです。 |

- 改善案： PCGの増分を奇数にする。Xorshiftでは展開後のゼロ状態を検出して非ゼロへ置き換える。入力seedを奇数化するだけでは、これらの条件を保証できません。

11. 初期バッファのゼロを乱数として返す

- 重要度：Medium／場所： Threefish256:83 (D:/.repo/urng-review_gpt-6.1-sol/src/cbrng/b64/threefish.rs:83)、Philox4x64:129 (D:/.repo/urng-review_gpt-6.1-sol/src/cbrng/b64/philox.rs:129)／確信度：高
- 問題： バッファが未生成なのに、posが「残りあり」を示します。
- 発生条件・影響： Rng::nextu()でThreefish256は最初の4語、Philox4x64は最初の2語が、シードに関係なく0になります。
- 根拠： 初期化とリングバッファの条件を照合し、seed=1で実行確認しました。
- 改善案： 両方とも初期posをバッファ長の4にする。

12. long_jump()が指定された距離を進めない

- 重要度：Medium／場所： src/prng/b64/xoroshiro.rs:55 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/xoroshiro.rs:55)、同:148 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/xoroshiro.rs:148)／確信度：高
- 問題： マクロ内部で全係数を走査し、途中で状態を置き換え、そのマクロをさらに2回呼びます。
- 発生条件・影響： 2^96ステップのジャンプにならず、ストリーム分割の前提が崩れます。
- 根拠： 正常な状態遷移を持つXoroshiro128Ssでも、作者のlong-jumpと不一致を実行確認しました。同じ比較方法で通常のjump()は一致しました。
- 改善案： 係数を一度だけ走査し、全走査後に一度だけ状態を代入する。

13. wide版Threefryの上位カウンタにcarryしない

- 重要度：Medium／場所： src/wide/threefry.rs:100 (D:/.repo/urng-review_gpt-6.1-sol/src/wide/threefry.rs:100)、同:255 (D:/.repo/urng-review_gpt-6.1-sol/src/wide/threefry.rs:255)／確信度：高
- 問題： c0だけを加算し、c1以降を更新しません。
- 発生条件・影響： 各レーンは2^32ブロックで同じカウンタに戻り、系列が反復します。
- 根拠： カウンタ更新の静的確認です。2^32ブロックの実生成は行っていません。
- 改善案： スカラー版と同様に上位語へcarryする。最大値付近の状態から境界テストを行う。

14. シャッフルが一様な置換を生成しない

- 重要度：Medium／場所： src/shuffle.rs:46 (D:/.repo/urng-review_gpt-6.1-sol/src/shuffle.rs:46)、shuffled()／確信度：高
- 問題： 各位置の交換先を、毎回スライス全体から選んでいます。
- 発生条件・影響： 3要素以上で置換の確率が偏ります。
- 根拠： 交換先を理想的に一様と仮定しても、3要素の全9経路には、1回現れる置換と2回現れる置換があります。実コードで全経路を列挙して確認しました。
- 改善案： Fisher–Yatesとして、位置iでは未処理区間[i, len)から交換先を選ぶ。両メソッドを共通化する。

15. 有効な整数範囲でpanicまたは定数出力になる

- 重要度：Medium／場所： src/rng.rs:59 (D:/.repo/urng-review_gpt-6.1-sol/src/rng.rs:59)、C ABIのSIMD整数範囲変換／確信度：高
- 問題： スカラー版はオフセットを狭い符号付き型へ変換してからminを加算します。SIMD版には幅2^32をu32へ切り詰める処理があります。
- 発生条件・影響： 整数全域などの正当な指定で、スカラー版はoverflow checks有効時にpanicします。SIMD版は範囲幅が0になります。
- 根拠： u32::MAX.to_randi(i32::MIN, i32::MAX)とu64版のpanicを確認しました。sfc32x4_rand_i32sの全i32範囲指定では、32出力すべてがi32::MINでした。
- 改善案： 加算まで広い符号付き型で計算してから変換する。SIMDでは全域指定を特別扱いする。

16. multiply-shiftだけでは「一様」の契約を満たさない

- 重要度：Medium／場所： src/rng.rs:56 (D:/.repo/urng-review_gpt-6.1-sol/src/rng.rs:56)、to_index()／確信度：高
- 問題： 整数範囲変換に棄却処理がありません。
- 発生条件・影響： 範囲幅が元の語数を割り切らないと、各出力に対応する入力数が異なります。大きな範囲では偏りも小さくありません。
- 根拠： 幅3 * 2^29というi32内の範囲で、先頭3値の入力対応数は3, 3, 2です。式から厳密に求められます。
- 改善案： Rng側で追加の乱数取得と棄却を行う。一語だけの高速変換を残すなら、近似的一様であることをAPI名・文書で区別する。

17. SSE版の整数変換で奇数レーンに積の下位語を返す

- 重要度：Medium／場所： src/rngv.rs:36 (D:/.repo/urng-review_gpt-6.1-sol/src/rngv.rs:36)／確信度：高
- 問題： __m128i::to_randi()のshuffleが、奇数レーンでは積の上位32ビットを選択していません。
- 発生条件・影響： 指定範囲外の整数を返します。
- 根拠： 入力全レーン1、scale全レーン3、min全レーン0で、結果が[0, 3, 0, 3]でした。期待値は全レーン0です。
- 改善案： 偶数・奇数それぞれの積の上位語を正しく合成する。レーンごとに異なるscaleも検証する。

18. TwistedGFSRと共通のWord変換が整合しない

- 重要度：Medium／場所： src/prng/b64/twisted_gfsr.rs:69 (D:/.repo/urng-review_gpt-6.1-sol/src/prng/b64/twisted_gfsr.rs:69)、src/choice.rs:8 (D:/.repo/urng-review_gpt-6.1-sol/src/choice.rs:8)／確信度：高
- 問題： Word = u64ですが、nextu()はu32をゼロ拡張した値しか返しません。
- 発生条件・影響： 64ビット分の出力を前提にするto_index()と組み合わせると、長さ4のchoice()は常に先頭要素を選びます。シャッフルも影響を受けます。
- 根拠： 式では(u32値 * 4) >> 64 == 0です。実際のジェネレータで確認しました。nextf/randiの独自overrideでは、この共通API側の問題を解消できません。
- 改善案： Word = u32にするか、2語を結合して本当に64ビットの出力を作る。

19. weighted samplerに重み処理の欠陥がある

- 重要度：Medium／場所： Alias32:39 (D:/.repo/urng-review_gpt-6.1-sol/src/sampler/b32/alias.rs:39)、Bst32:53 (D:/.repo/urng-review_gpt-6.1-sol/src/sampler/b32/bst.rs:53)、各64版／確信度：高
- 問題・発生条件・影響・根拠：

  | 実装 | 確認結果 |
  |---|---|
  | Alias32/64 | 有限・正の重み[MAX, MAX]でも合計が∞になり、確率表が壊れます。両幅で1000回すべてindex 0になりました。 |
  | Bst32/64 | 累積値と乱数が等しい場合に、その位置を返します。重み[0, 1]で乱数0を与えると、重み0のindex 0を選択しました。 |

- 改善案： Aliasは最大重みでのスケーリングなどでオーバーフローを避ける。Bstは「累積値が乱数より厳密に大きい最初の位置」を選択する。

20. rand featureの64ビット型に乱数取得traitが実装されていない

- 重要度：Medium／場所： src/rand/b64/mod.rs:3 (D:/.repo/urng-review_gpt-6.1-sol/src/rand/b64/mod.rs:3)、src/_internal.rs:663 (D:/.repo/urng-review_gpt-6.1-sol/src/_internal.rs:663)／確信度：高
- 問題： 64ビット型にはSeedableRngだけを実装し、rand_core::TryRngを実装していません。
- 発生条件・影響： seed_from_u64()で生成できても、Randの乱数APIには渡せません。
- 根拠： 全feature有効のライブラリで、Xoshiro256Pp::seed_from_u64(1).next_u64()に相当する利用例が、TryRng未実装によるコンパイルエラーになりました。
- 改善案： 64ビット出力用のTryRng実装を追加し、Rand側の汎用関数へ渡すコンパイルテストを置く。

21. randf()が半開区間の上端を返す

- 重要度：Low／場所： src/rng.rs:63 (D:/.repo/urng-review_gpt-6.1-sol/src/rng.rs:63)／確信度：高
- 問題： 最後の乗算・加算の丸めにより、maxに達します。
- 発生条件・影響： 特に狭い浮動小数点範囲で、文書の[min, max)に違反します。
- 根拠： min=1.0f32、max=f32::from_bits(1.0f32.to_bits()+1)、生の乱数u32::MAXで、結果がmaxになりました。
- 改善案： 有効な境界条件を定義し、丸めで上端へ達した場合の補正を加える。

22. READMEのportable wide利用例がコンパイルできない

- 重要度：Low／場所： README.md:90 (D:/.repo/urng-review_gpt-6.1-sol/README.md:90)／確信度：高
- 問題： 実在しないurng::wide::WRngをimportしています。実装はRngWです。
- 発生条件・影響： 掲載例をコピーするとコンパイルエラーになります。
- 根拠： wide有効のライブラリに対してコンパイルし、失敗を確認しました。
- 改善案： 名前を修正し、READMEの例も検証対象にする。現在はlib.rsのcfg_attr(not(doctest), …)によってREADMEがdoctestから外れています。

### 追加検証が必要な懸念

wideベンチマークの最適化による測定対象消失

- 重要度：Low／場所： examples/wide/test.rs (D:/.repo/urng-review_gpt-6.1-sol/examples/wide/test.rs)のmeasure_wide・bench_wide／確信度：中
- 問題： 出力バッファや最終状態をblack_boxなどで観測していません。
- 発生条件・影響： LTOなどの条件によって、不要と判断された計算・ストアが消える可能性があります。
- 根拠： ソース上の懸念です。生成アセンブリや測定値で消去を確認したわけではありません。
- 改善案： 出力を観測可能にし、releaseの生成コードで測定ループが残ることを確認する。

## C. 改善提案

優先順位は次のとおりです。

1. 最優先：独立した正解との比較を追加する。
   現在の決定性テストに、作者の既知解、同一初期状態でのスカラー対SIMD比較、チャンク間の重複検査を加えるべきです。統計検定だけでは実装の一致を証明できません。
2. 次点：系列と再現性の契約を定義する。
   Rust APIとC ABI、分割したfillと一括fill、スカラーとSIMDで、どこまで同じ系列を保証するか明記してください。C ABIの再シード方式では、呼び出しサイズによって系列が変わる型があります。
3. 次点：ベンチマークの比較条件を記録する。
   CPU、メモリ、rustc、RUSTFLAGS、Rayonスレッド数を記載し、単一系列の取得・単一スレッドのブロック生成・並列fillを区別する。大きなバッファの帯域測定だけで演算性能を順位付けしないことが重要です。
4. その後：特殊化処理の重複を減らす。
   数値変換、tail処理、NT store、fenceが複数ファイルに重複し、実際に修正漏れが生じています。共有化で性能が落ちないかは、生成コードと測定で判断してください。
5. 必要に応じて：no_stdを独立した設計課題にする。
   現状は時刻・同期・エラー型などでstdが必要です。wrapnがno_stdであってもurng全体は対応していません。対応するなら、生成コア、allocを使う機能、時刻・並列処理を分離する方針が現実的です。

## D. 評価できる設計

- sealedなWordで数値型の組を閉じている。
  u32/f32/i32、u64/f64/i64の関係を一か所で定義し、外部実装による不整合を防いでいます。変換式を修正した際に効果を集約できる構造です。
- RngとSeedの責務を分けている。
  乱数取得だけを実装する外部ジェネレータも受け入れられ、初期化方法を一律に強制しません。
- ホットパスの多くが固定サイズの状態とwrapping演算で構成されている。
  wrapn 0.10.0の実装も確認しました。通常の状態更新の桁あふれを、一般的な整数オーバーフローバグとして扱う必要はありません。多くの生成処理でヒープ確保もありません。
- SplitMix64のC ABIは、カウンタから並列位置を直接計算できている。
  スカラー版との出力・継続状態を、端数とチャンク境界を含む32ケースで比較し、すべて一致しました。
- 共通のfillヘルパーには具体的な境界対策がある。
  非整列時のフォールバック、小さいバッチの処理、tail処理、NT store後のfenceが実装されています。この設計を特殊化経路にも適用する価値があります。

## E. 検証状況と限界

実際に読んだ範囲

- Cargo設定、依存関係、build.rs、README、bench.md、実行スクリプト。
- 公開trait、数値変換、choice、shuffle、sampler、seedgen、Rand連携。
- PRNG・counter-based・cipher-based・wideの各ファミリーの主要な初期化・生成処理。
- C ABIの共通処理と、本報告で指摘した特殊化経路。すべてのC ABI関数を個別に精査したわけではありません。
- 外部クレートのwrapn、rand_core、rand_sfcの関連実装。対象版の直接依存にurng_coreはなく、traitはリポジトリ内にあります。

実行した検証

環境はWindows x86_64、rustc 1.98.1。実行時検査でAVX2・AVX-512F/VL/DQ対応を確認しました。

| コマンド・検証 | 結果 |
|---|---|
| cargo test --locked --offline | 48 unit tests、44 doctests成功 |
| cargo test --locked --offline --all-features -q | 101 unit tests、104 doctests成功 |
| cargo check --locked --offline --all-features --all-targets | 成功 |
| 本報告の短い再現プログラム | 記載した出力・panic・系列重複を確認 |
| safe FFIからの二重解放例 | コンパイルのみ確認、実行なし |
| SplitMix64のスカラー対C ABI比較 | 32ケース成功 |
| 最終git diff・status | 追跡対象の変更なし |

実行していない検証・保留事項

- ベンチマーク本体と性能改善案の計測。
- メモリ破壊箇所の実行、Miri、sanitizer。
- Linux・AArch64・AVX2のみのCPU、宣言MSRVのRust 1.92での実行。
- PractRand、TestU01、criblerなどの統計検定、長周期の実測。
- アルゴリズムを変更した箇所の統計品質・周期の定量評価。
- Threefry/Threefishにある標準と異なるtweak・feed-forward処理が、意図的な派生方式かどうかの判断。

既存テストの成功は確認できましたが、ライブラリ全体の正確性・安全性を保証する結果ではありません。特に1〜8の修正と、それを検出する独立した検証がリリース前の優先事項です。