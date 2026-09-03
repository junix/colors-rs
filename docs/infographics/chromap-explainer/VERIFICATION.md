# VERIFICATION — chromap 图解验收记录

写给不信任本图的读者：每句话都给可重跑的命令；能低说不高说；修正记录只标注不删除。
所有命令的当前目录 = 本交付目录（`docs/infographics/chromap-explainer`），
引擎仓库 = 本目录向上三层（`colors-rs` 仓库根，下称 `$REPO`）。

## 0. 结论总表

| 检查 | 结果 |
| --- | --- |
| svg-linter 结构门禁（11 个 SVG，逐文件） | **0 findings**（错误级与警告级均为 0；无需 line-overlap 逐项判定） |
| 渲染断言 | 位图 2400×17776 = 页面 CSS 1200×8888 × dpr 2，`stitch.py` 内建断言通过 |
| 渲染确定性 | 两次独立浏览器会话渲染 sha256 完全一致（§2.3） |
| index.html 重建 | `python3 build.py` 重建后 `cmp` byte-identical（§5） |
| 页面自包含 | 零 `<script>`、零 CDN、零外部请求（§4） |
| 事实红线 | 论点数值 / 引文 / 计数 / 退出码 / 源码行号全部实机对拍（§3） |
| 目检 | 11 面板全尺寸 + 页眉页脚裁片 + 灰度 + 缩略（§6） |
| 第二轮修复（2026-09-02） | 独立对抗验证 14 条 findings 修净、0 推翻；门禁/渲染/重建全链路重跑通过（§3.6 修正 8-16、§8 迁移链） |

## 1. 证据冻结（provenance）

冻结脚本 `prep_data.py`（前提：在 colors-rs 仓库内的本目录执行——依赖
`cargo run --example design_tokens`、git 元数据与 named.rs 计数；启动即校验
上下文、全部自检通过后才落盘，见 §3.6 修正 16）；引擎身份（`data/engine.json`）：

- version `chromap 0.1.0`
- binary sha256 `39baa731a44076456d43b82527fb15bdc49663390b7554bff7bed468cd658b3a`
- repo commit `f6a58fac2f5e22603105854c39fd41f577d6f13a`（工作区干净，仅本目录 untracked）

复现：

```bash
chromap --version                 # chromap 0.1.0
shasum -a 256 "$(command -v chromap)"
ls data/*.json | wc -l            # 20（17 组命令产出 + selfchecks + engine + provenance）
python3 - <<'PY'
import json
s = json.load(open("data/selfchecks.json"))
print(f"{s['passed']}/{s['total']}")   # 21/21
PY
```

`data/provenance.json` 记录每个 JSON 的完整 argv；基色 `#4f7cff` 出自
`$REPO/crates/chromap/examples/design_tokens.rs:6`（引擎自带例程，非手选）；
跨引擎样本出自 `color-palette-rs --json show tokyo-night`。

## 2. 门禁与渲染

### 2.1 svg-linter（一次一个文件）

```bash
for f in svg/*.svg; do svg-linter --plain check "$f"; done
```

11 个文件全部 `exit=0` 且 findings 为 0（全量输出冻结在 `evidence/gate.txt`）。
首轮门禁曾报 17 条错误级 findings（p-cli 芯片文字重叠 ×1：等宽字体实测行进
≈0.633em 而估宽 0.55em；p-contrast 右缘越界 ×2 与下方标签压诚实卡；p-evidence
自检明细右缘越界 ×7；p-luminance 曲线贴轴 line-overlap ×1 与自检芯片越界 ×2；
p-spaces 三条汇聚连接线共线重叠 ×2；p-tokens CSS 行距压字 ×1），全部以几何修正
清零（芯片估宽提到 0.65em、标签块右缘翻转、入口点错开、行距下移），不是豁免。

### 2.2 渲染管线（切片，从 y=0）

```bash
node shoot.js "file://$PWD/index.html" render   # 等 fonts.ready + 双 rAF 后拍摄
python3 stitch.py
```

- shoot.js：`document.fonts.ready` + 双 `requestAnimationFrame` 之后才截图；
  切片 = 从 `y=0` 起、全宽 1200、定高 3600 CSS px；每片拍摄前把滚动位置断言到
  目标 y（误差 ≤1px，否则报错退出）。
