# 视觉契约 — chromap-explainer

## 受众与论点

- **受众**：写前端 / 设计系统 / CLI 工具的工程师——会取色、会配对，但不一定分得清
  「色环关系」和「WCAG 可读性」是两套度量。
- **一句话论点**：色轮上的互补色回答「配不配」，实测对比度回答「读不读得出」；
  chromap 把这两件事拆成两条命令（`palette` vs `contrast`），并提供 `ensure` 自动修复。
  论点的定量证据：rebeccapurple 的 OKLCH 互补色与本体**同亮度**（palette 输出侧 L 同为
  44.0272%），实测对比度 **1.120197176950255:1**，四档 WCAG 阈值全部不及格。
  （初稿曾凭记忆写 1.0226902994676896:1，冻结实跑推翻，修正记录见 VERIFICATION.md。）
- **读者问题**：
  1. 引擎怎么串？（CLI 11 子命令 → parse.rs 十个语法族 → Color 内核 → 七空间 → format/visual 双出口）
  2. 为什么互补色不可读？（OKLCH 只动色相 +180°，亮度不动；亮度经 gamma→linear 链决定 WCAG 比率）
  3. 指标怎么读？（WCAG 3.0/4.5/4.5/7.0 四档、ΔE_ok vs ΔRGB、14 种 palette kind、7 种输出格式）
  4. 怎么用？（inspect 度量 → palette/adjust 生成 → contrast ensure 修复 → --json/--css-prefix/PNG 冻结为 token）

## 差异化硬约束（与姊妹项目 color-palette-rs 长图的关系）

- 本页**不**讲 OKLCH 色彩科学通解（感知均匀性原理、RGB 欧氏距离反例通解）——
  姊妹长图已覆盖，重复即废稿。
- 本页的机制故事锚定在**本引擎的 API 切分**上：`lib.rs:3-4` 明文写着
  「Color-wheel complements and readable foregrounds are separate concepts」——
  互补色板（palette.rs `harmony`，只加色相偏移）与可读前景
  （contrast.rs `ensure_contrast`，二分搜索 OKLCH 亮度）是**两条独立代码路径**，
  中间只靠 gamma→linear→相对亮度 这条换算链桥接。
- ΔRGB 反例只作为「distance 子命令同时输出两个数」的读数口径出现，不做通解展开。

## 证据面（一切数字的来源）

- 引擎真实运行：`chromap`（安装版 `chromap 0.1.0`，binary sha256 见 `data/engine.json`）
  的 inspect / convert / palette / contrast / distance / dominant / average / PNG 输出，
  全部冻结为 `data/*.json`；浮点全精度、不截断。
- 重算自检（Python 复算 vs 引擎输出，落盘前全部通过，结果冻结在 `data/selfchecks.json`）：
  相对亮度、比率公式、√2 / √3 / ΔE_ok=1 三组闭式值、互补色 +180° 色相守恒、
  OKLab 链路、PNG 双跑 sha256 一致、CLI palette 与 examples/design_tokens.rs 等价。
- 源码锚点：仓库 commit `f6a58fa`，逐条 `file:line` 见页面页脚来源表。
- 通解性内容（如 gamma 曲线形状、WCAG 阈值的来历）标注「通解」徽章；
  曲线几何由 `spaces.rs:387-400` 的公式直接采样生成（公式即数据源），非手画。

## 叙事顺序（四故事 × 11 面板）

1. **壹 架构与数据流**
   - `p-hero`：inspect 一个颜色——七空间全表示 + 三组度量全精度（实跑冻结）
   - `p-cli`：CLI 表面 → parse → Color 内核 → 11 子命令 → format/visual 双出口；退出码 0/2
   - `p-parse`：parse.rs 十个语法族、十一发实测样本（命名色 149 个含 transparent）
2. **贰 机制通解（互补色 ≠ 可读前景色）**
   - `p-spaces`：七空间转换图——gamma 域三空间 vs linear→OKLab 链，rebeccapurple 全程取值
   - `p-complement`：论点面板——互补色板同亮度 → 比率 1.1202 全不及格 → ensure 修复封套
   - `p-luminance`：换算链解剖——2.4 幂 gamma、0.2126/0.7152/0.0722 加权、(L₁+0.05)/(L₂+0.05)
3. **叁 指标图鉴**
   - `p-contrast`：WCAG 四档阈值 + 五个实测点落在数轴上 + unreachable 错误路径
   - `p-distance`：ΔE_ok vs ΔRGB 同对不同读数；黑↔白两度量分别为 1.0 与 √3（闭式可复算）
   - `p-kinds`：14 种 palette kind 实测色板（harmony 类长度固定，-c 不生效）+ 7 种输出格式
4. **肆 操作管线**
   - `p-tokens`：品牌色 → 9 级明度标尺 → 逐级对比度实测 → CSS 变量 + PNG 冻结（design_tokens 工作流）
   - `p-evidence`：本图自己的来路——prep_data.py 冻结管线与自检清单

## 媒介与尺寸

- 单页 HTML 长图，1200 CSS px 宽，@2x 渲染 2400 px；零 JS、零 CDN、无外部请求。
- 配色：沿用 lut-ops / pm-file-parser 可审计长图的角色调色板（浅色纸面 + 蓝系主色）：
  PAPER `#F7F4EE` / INK `#17212B` / MUTED `#5D6873` / RULE `#D9E1E3` /
  FLOW `#356A79`（主蓝）· FLOW_DK `#28505C` · FLOW_LT `#7FA3AD` · FLOW_XLT `#B9CDD3` /
  TEAL `#2A9D8F` · TINT `#DDF2EC` · WARN `#E76F51` · WARN_LT `#F4C7B8` · OUTCOME `#E9C46A`。
  面板内出现的**样本色块**（如 #663399、#475c00）一律是引擎冻结输出，不是编辑装饰色。
- 字体：Source Han Serif SC（标题）/ Source Han Sans SC（正文）/ 0xProto Nerd Font · SF Mono · Menlo（代码）。
- 中文正文；代码符号、子命令、字段名一律英文原样；浮点全精度。

## 面积预算

11 个 SVG 面板 + HTML 头部（kicker/h1/lede/KPI）+ 页脚（来源表 + provenance 摘要）。
目标页面高度 ≈ 8000–10000 CSS px（与 pm-file-parser 长图同量级）。
