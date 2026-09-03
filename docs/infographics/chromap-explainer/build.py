#!/usr/bin/env python3
"""Assemble index.html + per-panel svg/*.svg from frozen data.

Zero JS, zero CDN, zero external requests. Deterministic: same data/*.json +
same generators → byte-identical output (no timestamps, no absolute paths).

Run from anywhere:  python3 build.py
"""
from __future__ import annotations

from pathlib import Path

import panels as P

HERE = Path(__file__).resolve().parent
SVG_DIR = HERE / "svg"


def fig(panel_id, svg_str, num, title, note):
    return (f'<figure id="{panel_id}">'
            f"<figcaption><span class=\"figno\">{num:02d}</span> {title}"
            f"<span class=\"fignote\">{note}</span></figcaption>"
            f"{svg_str}</figure>")


def chapter(no, zh, en, intro):
    return (f'<section class="chapter"><div class="ch-head">'
            f'<div class="ch-no">{no}</div>'
            f'<div><h2>{zh}</h2><p class="ch-en">{en}</p></div></div>'
            f'<p class="ch-intro">{intro}</p>')


CSS = """
:root { color-scheme: light; }
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  width: 1200px; margin: 0 auto; padding: 64px;
  background: #F7F4EE; color: #17212B;
  font-family: 'Source Han Sans SC','PingFang SC','Noto Sans SC',sans-serif;
  font-size: 14px; line-height: 1.65;
}
header { margin-bottom: 56px; }
.kicker {
  font-family: '0xProto Nerd Font','SF Mono',Menlo,monospace;
  font-size: 12px; letter-spacing: 2px; color: #356A79;
  text-transform: uppercase; margin-bottom: 14px;
}
h1 {
  font-family: 'Source Han Serif SC','Songti SC',serif;
  font-size: 40px; line-height: 1.28; font-weight: 600; letter-spacing: 0.5px;
  max-width: 1000px; margin-bottom: 18px;
}
.lede { font-size: 16px; color: #5D6873; max-width: 940px; margin-bottom: 36px; }
.lede b { color: #17212B; font-weight: 600; }
.kpi-row { display: flex; gap: 14px; }
.kpi {
  flex: 1; background: #FFFFFF; border: 1px solid #D9E1E3; border-radius: 8px;
  padding: 14px 18px 12px;
}
.kpi .n {
  font-family: '0xProto Nerd Font','SF Mono',Menlo,monospace;
  font-size: 30px; font-weight: 600; color: #28505C; line-height: 1.1;
}
.kpi .l { font-size: 12px; color: #5D6873; margin-top: 4px; }
section.chapter { margin-top: 64px; }
.ch-head { display: flex; align-items: baseline; gap: 18px;
  border-top: 2px solid #17212B; padding-top: 18px; margin-bottom: 14px; }
.ch-no {
  font-family: 'Source Han Serif SC','Songti SC',serif;
  font-size: 44px; font-weight: 700; color: #356A79; line-height: 1;
}
h2 { font-family: 'Source Han Serif SC','Songti SC',serif;
  font-size: 26px; font-weight: 600; letter-spacing: 0.5px; }
.ch-en { font-size: 12px; color: #5D6873; letter-spacing: 1px; }
.ch-intro { color: #5D6873; max-width: 980px; margin-bottom: 30px; }
figure { margin: 0 0 40px; max-width: 1072px; }
figcaption {
  font-size: 12.5px; color: #17212B; font-weight: 600;
  margin-bottom: 10px; display: flex; align-items: baseline; gap: 10px;
}
figcaption .figno {
  font-family: '0xProto Nerd Font','SF Mono',Menlo,monospace;
  color: #356A79;
}
figcaption .fignote { font-weight: 400; color: #5D6873; font-size: 11.5px; }
footer { margin-top: 72px; border-top: 2px solid #17212B; padding-top: 24px; }
footer h3 { font-size: 15px; margin-bottom: 12px; }
table.src { border-collapse: collapse; width: 100%; font-size: 12px; }
table.src th, table.src td { text-align: left; padding: 7px 12px;
  border-bottom: 1px solid #D9E1E3; vertical-align: top; }
table.src th { color: #5D6873; font-weight: 600; width: 46%; }
table.src td.mono, .mono {
  font-family: '0xProto Nerd Font','SF Mono',Menlo,monospace; font-size: 11.5px;
}
.prov { background: #FFFFFF; border: 1px solid #D9E1E3; border-radius: 8px;
  padding: 16px 20px; margin-top: 20px; font-size: 12px; color: #5D6873; }
.prov .mono { color: #28505C; }
.colophon { margin-top: 28px; font-size: 11px; color: #5D6873; }
"""