- stitch.py：**只按切片顺序拼接**（逐 section 拼接会丢段间距，属明令禁止项），
  并断言 `位图高 == cssHeight × dpr`：当前 `2400×17776 == 1200×8888 × 2`。

### 2.3 渲染确定性

两次**独立会话**渲染 sha256 一致（证据冻结在 `evidence/render-determinism.txt`）：

```
run1 db4b6539c4cc973556de68091c24ee6ef02b4192d0a1a2e88c6a33503b31cc1a
run2 db4b6539c4cc973556de68091c24ee6ef02b4192d0a1a2e88c6a33503b31cc1a
deterministic: true
```

复现（两个独立浏览器会话的真对比——run1 整目录挪走后重渲染，逐文件对照；
~~旧复现命令渲染到 /tmp 却哈希 render/ 旧产物，不构成对比~~【后证不实，本轮
已修正，见 §3.6 修正 13】）：

```bash
mv render render.run1
node shoot.js "file://$PWD/index.html" render
python3 stitch.py
shasum -a 256 render.run1/full@2x.png render/full@2x.png   # 两行必须相同
for f in render.run1/slice-*.png; do cmp "$f" "render/${f##*/}"; done
rm -rf render.run1
```

## 3. 事实核查（红线逐条重跑）

以下命令在本轮验收中实际执行过；输出如实摘录。

### 3.1 论点数值（页面出现的每个浮点）

```bash
chromap --json contrast ratio '#663399' '#475c00'
# ratio 1.120197176950255 —— 四档阈值全 false（页面 hero/lede/面板 05/面板 07）
chromap --json inspect rebeccapurple
# relative_luminance 0.07492341159447033 · on black 2.4984682318894067 ·
# on white 8.405149896230322
chromap --json contrast ensure '#475c00' '#663399' --target aa
# #475c00 -> #adc777 · L 44.1177% -> 79.2402% · ratio 4.500000000000004 · lighter
chromap --json distance '#ff0000' '#00ff00'
# oklab 0.5198128892674524 · srgb 1.4142135623730951（= √2，闭式复算通过）
chromap --json distance '#000000' '#ffffff'
# oklab 0.9999999934735468 · srgb 1.7320508075688772（= √3，闭式复算通过）
```

论点比率——hero 的黑白对比率、面板 05 的互补与 ensure 比率、面板 07 的六个实测
点、面板 08 的四个距离——与 oklab/oklch 串、14 种 palette 色板、9 级标尺的 hex
均为 `data/*.json` 冻结值逐字渲染。显示截断共三类、逐一披露（§3.6 修正 2 /
10 / 11）：面板 06/07 的 linear 三元组 6 位小数；面板 10 逐级比率的 2 位小数
（9 个显示值均为冻结全精度值的正确舍入，全精度比率在 index.html 中出现 0 次，
存于 `data/design-tokens.json` 的 `contrast_on_white`）；sha256 摘要以截断前缀
显示（面板 10 的 PNG 双跑值前 26 位、页脚与面板 11 的 binary 值前 16 位，均以
… 明示截断，全值冻结于 `data/png-freeze.json` 与 `data/engine.json`）。

### 3.2 引文逐字对拍

```bash
sed -n '3,5p' $REPO/crates/chromap/src/lib.rs
# Color-wheel complements and readable foregrounds are separate concepts:
# use [`harmony`] for hue relationships and [`best_foreground`] or
# [`ensure_contrast`] for measured contrast.
```

面板 05 引文逐字命中（方括号链接标记按页面排版省略，标注「crate 文档第一段」）。

### 3.3 计数类声明

```bash
python3 - <<'PY'
import json, subprocess
cli = json.load(open("data/cli.json"))
print(len(cli["subcommands"]))          # 11（--help 逐字对拍）
print(json.load(open("data/parse-forms.json"))["named_count"])   # 149
print(len(json.load(open("data/palettes.json"))["kinds"]))       # 14
print(len(json.load(open("data/formats.json"))))                 # 7
PY
grep -c 'from_rgb8' $REPO/crates/chromap/src/named.rs   # 148
sed -n '153p' $REPO/crates/chromap/src/named.rs        # "transparent" => Color::TRANSPARENT,
```

