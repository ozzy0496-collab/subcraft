# Element 3D 連携（SubCraft-E3D fork）

上流: https://github.com/Aalzard/subcraft （調査時点 63fb47d / v0.2.1）
前身: E3D Texture Lab（E:\personal\AI\substance\E3D_Texture_Lab.html, DEV_NOTES.md）

## E3D の仕様（実機で確認済み）

- テクスチャスロット: Diffuse / Reflectivity / Glossiness / Illumination / Normal Bump / Occlusion / Environment。
  Specular スロットは無い。metal/rough 方式ではない。
- MTL から自動で読まれるのは map_Kd→Diffuse、map_Ns→Glossiness、map_bump→Normal Bump のみ。
  map_Ks・map_Ka は読まれない。Reflectivity / Occlusion / Illumination は手動割り当て。
- E3D 自身の OBJ 書き出しの MTL には map_Kd / map_Ks / bump / refl だけが出る。
- E3D が書き出した OBJ を E3D に読み込むと Y 軸 180° 逆・2.5 倍になる
  → 書き出し時に Y180° 回転・0.4 倍すると元と一致。
- usemtl はマテリアル名そのまま。プリセットを当てるとプリセット名になり、同じプリセットのマテリアルは同名で区別できない。
- 既定で 2048px 超のテクスチャは縮小される（Preferences の Map Size Limit で 8192 まで）。
- 押し出しモデルは重なった見えない面が多い。

## 方針

- 上流の main に追従しやすいよう、E3D 固有のコードは新規ファイル（`e3d.rs`）に寄せ、既存ファイルには登録・分岐の数行だけを足す。
- 作業ブランチは `e3d`。`main` は上流と同じ状態に保つ。
- CLAUDE.md のアーキテクチャ規則（依存方向・テスト必須・progress.md への記録）に従う。

## 改造ポイント（優先順）

### 1. E3D 書き出しテンプレート（小）

- `crates/export/src/template.rs`: `ExportTemplate::element_3d()` を追加し、`builtin()` に登録する。
  - Diffuse      = BaseColor RGB、sRGB
  - Glossiness   = Roughness を反転（`Source::Channel { kind: Roughness, invert: true }`）、gray
  - Reflectivity = Metallic、gray（※E3D の Reflectivity が「金属度」として妥当かは実機で比較）
  - Occlusion    = AmbientOcclusion、gray
  - Illumination = Emissive RGB、sRGB
  - NormalBump   = `normal(false)`（OpenGL +Y）。※E3D が +Y / −Y どちらかを最初に実機で検証する
- オプション: 金属部の Diffuse を暗くする（BaseColor×(1−Metallic×k)）。既存の `Source` では表現できないので、
  `Source::E3dDiffuse` のような変種を足すか、`render_maps`（`crates/export/src/lib.rs`）に後処理フックを入れる。
- 2048px を超える書き出しでは「E3D の Map Size Limit を上げてください」と UI に警告を出す。

### 2. E3D 向け OBJ/MTL 書き出し（中）

- 既存: `crates/export/src/scene.rs` の `SceneFormat::ObjMtl` / `obj_text` / `mtl_text`。
  MTL は map_Kd / map_Ke / map_Pr / map_Pm / map_Ka / norm を書く → E3D はほぼ読まない。
- 追加: `SceneFormat::E3dObj`。
  - MTL: `map_Kd`（Diffuse）、`map_Ns`（Glossiness）、`map_bump`（Normal）だけを書く。
    `SceneMaterial` に `glossiness` を足す（または E3D 用の構造体を別に持つ）。
  - その他のマップは同じフォルダに `{mesh}_{set}_{Reflectivity|Occlusion|Illumination}.png` として出す。
  - 位置補正オプション「E3D から書き出した OBJ を元の位置に戻す」= 頂点を Y180°回転（x,z を反転）＋0.4 倍。
    法線も同じ回転をかける。
- 2 は 3（UV 変更）を E3D に戻す手段でもあるので、3 より先に作る。

### 3. E3D 出力 OBJ の読み込み（中）

- 既存: `crates/mesh/src/import.rs` の `load_obj`。tobj で `o` ごとに model を作り、
  `b.material(&mat_name)` で同名マテリアルは同じスロット（=同じテクスチャセット）に統合される。
- 問題: 同じプリセットを当てたマテリアルは同名なので、1 つのテクスチャセットに潰れる。
- 対策（インポートオプション）: 「マテリアル名を object 名で分ける」→ スロット名を `{object}_{usemtl}` にする。
  逆に統合したい場合は今のままでよい。Parts 機能（object 名・material ID 単位の選択）はそのまま使える。
- 書き出し時に元の usemtl 名へ戻すかどうかも設定にする（E3D 側でマテリアル名で割り当て直すため）。

### 4. 見えない面の UV 縮小（大）

- 判定: `subcraft_mesh::bvh` と、bake の AO が使っている半球サンプリング（`crates/bake/src/surface.rs` の
  `cosine_directions`）を流用する。各三角形の数点からレイを飛ばし、「外に抜けるレイが 1 本も無い」三角形を不可視とする。
  外側の球面からメッシュに向けて撃つ方式のほうが、押し出しの内側面には確実。
- 再配置: `crates/mesh/src/unwrap.rs` の `pack()`（チャートを単位正方形に詰める）を使う。
  - A案: 既存 UV のアイランドはそのまま形を保ち、不可視アイランドだけスケールを例えば 1/8 にして再パックする（E3D の UV を壊さない、推奨）
  - B案: `unwrap()` で作り直し、不可視の三角形の面積の重みを下げる（`balance_density` を三角形ごとの重みに拡張）
- UV を変えると描いたテクセルがずれるため、「読み込み直後・ペイント前」の前処理コマンドとする
  （ペイント済みなら、旧 UV → 新 UV へ転写する。`crates/mesh/src/transfer.rs` が使えるか要確認）。
- 不可視判定の結果は Parts の選択として見せて、手で修正できるようにする。

## 実機で検証すること

- [ ] E3D の Normal Bump は +Y（OpenGL）か −Y（DirectX）か
- [ ] Reflectivity に Metallic をそのまま入れた見た目が妥当か
- [ ] Glossiness = 1−Roughness で SubCraft のプレビューと近いか
- [ ] map_Ns に画像パスを書いて Glossiness に自動で入るか（Texture Lab では確認済み）
- [ ] SubCraft の OBJ 書き出しを E3D に読ませたとき、スケールと向きが合うか