def esc(s: str) -> str:
    return (str(s).replace("&", "&amp;").replace("<", "&lt;")
            .replace(">", "&gt;").replace('"', "&quot;"))


def main() -> None:
    insp = P.load("inspect-rebeccapurple.json")
    cli = P.load("cli.json")
    forms = P.load("parse-forms.json")
    chain = P.load("spaces-chain.json")
    comp = P.load("complementary.json")
    ratio = P.load("ratio-complement.json")
    ensure = P.load("ensure-complement.json")
    ladder = P.load("contrast-ladder.json")
    dist = P.load("distance.json")
    pals = P.load("palettes.json")
    fmts = P.load("formats.json")
    tok = P.load("design-tokens.json")
    png = P.load("png-freeze.json")
    engine = P.load("engine.json")
    prov = P.load("provenance.json")
    selfchecks = P.load("selfchecks.json")

    svg_panels = [
        ("p-hero", P.p_hero(insp),
         "inspect：一个颜色，七个空间，三组度量",
         "全部字段 = chromap --json inspect rebeccapurple 冻结（浮点全精度）"),
        ("p-cli", P.p_cli(cli),
         "一条 argv 的完整路径：旗标 → 解析 → 内核 → 子命令 → 双出口",
         "子命令分组 = chromap --help 对拍（data/cli.json）· 退出码六探针（data/exits.json）"),
        ("p-parse", P.p_parse(forms),
         "parse.rs：十个语法族，十一发实测样本",
         "每行一次 chromap --json convert 实跑（data/parse-forms.json）· 命名色 named.rs"),
        ("p-spaces", P.p_spaces(chain, insp),
         "七个空间，两条换算路：gamma 域直接投影 vs linear→OKLab 链",
         "取值 = rebeccapurple 全程冻结；gamma 曲线锚 spaces.rs:387-400 公式（面板 07 解剖）"),
        ("p-complement", P.p_complement(comp, ratio, ensure),
         "论点：色环互补 ≠ 可读前景 —— 1.120197176950255:1，四档阈值全 false",
         "引文 lib.rs:3-5 · 三条命令实跑（complementary / contrast ratio / contrast ensure）"),
        ("p-luminance", P.p_luminance(chain, insp, selfchecks),
         "亮度从哪来：2.4 幂 gamma 解码 → linear 加权 → (L₁+0.05)/(L₂+0.05)",
         "曲线 = spaces.rs:387-400 公式采样 41 点（公式即数据源，非手画）· 复算 |Δ| < 1e-15"),
        ("p-contrast", P.p_contrast(ladder),
         "WCAG 四档阈值与六个实测点（含两条诚实拒绝路径）",
         "阈值 contrast.rs:225-233 · 每点一次引擎运行（data/contrast-ladder.json）"),
        ("p-distance", P.p_distance(dist),
         "distance 的双口径：ΔE_ok 与 ΔsRGB，同一对颜色两个读数",
         "四对实测（data/distance.json）· √2 / √3 闭式复算通过 · 通解不展开（见姊妹长图）"),
        ("p-kinds", P.p_kinds(pals, fmts),
         "palette 十四种 kind × 七种输出格式",
         "14 次 palette 实跑冻结（data/palettes.json）· 基色 = examples/design_tokens.rs:6"),
        ("p-tokens", P.p_tokens(tok, png),
         "操作闭环：品牌色 → 9 级标尺 → 逐级验收 → CSS 变量 + PNG 冻结",
         "palette + 9×ratio + cargo example 三方对拍 9 级 hex（data/design-tokens.json）· PNG 双跑 sha256 一致"),
        ("p-evidence", P.p_evidence(prov, engine, selfchecks,
                                    n_panels=11),
         "本图自己的来路：冻结 → 生成 → 门禁 → 断言渲染",
         "prep_data.py 21/21 复算自检 · provenance.json 记录每条完整 argv"),
    ]

    figures = {pid: fig(pid, s, i + 1, t, n)
               for i, (pid, s, t, n) in enumerate(svg_panels)}

    src_rows = [
        ("引擎论点引文「complements … separate concepts」",
         "crates/chromap/src/lib.rs:3-5"),
        ("Color 内核（归一化 gamma sRGB + alpha，全 f64）",
         "crates/chromap/src/color.rs:15"),
        ("十个语法族（#hex 3/4/6/8 · 0x · rgb/rgba · hsl/hsla · 命名色）",
         "crates/chromap/src/parse.rs:6-10 · data/parse-forms.json"),
        ("命名色 149 表项（含 transparent / rebeccapurple）",
         "crates/chromap/src/named.rs · data/parse-forms.json:named_count"),
        ("gamma 域投影 HSL / HSV / CMYK",
         "crates/chromap/src/spaces.rs:195 · :243 · :293"),
        ("linear→OKLab 前向矩阵（10 位小数系数 + cbrt）",
         "crates/chromap/src/spaces.rs:403-415"),
        ("OKLCH = oklab 的极坐标（c=√(a²+b²)，h=atan2）",
         "crates/chromap/src/spaces.rs:165-174"),
        ("色域外回程：32 步二分压 chroma，保 L 与 h",
         "crates/chromap/src/spaces.rs:359-380"),
        ("相对亮度 0.2126 R + 0.7152 G + 0.0722 B",
         "crates/chromap/src/contrast.rs:90-93"),
        ("对比率 (max(L)+0.05)/(min(L)+0.05)",
         "crates/chromap/src/contrast.rs:235-239"),
        ("WCAG 四档阈值 3.0 / 4.5 / 4.5 / 7.0",
         "crates/chromap/src/contrast.rs:225-233"),
        ("对比目标值域 1.0..=21.0（越界即拒绝）",
         "crates/chromap/src/contrast.rs:241-247"),
        ("ensure_contrast：48 步二分 OKLCH 亮度",
         "crates/chromap/src/contrast.rs:172-222 · :249-285"),
        ("harmony 色相偏移表（互补 [0,180] 等）",
         "crates/chromap/src/palette.rs:24-32"),
        ("黄金角 137.50776405003785°",
         "crates/chromap/src/palette.rs:162-165"),
        ("ΔE_ok / ΔsRGB 双距离同时输出",
         "crates/chromap/src/difference.rs:4-18"),
        ("PNG 色板网格：8 列，cell 104×80",
         "crates/chromap-cli/src/visual.rs:13-15 · data/png-freeze.json"),
        ("--json 与 --plain 互斥（exit 2）",
         "crates/chromap-cli/src/main.rs:38 · data/exits.json"),
        ("--css-prefix 字符集校验（ASCII 字母/数字/-/_）",
         "crates/chromap-cli/src/main.rs:759-766"),
        ("11 个功能子命令（--help 逐字对拍）",
         "data/cli.json · crates/chromap-cli/src/main.rs"),
        ("退出码 0（成功）/ 2（解析·参数·不可达）",
         "data/exits.json 六探针"),
        ("面板上全部数字",
         "data/*.json（provenance.json 记录每条完整 argv 与 sha256）"),
    ]
    src_html = "".join(f"<tr><th>{esc(h)}</th><td class=\"mono\">{esc(d)}</td></tr>"
                       for h, d in src_rows)

    n_kinds = len(pals["kinds"])
    n_cmds = len(prov["commands"])
    n_files = len(list((HERE / "data").glob("*.json")))

    html = f"""<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=1200">
<title>chromap 图解 · 互补色 ≠ 可读前景色</title>
<style>{CSS}</style>
</head>
<body>
<header>
<div class="kicker">chromap · 颜色引擎图解 · 2026-09</div>
<h1>chromap：色环管「配不配」，<br>实测对比度管「读不读得出」</h1>
<p class="lede">取色、配色、验收，工程师天天做 —— 但「色轮上的互补色」和
「WCAG 可读前景色」是<b>两套独立的度量</b>。chromap 把它们拆成两条命令：
<span class="mono">palette</span> 只在 OKLCH 里把色相转 180°（亮度不动），
<span class="mono">contrast</span> 用 gamma→linear→加权亮度实测比率；
实测下来，rebeccapurple 的互补对只有 <b>1.120197176950255:1</b>，
四档 WCAG 阈值全部不及格 —— 而 <span class="mono">ensure</span> 能只动亮度把它修复到阈值。
本页每个数字都来自引擎实跑冻结的 <span class="mono">data/*.json</span>（浮点全精度），
复现命令见 VERIFICATION.md。</p>
<div class="kpi-row">
<div class="kpi"><div class="n">{len(cli["subcommands"])}</div><div class="l">功能子命令（--help 对拍）</div></div>
<div class="kpi"><div class="n">7</div><div class="l">空间表示，一个 Color 内核</div></div>
<div class="kpi"><div class="n">{n_kinds}</div><div class="l">palette 种类（逐一实跑）</div></div>
<div class="kpi"><div class="n">{selfchecks["passed"]}/{selfchecks["total"]}</div><div class="l">落盘前复算自检</div></div>
<div class="kpi"><div class="n">{n_files}</div><div class="l">冻结证据文件（data/）</div></div>
</div>
</header>

{figures["p-hero"]}

{chapter("壹", "架构与数据流", "ARCHITECTURE &amp; DATA FLOW",
         "一条 argv 进来：全局旗标 → parse 十个语法族 → Color 内核（全 f64 的 gamma sRGB）→ 十一个子命令 → 字符串与像素双出口。无隐藏状态，每次运行都是纯函数；失败路径明说原因并退出 2，绝不猜。")}
{figures["p-cli"]}
{figures["p-parse"]}
</section>

{chapter("贰", "机制：互补色 ≠ 可读前景色", "COMPLEMENT IS NOT READABILITY",
         "本页的核心机制只讲这一件事：palette 的 harmony 只把 OKLCH 色相 +180°，亮度与 chroma 原样保留 —— 而对比度由亮度唯一决定。中间隔着一整条换算链：gamma sRGB → 2.4 幂解码 → linear → 加权亮度 → 比率公式。ensure 的修复也只动这条链上唯一相关的量：OKLCH 亮度。")}
{figures["p-spaces"]}
{figures["p-complement"]}
{figures["p-luminance"]}
</section>

{chapter("叁", "指标图鉴", "METRIC GALLERY",
         "三个读数工具的刻度与陷阱：WCAG 四档阈值（3.0 / 4.5 / 4.5 / 7.0）落在数轴上和六个实测点对齐；distance 同时输出 ΔE_ok 与 ΔsRGB 两个口径；palette 十四种 kind 里，标尺类与和谐类是两组生成器 —— -c 只对前者生效。")}
{figures["p-contrast"]}
{figures["p-distance"]}
{figures["p-kinds"]}
</section>

{chapter("肆", "操作管线", "OPERATIONS PIPELINE",
         "品牌色到设计 token 的完整闭环：inspect 度量 → 生成 9 级明度标尺 → 逐级对白底实测验收 → 冻结为 CSS 变量与 PNG（与库例程逐字节等价）。最后一格把镜头转向本页自己：冻结、生成、门禁、断言渲染的完整来路。")}
{figures["p-tokens"]}
{figures["p-evidence"]}
</section>

<footer>
<h3>来源表（每个声明 → 出处）</h3>
<table class="src">
<tr><th>声明</th><th>出处</th></tr>
{src_html}
</table>
<div class="prov">
<div>证据冻结：engine <span class="mono">{esc(engine["version"])}</span> ·
binary sha256 <span class="mono">{engine["binary_sha256"][:16]}…</span> ·
repo commit <span class="mono">{engine["repo_commit"][:12]}</span>（完整 argv
与文件清单见 <span class="mono">data/provenance.json</span>）</div>
<div style="margin-top:6px">实跑 <span class="mono">{n_cmds}</span> 组命令冻结为
<span class="mono">{n_files}</span> 个 JSON；引擎输出的浮点一律全精度、不截断
（面板上仅 linear 三元组以 6 位小数展示并已披露，全精度在
<span class="mono">data/spaces-chain.json</span>）。</div>
<div style="margin-top:6px">重建：<span class="mono">python3 build.py</span> 生成
index.html 与 svg/*.svg；渲染管线与全部验收命令见 README.md / VERIFICATION.md。</div>
</div>
<div class="colophon">零 JavaScript · 零 CDN · 无外部请求 —— 图即文档，数字即证据。</div>
</footer>
</body>
</html>
"""

    (HERE / "index.html").write_text(html, encoding="utf-8")

    SVG_DIR.mkdir(exist_ok=True)
    for pid, s, _, _ in svg_panels:
        (SVG_DIR / f"{pid}.svg").write_text(s + "\n", encoding="utf-8")

    print(f"index.html: {len(html)} bytes; svg/: {len(svg_panels)} panels")
    for pid, s, _, _ in svg_panels:
        print(f"  {pid}: {len(s)} bytes")


if __name__ == "__main__":
    main()