「149 命名色」的口径：`data/parse-forms.json:named_count`（prep_data 对 named.rs
正则机器计数并断言后冻结——148 条 `from_rgb8` 表项 + transparent 特例
（named.rs:153）= **149 个 match 臂**，可用上面两条命令逐项复核；溯源修正见
§3.6 修正 9）。

「十个语法族」= #hex 3/4/6/8 位 + 0x + rgb()/rgba() + hsl()/hsla() + 命名色
（parse.rs:6-10 的 Supported forms 注释），页面用 11 发样本实证（多出发的一发是
transparent，属命名色族的特殊值，页面明写）。

### 3.4 退出码（exits.json 六探针，可重跑）

```bash
chromap convert '#abc' >/dev/null; echo $?                                   # 0
chromap convert '#zzzzzz' 2>/dev/null; echo $?                               # 2
chromap --json --plain convert '#abc' 2>/dev/null; echo $?                   # 2（互斥，main.rs:38）
chromap --json contrast ratio '#66339980' '#ffffff' 2>/dev/null; echo $?     # 2（alpha 需 canvas）
chromap --json contrast ensure '#333333' '#555555' --minimum 21 2>&1 | head -1
# error: contrast target 21:1 is unreachable; best available ratio is 7.455177810447525:1
```

### 3.5 源码行号（页脚来源表逐条抽查）

```bash
sed -n '15p' $REPO/crates/chromap/src/color.rs          # pub struct Color
sed -n '90,93p' $REPO/crates/chromap/src/contrast.rs    # 0.2126/0.7152/0.0722 加权
sed -n '225,233p' $REPO/crates/chromap/src/contrast.rs  # 阈值 3.0/4.5/4.5/7.0
sed -n '249,285p' $REPO/crates/chromap/src/contrast.rs  # search_lightness（48 步二分）
sed -n '25,32p' $REPO/crates/chromap/src/palette.rs     # harmony offsets
sed -n '165p' $REPO/crates/chromap/src/palette.rs       # GOLDEN_ANGLE 137.507_764_050_037_85
sed -n '387,393p' $REPO/crates/chromap/src/spaces.rs    # v/12.92 与 2.4 幂分段
sed -n '403,415p' $REPO/crates/chromap/src/spaces.rs    # linear_to_oklab 矩阵
sed -n '165,174p' $REPO/crates/chromap/src/spaces.rs    # Oklch::from_oklab 公式体：c=a.hypot(b)（167）、h=atan2(b,a)（171）
sed -n '13,15p' $REPO/crates/chromap-cli/src/visual.rs  # GRID_COLUMNS 8 / CELL 104×80
sed -n '772,778p' $REPO/crates/chromap-cli/src/main.rs  # CSS 变量循环体（"  --{prefix}-{}: {};\n" 与 index + 1 在 777）
sed -n '10,14p' $REPO/crates/chromap/examples/design_tokens.rs  # println! 块（"  --brand-{}: {};" 与 (index+1)*100 在 11-12）
```

### 3.6 修正记录（撤回不删除，修正前 → 修正后）

1. 【后证不实，已修正】初稿论点数值写成互补对比率 **1.0226902994676896:1**
   （凭早期手算记忆）→ 冻结实跑与本次验收复跑均为 **1.120197176950255:1**
   （`chromap --json contrast ratio '#663399' '#475c00'`；contract.md 同步修正
   【修正 15：该同步当时不完整，叙事顺序一节残留旧舍入值，本轮已补齐】）。
   「四档阈值全 false」的结论不受影响（两值都远低于 3.0）。
2. 【口径披露】linear sRGB 三元组在面板 06/07 以 **6 位小数** 展示（引擎 CLI 不
   输出 linear 通道；该三元组为公式复算、以引擎全精度亮度/比率为锚，复算 |Δ| < 1e-15），
   全精度在 `data/spaces-chain.json` —— ~~这是全页唯一非全精度展示，页脚与面板
   脚注均已披露~~【后证不实，已修正：全页显示截断共三类（本条 6 位小数、面板 10
   标尺比率 2 位小数、sha256 截断前缀），见修正 10/11；「页脚与面板脚注已披露」
   仍属实】。
