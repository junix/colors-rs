# chromap 图解（可审计技术长图）

单页长图，讲清 chromap 颜色引擎的一个核心机制：**色环互补色 ≠ 可读前景色**。
每个数字都来自引擎实跑冻结的 `data/*.json`（浮点全精度），每个声明都能在
VERIFICATION.md 里找到可重跑的对拍命令。

## 交付物

| 文件 | 说明 |
| --- | --- |
| `index.html` | 长图页面（1200 CSS px 宽，8943 CSS px 高；零 JS / 零 CDN / 无外部请求；不印代码坐标，声明以锚点编号 A-xx 引用 VERIFICATION §9.2） |
| `chromap-explainer@2x.png` | 最终位图 2400×17886（= 1200×8943 × dpr 2，stitch 断言通过） |
| `chromap-explainer-thumb.png` | 1/4 缩略图（层级节奏检查） |
| `svg/` | 11 个面板的独立 SVG（svg-linter 门禁对象） |
| `data/` | 20 个冻结证据 JSON + `provenance.json`（每条完整 argv） |
| `evidence/` | 门禁输出、渲染确定性证明、灰度可读性裁片 |
| `contract.md` | 视觉契约（受众 / 论点 / 差异化约束 / 面板清单） |
| `VERIFICATION.md` | 验收记录：门禁、事实核查表、修正记录 |

## 生成管线（4 步，全部可重跑）

```bash
cd docs/infographics/chromap-explainer

# 0) 引擎身份（默认取 $HOME/sync/macos-arm64-bin/chromap，可用 CHROMAP_ENGINE 覆盖）
chromap --version                                   # chromap 0.1.0
shasum -a 256 "$(command -v chromap)"               # 见 data/engine.json

# 1) 冻结证据：实跑 17 组命令 → data/*.json，落盘前 21 条复算自检全过才写盘
#    前提：在 colors-rs 仓库内的本目录执行（依赖 cargo run --example design_tokens、
#    git 元数据与 named.rs 源码计数）；脚本启动即校验仓库上下文，预检或任一自检
#    不过都不落盘。仓库外快照（无 Cargo.toml / git）直接使用已交付 data/，勿重冻结。
python3 prep_data.py                                # 可选：data/ 已随仓库交付

# 2) 生成页面与 SVG（确定性：无时间戳 / 无绝对路径 / 无随机序）
python3 build.py                                    # -> index.html + svg/*.svg
#    build 末尾跑 code_detail_gate：12 产物 × 6 式清扫（file:line / 引擎文件名 /
#    行区间 / 第N行 / rust 关键字 / 标识符调用串）必须零命中——代码细节下页
#    政策见 VERIFICATION §9；引擎源码名清单冻结于 build.py 内。

# 3) 结构门禁：一次一个文件，错误级 findings 必须为 0
for f in svg/*.svg; do svg-linter --plain check "$f"; done

# 4) 断言渲染：从 y=0 起全宽定高切片，只按切片顺序拼接，断言位图高 == CSS 高 × dpr
node shoot.js "file://$PWD/index.html" render
python3 stitch.py                                   # -> render/full@2x.png + 交付位图
```

## 重建一致性

```bash
cp index.html /tmp/index.before.html
python3 build.py >/dev/null
cmp /tmp/index.before.html index.html && echo byte-identical
```

两次**独立浏览器会话**渲染 sha256 一致（真对比：run1 整目录挪走后重渲染、
双哈希 + 逐切片 `cmp`；证据：`evidence/render-determinism.txt`）：

```bash
mv render render.run1
node shoot.js "file://$PWD/index.html" render
python3 stitch.py
shasum -a 256 render.run1/full@2x.png render/full@2x.png   # 两行必须相同
rm -rf render.run1
```

## 页面引用的引擎命令（README 逐条实跑过）

```bash
chromap --json inspect rebeccapurple                    # 面板 01：七空间 + 三度量
chromap --help                                          # 11 个功能子命令（data/cli.json 对拍）
chromap --json convert <样本>                           # 面板 03：十个语法族逐行探针
chromap --json palette rebeccapurple -k complementary   # 面板 05：互补色板（同亮度）
chromap --json contrast ratio '#663399' '#475c00'       # 面板 05：1.120197176950255:1
chromap --json contrast ensure '#475c00' '#663399' --target aa
chromap --json palette '#4f7cff' -k neighbors -c 9 \
  --lightness-span 0.64 --css-prefix brand              # 面板 10：9 级标尺 + CSS 变量
chromap --json contrast ratio '#<标尺级>' '#ffffff'      # 面板 10：逐级验收 ×9
cargo run --quiet --package chromap --example design_tokens
#   ↑ 例程输出 --brand-100…900；CLI --css-prefix 输出 --brand-1…9（命名口径不同，
#     9 级 hex 两边全等，面板 10 如实呈现两个口径）
```

## 证据管线命令（provenance，未上画面）

```bash
color-palette-rs --json show tokyo-night   # 跨引擎样本：dominant/average 的输入原料，
                                           # 冻结在 data/tokyo-night.json，不上画面
                                           # （见 VERIFICATION §7）
```

依赖：`python3`（+ Pillow，仅 stitch 用）、`node`（shoot.js 走 CDP，用 playwright
缓存里的 chrome-headless-shell）、`svg-linter`。页面本身无任何运行时依赖。

## 视觉体系

沿用可审计长图的角色调色板（浅色纸面 + 蓝系主色，见 `svgkit.py`）；面板里的
**样本色**一律是引擎冻结输出（#663399 / #475c00 / #adc777 / 14 种 palette 实跑色……），
基色 #4f7cff 出自 `crates/chromap/examples/design_tokens.rs:6`，跨引擎样本取自
`color-palette-rs --json show tokyo-night`——页面上没有凭记忆手填的颜色。