3. 【口径披露】黑/白 ΔE_ok = **0.9999999934735468** ≠ 恰好 1.0：OKLab 公布矩阵
   系数为 10 位小数近似，正逆变换非严格互逆（自检 #12 放宽到 1e-7 并如实注明）。
4. 【口径披露】ensure 自报 ratio **4.500000000000004**，其输出 hex `#adc777` 重新
   `contrast ratio` 复测为 **4.487789210548097**（aa_normal false）：ensure 在连续
   f64 空间搜索，hex 是 8 位量化。两个数都如实上页面（面板 05 口径注 + 面板 07
   实测点），不合并口径——这正是「验收要测落盘 hex」的论据。同理，palette 输出的
   oklch 串描述其内部 f64 色，同一 `#475c00` 重新 convert 得 L 44.1177%
   （palette 侧 44.0272%）。
5. 【口径修正】面板 10 右卡初稿写「CLI palette 与 examples/design_tokens.rs 输出
   **完全一致**」→ 实测：**9 级 hex 全等（9/9）**，但变量命名不同——CLI
   `--css-prefix` 给 `--brand-1…9`（~~main.rs:772-776~~【修正 14：实为 772-778】），
   例程自带 `--brand-100…900`（~~design_tokens.rs:13-15~~【修正 14：实为 10-14】）。
   已改为精确表述并把两个口径都写上页面。
6. 【视觉修正】首轮门禁 17 条错误级 findings（明细见 §2.1）全部以几何修正清零：
   芯片估宽 0.55em → 0.65em（等宽实测 ≈0.633em）、右缘标签块翻转、p-spaces 汇聚
   连接线入口错开、p-luminance 贴轴网格线让位、CSS 行距下移。
7. 【计数修正】页面早期版本写「19 个冻结 JSON」→ 实数 **20**（`ls data/*.json | wc -l`）；
   现由 build 时 `glob` 动态计数，不再可能漂移。

**第二轮（2026-09-02）——独立对抗验证：14 条 findings 全部确认、0 条推翻；去重后
11 项同根合并修复，逐条如下。本轮全部指纹作废并走迁移链（§8）。**

8. 【计数口径修正】§3.3 曾写 `grep -c 'from_rgb8' named.rs` 输出 **150** 并注
   「149 命名色 + transparent 特例」→ 实跑输出 **148**。正确口径：148 条
   `from_rgb8` 表项 + transparent 特例（named.rs:153，`Color::TRANSPARENT`）=
   **149 个 match 臂**。`named_count=149` 本身无误，误的是 grep 期望值与
   「表项 150 条」的散文（同一缺陷在三处检查重复出现，同根一次修复）。
9. 【溯源修正】`named_count` 曾被 §3.3 描述为「prep_data 遍历引擎输出统计冻结」
   → 实为硬编码字面量（旧 JSON note 自注「静态统计自源码」）。已改为 prep_data
   运行时对 named.rs 正则机器计数并断言 == 149 后冻结；重跑前后对 20 个
   `data/*.json` 全量 sha256 对照，其余 19 个逐字节不变（仅本条 note 字段变更）。
10. 【总述口径修正】§3.1 曾总述「面板上全部比率……PNG sha256 均为冻结值逐字
    渲染」→ 实查：面板 10 的 9 级标尺比率显示 **2 位小数**（数字全对，均为冻结
    全精度值的正确舍入，面板脚注本已披露；全精度比率在 index.html 出现 0 次，
    存于 `data/design-tokens.json:contrast_on_white`）。已把总述限定为「论点比率
    （hero / 面板 05 / 07 / 08）全精度逐字」，截断清单改列三处（§3.1），并删
    §3.6-2 的「唯一」。
11. 【总述口径修正】§3.1 曾称「PNG sha256 逐字渲染」→ 实查：页面仅显示截断
    前缀（面板 10 的 PNG 双跑 sha256 前 **26** 位、页脚与面板 11 的 engine
    binary sha256 前 **16** 位，均以 … 明示截断）。全值冻结于
    `data/png-freeze.json` 与 `data/engine.json`。
12. 【归类修正】README 曾把 `color-palette-rs --json show tokyo-night` 列在
    「页面引用的引擎命令」小节 → 该命令的输出未上画面（仅作 dominant/average
    的原料冻结在 `data/tokyo-night.json`）。已移出并单独标注「证据管线命令
    （provenance，未上画面，见 VERIFICATION §7）」。
13. 【复现块修正】§2.3 / README 的渲染确定性复现命令曾「渲染到 /tmp、却哈希
    `render/` 旧产物」，不构成对比（空转）→ 已改为 run1 整目录挪走重渲染、
    双哈希 + 逐切片 `cmp` 的真两跑对比；`evidence/render-determinism.txt`
    按新命令真实重跑重写。
14. 【锚点修正】三处源码行号区间收窄到论断实际所在（页面面板 10、页脚来源表、
    §3.5 与本记录同步）：`design_tokens.rs:13-15 → 10-14`（命名行
    `"  --brand-{}: {};"` 与 `(index + 1) * 100` 在 11-12，原区间只有
    to_hex/`)`/`}`）；`main.rs:772-776 → 772-778`（`index + 1` 在 777，原区间
    把它排除在外）；`spaces.rs:137-166 → 165-174`（论断点名的
    `value.a.hypot(value.b)`(167) 与 `value.b.atan2(value.a)`(171) 原区间只盖到
    fn 签名行；且 impl 闭括号在 175，「137-174」亦不完整）。
15. 【残留值修正】contract.md「叙事顺序」一节残留旧舍入值「比率 **1.0227**」
    （已撤回值 1.0226902994676896 的舍入）→ 修正 1 自称的「contract.md 同步
    修正」当时不完整，本轮补齐为 **1.1202**。
16. 【防护加固】prep_data.py 曾边跑边落盘（README 声称的「落盘前自检全过才写盘」
    与实现不符）、且不校验执行上下文 → 已加启动预检（仓库 Cargo.toml / 例程 /
    named.rs / git 元数据，仓库外拒绝执行）并改为上下文预检 + 全部自检通过后
    统一落盘。负路径实测：仓库外副本运行 exit 1、`data/` 分毫未动。

## 4. 页面自包含

```bash
grep -c "<script" index.html                     # 0
grep -E "https?://" index.html | grep -cv xmlns  # 0（xmlns 为内联 SVG 命名空间常量，
                                                 #  浏览器不对其发起请求）
```

图片零外链：本图不含位图资源，全部图形为内联 SVG。

## 5. 重建一致性

```bash
cp index.html /tmp/index.before.html
python3 build.py >/dev/null
cmp /tmp/index.before.html index.html && echo byte-identical
# index.html sha256 6236ccbbe9cad70cb0e0d763930c8a6e765c73d549d605d0586d72591461369f
```

build.py 无时间戳、无绝对路径、无随机序；数据不变则产物不变。
交付位图指纹（`shasum -a 256`；2026-09-02 锚点修正重建后的值，旧值与迁移
原因见 §8）：

```
chromap-explainer@2x.png        db4b6539c4cc973556de68091c24ee6ef02b4192d0a1a2e88c6a33503b31cc1a
chromap-explainer-thumb.png     059759f27a17e2711755f1da420ea6fb3f13db6e2101893b7569b8a1a65987e8
```

## 6. 目检记录（全部基于断言过的 1:1 渲染）

- 逐面板全尺寸（`render/panels/p-*@2x.png`，11/11）：hero 七空间卡与三度量卡、
  CLI 管线箭头、parse 十一行探针表、spaces 双路换算图、complement 三卡对照与
  两块「实测可见性」演示（紫底绿字确实读不出 / 修复后读得出）、luminance 曲线
  三通道落点、contrast 数轴六点与右缘翻转标签、distance 四行全精度、kinds 14
  色板 + 7 格式、tokens 9 级标尺 + 双列 CSS、evidence 管线与自检选录——未见
  压字、越界、断词。
- 页眉/页脚裁片：KPI 五块（11 / 7 / 14 / 21/21 / 20）与来源表 22 行 +
  provenance 摘要与 `data/engine.json` 一致。
- 灰度版（`render/full-gray.png`，裁片存 `evidence/gray-proof.png`）：正文与
  表格去色后仍全部可读；曲线通道点带 R/G/B 字母标注，无仅靠颜色传达的信息。
- 缩略图（600×4444）：四故事层级、章节分隔线、表格密度节奏清晰。
- **第二轮目检（2026-09-02，锚点修正重建后）**：面板 10 全尺寸与右卡放大（新锚点
  `main.rs:772-778` / `design_tokens.rs:10-14` 两行文字完整、无压字越界）、页脚
  来源表裁片（OKLCH 行 `spaces.rs:165-174` 在列、表格无断行错位）、灰度裁片
  （9 级标尺 + CSS 块去色后仍可读）、缩略条（四章节节奏不变）——受影响区域
  全部复查通过；其余面板像素未变（本轮 diff 仅触及面板 10 与页脚一行）。

## 7. 边界与未声明事项（防过度解读）

- 页面呈现的是 `chromap 0.1.0`（commit f6a58fa）在**本页 17 组命令**上的实测面；
  mix / gradient / adjust / composite 只在 CLI 表中列出，未逐一展开面板。
- §3.6 修正 4 的量化差异只呈现实测两数，页面**不解释成因**（引擎内部工作副本
  未在 CLI 面暴露）。
- `data/tokyo-night.json` 为跨引擎输入样本（dominant/average 的原料），冻结在
  data 但未上画面——页面叙事不需要它，保留作 provenance。
- `render/` 与 `/tmp` 下产物为中间件，可由命令完整再生；`evidence/` 下三个文件
  是门禁/确定性/灰度的冻结快照。

## 8. 指纹迁移链（重建即作废旧指纹；旧 → 新 + 一行原因）

2026-09-02 修复轮（§3.6 修正 8-16）后全量重建，此前记录的指纹全部作废：

| 产物 | 旧 sha256 | 新 sha256 | 原因（一行） |
| --- | --- | --- | --- |
| index.html | 5ea23143100da901f8ddeb19b5a4944a4b2a16ebdc6c2832433d4da855fd1b98 | 6236ccbbe9cad70cb0e0d763930c8a6e765c73d549d605d0586d72591461369f | 页脚 spaces.rs 锚点 137-166→165-174；面板 10 两处锚点（修正 14） |
| chromap-explainer@2x.png | 2307d188c85ef775bcc053f9eed99494a84cbfee21aa623c868cde338cfb1946 | db4b6539c4cc973556de68091c24ee6ef02b4192d0a1a2e88c6a33503b31cc1a | index.html 变更后的重渲染（修正 14） |
| chromap-explainer-thumb.png | 96757beb53ef4884dac3c061f5f568a0efb70351176acfed5197714b0a89b7c7 | 059759f27a17e2711755f1da420ea6fb3f13db6e2101893b7569b8a1a65987e8 | 同上 |
| svg/p-tokens.svg | 4e59f0bd96d36cbda58c7077da8dabdd1a39138fdb38978389e38539428f400a | a55d9e7738556931b2611ea5fab82db9ed7bf4ae50cbd7881978461c5bdca49f | 面板 10 两处锚点文字（修正 14） |
| svg/ 其余 10 个 | — | **未变** | 本轮未触及；`shasum -a 256 svg/*.svg` 复核，除 p-tokens 外与首轮一致 |
| data/parse-forms.json | 8fce35cac8a16b2ca8203a8cbdcbc0af0e2c951ebb0293eec9fc44ddb5aeb64a | 98bda3152f18414bee606f46b369bc156cb8c8b6620d234fd1eca95b3cb07f34 | named_count_note 改机器计数口径（修正 9）；named_count=149 未变 |
| data/ 其余 19 个 | — | **未变** | prep_data 重跑逐字节不变（修正 9 的前后全量 sha256 对照） |

`evidence/` 三个冻结快照同轮按原命令重跑刷新：`gate.txt`（11×exit=0、
0 findings）、`render-determinism.txt`（新命令的真实两跑对比，修正 13）、
`gray-proof.png`（新位图同位置裁片：full-gray.png device y 13384..13704）。
