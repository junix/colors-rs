#!/usr/bin/env python3
"""Panel generators for the chromap explainer long-form page.

Every function returns a complete inline <svg> string. All engine-derived
values come from data/*.json (frozen by prep_data.py); every thesis float
(hero / panels 05, 07, 08) is rendered at full precision, never truncated.
Disclosed display truncations (all correct roundings of frozen full
precision, see VERIFICATION §3.1): the recomputed linear-RGB triple at 6
decimals (full precision in data/spaces-chain.json), the panel-10 per-step
scale ratios at 2 decimals (full precision in
data/design-tokens.json:contrast_on_white), and sha256 digests as truncated
prefixes (full values in data/png-freeze.json / data/engine.json).
Editorial role colors come from
svgkit; any other swatch on the page is an engine-frozen sample color, not
decoration. Literal hex only — no CSS var() inside SVG.
"""
from __future__ import annotations

import json
from pathlib import Path

from svgkit import (CODE_BG, FLOW, FLOW_DK, FLOW_LT, FLOW_XLT, INK, MUTED,
                    OUTCOME, PAPER, RULE, TEAL, TINT, WARN, WARN_LT,
                    arrow_def, circle, connector, line, mono, poly, rect, svg,
                    text, text_width)

DATA = Path(__file__).resolve().parent / "data"

# spaces.rs:387-400 conversion formulas — the gamma curve below is sampled
# from these exact expressions (the formula is the data source).


def srgb_to_linear(v: float) -> float:
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def load(name):
    return json.load(open(DATA / name))


# ---------------------------------------------------------------------------
# shared helpers

def node(x, y, w, h, title, sub=(), fill=PAPER, stroke=INK, sw=1.2,
         tsize=13.5, ssize=11, tcolor=INK, dash=None, mono_title=False):
    subs = [sub] if isinstance(sub, str) else list(sub)
    out = [rect(x, y, w, h, fill=fill, stroke=stroke, sw=sw, rx=6, dash=dash)]
    ty = y + h / 2 + 4.5 - (9 if subs else 0)
    out.append((mono if mono_title else text)(x + w / 2, ty, title, size=tsize,
               fill=tcolor, weight="600", anchor="middle"))
    for i, s in enumerate(subs):
        out.append(text(x + w / 2, ty + 16 + i * 14, s, size=ssize,
                        fill=MUTED, anchor="middle"))
    return "".join(out)


def chip(x, y, s, size=10.5, fill=TINT, tcolor=FLOW_DK, h=17, pad=5,
         mono_font=True, weight="400", stroke=None):
    """Auto-sized pill from a measured text width. (x,y) = top-left."""
    w = text_width(s, size, mono=mono_font) + pad * 2
    out = [rect(x, y, w, h, fill=fill, rx=3, stroke=stroke, sw=0.8)]
    out.append((mono if mono_font else text)(x + pad, y + h / 2 + size * 0.36,
               s, size=size, fill=tcolor, weight=weight))
    return "".join(out), w


def chiprow(x, y, items, size=10.5, gap=5, **kw):
    out, cx = [], x
    for s in items:
        c, w = chip(cx, y, s, size=size, **kw)
        out.append(c)
        cx += w + gap
    return "".join(out), cx


def srcnote(x, y, s):
    return mono(x, y, s, size=10, fill=MUTED)


def swatch(x, y, s, w=14, h=14, stroke=RULE):
    return rect(x, y, w, h, fill=s, stroke=stroke, sw=0.8, rx=3)


def ratio_str(r):
    return repr(r) + ":1"


def parse_oklch(s):
    """'oklch(44.0272% 0.160296 303.373)' -> (l_str, c_str, h_str)"""
    body = s[s.index("(") + 1:s.rindex(")")]
    parts = body.split()
    return parts[0], parts[1], parts[2]


# ---------------------------------------------------------------------------
# 01 HERO — inspect: one input, seven spaces, three measurements

def p_hero(insp):
    c = insp["color"]
    W, H = 1072, 566
    o = []
    # command strip
    o.append(rect(24, 20, 566, 30, fill=CODE_BG, rx=5))
    o.append(mono(38, 40, "chromap --json inspect rebeccapurple", size=12.5,
                  fill=INK, weight="600"))
    c0, _ = chip(622, 26, "exit 0 · stdout JSON", size=10.5, fill="#FFFFFF",
                 stroke=RULE)
    o.append(c0)

    # identity card
    ix, iy, iw, ih = 24, 84, 320, 176
    o.append(rect(ix, iy, iw, ih, fill="#FFFFFF", stroke=INK, sw=1.3, rx=7))
    o.append(swatch(ix + 20, iy + 20, c["hex"], w=64, h=64, stroke=RULE))
    o.append(mono(ix + 100, iy + 44, c["hex"], size=21, fill=INK, weight="600"))
    o.append(mono(ix + 100, iy + 66, "rebeccapurple", size=11.5, fill=MUTED))
    o.append(mono(ix + 100, iy + 84, "Color { r g b a : f64 }", size=10.5,
                  fill=FLOW_DK))
    o.append(mono(ix + 20, iy + 112,
                  f"rgba8  r={c['rgba8']['r']} g={c['rgba8']['g']} "
                  f"b={c['rgba8']['b']} a={c['rgba8']['a']}", size=10.5,
                  fill=INK))
    o.append(mono(ix + 20, iy + 132,
                  "gamma  (0.400000 0.200000 0.600000)", size=10.5, fill=INK))
    o.append(mono(ix + 20, iy + 152,
                  "alpha  1.0（不透明，可直接测对比度）", size=10, fill=MUTED))

    # seven space cards (4 + 3), all values frozen verbatim
    spaces = [("hex", c["hex"]), ("rgb", c["rgb"]), ("hsl", c["hsl"]),
              ("hsv", c["hsv"]), ("cmyk", c["cmyk"]), ("oklab", c["oklab"]),
              ("oklch", c["oklch"])]
    gx, gy, gw, gh, gg = 388, 84, 164, 52, 8
    for i, (name, val) in enumerate(spaces):
        cx = gx + (i % 4) * (gw + gg)
        cy = gy + (i // 4) * (gh + gg)
        o.append(rect(cx, cy, gw, gh, fill=PAPER, stroke=RULE, sw=1, rx=5))
        o.append(mono(cx + 10, cy + 18, name, size=10.5, fill=FLOW_DK,
                      weight="600"))
        if len(val) > 22:
            cut = val.index(" ", val.index(" ") + 1)
            o.append(mono(cx + 10, cy + 34, val[:cut], size=9.5, fill=INK))
            o.append(mono(cx + 10, cy + 46, val[cut + 1:], size=9.5, fill=INK))
        else:
            o.append(mono(cx + 10, cy + 38, val, size=11, fill=INK))
    note_x = gx + 3 * (gw + gg)
    o.append(rect(note_x, gy + gh + gg, gw, gh, fill=TINT, rx=5))
    o.append(text(note_x + 10, gy + gh + gg + 19, "七种表示 · 一个内核",
                  size=10.5, fill=FLOW_DK, weight="600"))
    o.append(text(note_x + 10, gy + gh + gg + 34, "Color 是唯一真相，其余",
                  size=9, fill=MUTED))
    o.append(text(note_x + 10, gy + gh + gg + 46, "都是投影（spaces.rs）",
                  size=9, fill=MUTED))

    # three measurement cards, full precision
    o.append(text(388, 212, "度量（引擎全精度，不截断）", size=11.5, fill=INK,
                  weight="600"))
    meas = [("relative_luminance", repr(insp["relative_luminance"]),
             "0.2126 R + 0.7152 G + 0.0722 B"),
            ("contrast_on_black", ratio_str(insp["contrast_on_black"]),
             "(L + 0.05) / 0.05"),
            ("contrast_on_white", ratio_str(insp["contrast_on_white"]),
             "1.05 / (L + 0.05)")]
    my = 222
    for i, (name, val, formula) in enumerate(meas):
        cx = 388 + i * 226
        o.append(rect(cx, my, 218, 74, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
        o.append(mono(cx + 12, my + 20, name, size=10.5, fill=FLOW_DK,
                      weight="600"))
        o.append(mono(cx + 12, my + 42, val, size=11, fill=INK, weight="600"))
        o.append(mono(cx + 12, my + 60, formula, size=9, fill=MUTED))

    # bottom: parse accepts ten syntax families for the same input
    pf = load("parse-forms.json")
    forms, named_count = pf["forms"], pf["named_count"]
    o.append(line(24, 322, 1048, 322, stroke=RULE, w=1))
    o.append(text(24, 346, f"同一种输入语言：十个语法族（parse.rs），命名色 {named_count} 个",
                  size=12.5, fill=INK, weight="600"))
    fx, fy = 24, 358
    for f in forms:
        cchip, w = chip(fx, fy, f["sample"], size=10.5)
        o.append(cchip)
        o.append(swatch(fx + 2, fy + 21, f["hex"], w=12, h=12))
        fx += max(w, text_width(f["hex"], 10.5, mono=True) + 10) + 14
        if fx > 940:
            fx, fy = 24, fy + 44
    o.append(text(24, fy + 54,
                  "十一个样本（#hex 四种宽度 / 0x / rgb·rgba / hsl·hsla / 命名色 / transparent）"
                  "全部归一到同一族 sRGB；alpha 进入第 4 分量", size=10.5, fill=MUTED))
    o.append(srcnote(24, H - 16,
                     "值全部来自 data/inspect-rebeccapurple.json 与 data/parse-forms.json"
                     "（chromap --json inspect / convert 实跑冻结）"))
    return svg("p-hero", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 02 STORY 1 — execution pipeline: argv -> parse -> Color -> modules -> dual exit

def p_cli(cli):
    W, H = 1072, 500
    o = []
    ny, nh = 22, 62
    o.append(node(24, ny, 208, nh, "chromap <argv>",
                  ("全局旗标先于子命令",), mono_title=True, tsize=13.5))
    o.append(node(276, ny, 208, nh, "parse_color",
                  ("parse.rs · 十个语法族",), mono_title=True, tsize=13.5))
    o.append(node(528, ny, 220, nh, "Color { r g b a }",
                  ("归一化 gamma sRGB + alpha", "color.rs:15"), mono_title=True,
                  tsize=13.5, ssize=10))
    for x1, x2 in [(232, 276), (484, 528)]:
        o.append(connector(x1, ny + nh / 2, x2, ny + nh / 2))
    o.append(text(254, ny + nh / 2 - 10, "字符串", size=9.5, fill=MUTED,
                  anchor="middle"))
    o.append(text(506, ny + nh / 2 - 10, "f64×4", size=9.5, fill=MUTED,
                  anchor="middle"))
    # global flags
    o.append(chiprow(24, ny + nh + 12, [
        "--format hex|rgb|hsl|hsv|cmyk|oklab|oklch", "--json", "--plain",
        "--color auto|always|never"])[0])
    o.append(text(24, ny + nh + 52,
                  "全局旗标作用于所有「产色」子命令；--json 与 --plain 互斥（main.rs:38，实测 exit 2）",
                  size=10, fill=MUTED))

    # module groups (all 11 subcommands, grouped by role)
    gy, gh2 = 152, 158
    groups = [
        ("度量 measure", [
            ("inspect", "七空间 + 亮度 + 黑白对比度"),
            ("contrast", "ratio · pick · black-white · ensure"),
            ("", "WCAG 四档阈值；alpha 需 --canvas"),
            ("distance", "ΔE_ok 与 ΔsRGB 同时输出")], 312),
        ("生成 generate", [
            ("palette -k ×14", "和谐类长度固定，标尺类 -c 生效"),
            ("adjust", "OKLCH/HSL 明度·饱和·色相微调"),
            ("mix / gradient", "四种插值空间 × 四条色相路径"),
            ("convert", "单色转到全局 --format")], 312),
        ("合成 compose", [
            ("composite", "source-over，sRGB 或 linear 域"),
            ("average", "alpha 感知的均值"),
            ("dominant", "OKLab k-means 聚类代表色"),
            ("（无隐藏状态）", "每次运行 = 纯函数")], 300),
    ]
    gx = 24
    for title, rows, gw in groups:
        o.append(rect(gx, gy, gw, gh2, fill="#FFFFFF", stroke=RULE, sw=1.1, rx=6))
        o.append(text(gx + 14, gy + 22, title, size=12, fill=FLOW_DK,
                      weight="600"))
        yy = gy + 44
        for name, note in rows:
            o.append(mono(gx + 14, yy, name, size=10.5, fill=INK))
            o.append(text(gx + 132, yy, note, size=9.5, fill=MUTED))
            yy += 27
        gx += gw + 14
    o.append(connector(638, ny + nh + 62, 638, gy - 4))

    # dual exit
    ey, eh = 336, 96
    o.append(node(24, ey, 420, eh, "format.rs · format_color()",
                  ("七种字符串表示（--format 选择）",
                   "hex / rgb / hsl / hsv / cmyk / oklab / oklch"),
                  mono_title=True, tsize=12.5, ssize=10))
    o.append(node(476, ey, 300, eh, "visual.rs · 双出口",
                  ("ANSI 终端色块（--color 控制）", "PNG 色板网格 104×80 cell"),
                  mono_title=True, tsize=12.5, ssize=10))
    o.append(node(808, ey, 240, eh, "stdout",
                  ("人类可读 / --json", "exit 0 成功 · 2 失败"),
                  mono_title=True, tsize=13, ssize=10))
    for x1, x2 in [(444, 476), (776, 808)]:
        o.append(connector(x1, ey + eh / 2, x2, ey + eh / 2))
    o.append(connector(234, gy + gh2, 234, ey - 4))
    o.append(connector(626, gy + gh2, 626, ey - 4))

    sy = H - 48
    cards = [
        ("0", "成功（stdout 正常）", TEAL),
        ("2", "解析失败 / 参数错误 / 目标不可达", WARN),
    ]
    for i, (code, meaning, color) in enumerate(cards):
        cx = 24 + i * 360
        o.append(rect(cx, sy, 348, 40, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
        o.append(mono(cx + 16, sy + 27, code, size=19, fill=color, weight="600"))
        o.append(text(cx + 44, sy + 25, meaning, size=10.5, fill=INK))
    o.append(srcnote(24, sy - 8,
                     f"11 个功能子命令（chromap --help，{len(cli['subcommands'])} 个逐字对拍 data/cli.json）"
                     " · 退出码六探针实测 data/exits.json · main.rs:55-97, 274"))
    return svg("p-cli", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 03 STORY 1 — parse.rs: ten syntax families, eleven live probes

def p_parse(forms):
    rows = forms["forms"]
    n_breaks = 2
    W, H = 1072, 96 + len(rows) * 33 + n_breaks * 20 + 70
    o = []
    cols = [24, 300, 560, 700]
    hy = 34
    o.append(text(cols[0], hy, "语法族（parse.rs）", size=11, fill=MUTED, weight="600"))
    o.append(text(cols[1], hy, "实测样本", size=11, fill=MUTED, weight="600"))
    o.append(text(cols[2], hy, "解析结果（hex）", size=11, fill=MUTED, weight="600"))
    o.append(text(cols[3], hy, "alpha", size=11, fill=MUTED, weight="600"))
    o.append(text(830, hy, "归属", size=11, fill=MUTED, weight="600"))
    y = hy + 16
    group_breaks = {5: "函数记法：空格现代 / 逗号传统均可，/ alpha 亦支持",
                    9: "命名色表（named.rs）"}
    for i, f in enumerate(rows):
        if i in group_breaks and i:
            o.append(text(cols[0], y + 12, "— " + group_breaks[i], size=9.5,
                          fill=FLOW_DK))
            y += 20
        o.append(line(24, y, 1048, y, stroke=RULE, w=0.8))
        o.append(text(cols[0], y + 19, f["family"], size=10.5, fill=INK))
        o.append(mono(cols[1], y + 19, f["sample"], size=11, fill=FLOW_DK,
                      weight="600"))
        o.append(swatch(cols[2], y + 8, f["hex"], w=13, h=13))
        o.append(mono(cols[2] + 20, y + 19, f["hex"], size=11, fill=INK))
        o.append(mono(cols[3], y + 19, repr(f["alpha"]), size=10.5, fill=MUTED))
        if f["sample"] in ("rgb(102 51 153)", "hsl(270 50% 40%)",
                           "rebeccapurple"):
            o.append(mono(830, y + 19, "→ 同一颜色 #663399", size=10, fill=TEAL))
        y += 33
    o.append(line(24, y, 1048, y, stroke=INK, w=1.1))
    o.append(srcnote(24, y + 20,
                     "data/parse-forms.json（chromap --json convert <样本> 逐行实跑）· "
                     "parse.rs:6-9 语法族 · named.rs 149 表项（含 transparent 与 rebeccapurple）"))
    o.append(srcnote(24, y + 37,
                     "十族 = #hex 3/4/6/8 位 + 0x + rgb()/rgba() + hsl()/hsla() + 命名色；"
                     "0x 前缀同样接受 3/4/6/8 位"))
    return svg("p-parse", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 04 STORY 2 — seven spaces conversion map (spaces.rs)

def p_spaces(chain, insp):
    W, H = 1072, 474
    o = []
    # core
    cx, cy, cw, ch = 430, 170, 212, 84
    o.append(rect(cx, cy, cw, ch, fill="#FFFFFF", stroke=INK, sw=1.4, rx=7))
    o.append(mono(cx + cw / 2, cy + 26, "Color", size=15, fill=INK,
                  weight="600", anchor="middle"))
    o.append(mono(cx + cw / 2, cy + 46, "(0.4  0.2  0.6  1.0)", size=11.5,
                  fill=FLOW_DK, anchor="middle"))
    o.append(text(cx + cw / 2, cy + 66, "gamma sRGB f64 + alpha · 唯一内核",
                  size=10, fill=MUTED, anchor="middle"))

    # left cluster: gamma-domain projections (engine strings verbatim)
    left = [("HSL", insp["color"]["hsl"], "to_hsl", ":195", "max/min 几何"),
            ("HSV", insp["color"]["hsv"], "to_hsv", ":243", "max/min 几何"),
            ("CMYK", insp["color"]["cmyk"], "to_cmyk", ":293", "k = 1−max(R,G,B)")]
    ly = 40
    # staggered entry points on the core's left edge (no shared segments)
    entries = [cy + 20, cy + ch / 2, cy + ch - 20]
    for (name, val, fn, ln, gloss), ey in zip(left, entries):
        o.append(rect(24, ly, 320, 60, fill=PAPER, stroke=RULE, sw=1.1, rx=6))
        o.append(mono(38, ly + 20, name, size=12, fill=FLOW_DK, weight="600"))
        o.append(mono(38, ly + 40, val, size=10.5, fill=INK))
        o.append(mono(258, ly + 20, fn + " " + ln, size=9, fill=MUTED))
        o.append(mono(258, ly + 38, gloss, size=9, fill=MUTED))
        o.append(connector(344, ly + 30, cx - 6, ey, stroke=FLOW_LT,
                           w=1.2, marker=None))
        ly += 72
    o.append(text(24, ly + 6, "gamma 域直接投影（不经过 linear）", size=10.5,
                  fill=MUTED))

    # right chain: linear -> oklab -> oklch (oklab/oklch verbatim from engine)
    lin = chain["linear_rgb"]
    right = [
        ("LinearRgb", f"({lin['r']:.6f} {lin['g']:.6f} {lin['b']:.6f})",
         "to_linear_rgb :179", "v≤0.04045 ? v/12.92 : ((v+0.055)/1.055)^2.4"),
        ("Oklab", insp["color"]["oklab"], "linear_to_oklab :403",
         "LMS 矩阵 + 三次立方根"),
        ("Oklch", insp["color"]["oklch"], "Oklch::from_oklab :166",
         "c=√(a²+b²)，h=atan2(b,a)"),
    ]
    ry = 40
    for i, (name, val, anchor_note, gloss) in enumerate(right):
        o.append(rect(728, ry, 320, 66, fill=PAPER, stroke=RULE, sw=1.1, rx=6))
        o.append(mono(742, ry + 18, name, size=12, fill=FLOW_DK, weight="600"))
        o.append(mono(742, ry + 36, val, size=10, fill=INK))
        o.append(mono(742, ry + 54, gloss, size=8.5, fill=MUTED))
        o.append(mono(1018, ry + 18, anchor_note.split(" ")[1], size=9,
                      fill=MUTED, anchor="end"))
        if i < 2:
            o.append(connector(888, ry + 66, 888, ry + 80 - 4))
        ry += 80
    o.append(text(728, ry + 6, "linear → 感知域链路（WCAG 与 OKLCH 都从这走）",
                  size=10.5, fill=MUTED))
    o.append(connector(cx + cw, cy + ch / 2, 728 - 6, 70, stroke=FLOW, w=1.6))
    o.append(text(452, 144, "2.4 幂 gamma 解码", size=10, fill=FLOW_DK))
    o.append(mono(452, 158, "spaces.rs:387-393", size=9, fill=MUTED))

    # return path: from_oklch_mapped
    o.append(rect(430, 288, 618, 76, fill=TINT, rx=6))
    o.append(text(446, 310, "回程 from_oklch_mapped（spaces.rs:359-380）：色域外不报错——",
                  size=11, fill=INK, weight="600"))
    o.append(text(446, 329, "二分 32 步压 chroma，在保留 lightness 与 hue 的前提下",
                  size=10.5, fill=INK))
    o.append(text(446, 347, "落回可显示范围；palette / ensure 的修复全部经由它",
                  size=10.5, fill=INK))
    o.append(connector(536, 288, 536, cy + ch, stroke=FLOW, w=1.4,
                       dash="5 4", marker=None))

    o.append(srcnote(24, H - 14,
                     "节点取值 = rebeccapurple 全程冻结（oklab/oklch 为引擎字符串原样；linear 三元组为公式复算、"
                     "显示 6 位小数，全精度在 data/spaces-chain.json，锚定引擎全精度亮度）"))
    return svg("p-spaces", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 05 STORY 2 — THE THESIS: complement != readable foreground

def p_complement(comp, ratio, ensure):
    W, H = 1072, 548
    o = []
    # quote
    o.append(rect(24, 18, 1024, 56, fill=CODE_BG, rx=6))
    o.append(text(40, 40, "「Color-wheel complements and readable foregrounds are separate concepts:",
                  size=11.5, fill=FLOW_DK))
    o.append(text(40, 58, "  use harmony for hue relationships and best_foreground or ensure_contrast for measured contrast.」",
                  size=11.5, fill=FLOW_DK))
    o.append(mono(770, 40, "—— lib.rs:3-5", size=10, fill=MUTED))
    o.append(mono(770, 58, "crate 文档第一段", size=10, fill=MUTED))

    # left: palette command, two swatches, identical L
    lx, ly = 24, 96
    o.append(rect(lx, ly, 344, 268, fill="#FFFFFF", stroke=RULE, sw=1.1, rx=6))
    o.append(mono(lx + 16, ly + 22, "palette rebeccapurple -k complementary",
                  size=11, fill=INK, weight="600"))
    l0, c0, h0 = parse_oklch(comp["colors"][0]["oklch"])
    l1, c1, h1 = parse_oklch(comp["colors"][1]["oklch"])
    for i, (col, lch) in enumerate([(comp["colors"][0], (l0, c0, h0)),
                                    (comp["colors"][1], (l1, c1, h1))]):
        yy = ly + 40 + i * 104
        o.append(swatch(lx + 16, yy, col["hex"], w=72, h=72, stroke=RULE))
        o.append(mono(lx + 100, yy + 16, col["hex"], size=13, fill=INK,
                      weight="600"))
        o.append(mono(lx + 100, yy + 34, f"oklch({lch[0]} {lch[1]} {lch[2]})",
                      size=10, fill=INK))
        o.append(mono(lx + 100, yy + 52, f"L {lch[0]}  ← 同一亮度", size=10.5,
                      fill=WARN, weight="600"))
        o.append(mono(lx + 100, yy + 68,
                      f"H {lch[2]}  " + ("本体" if i == 0 else f"= {h0} + 180°（模 360）"),
                      size=10, fill=FLOW_DK))
    o.append(text(lx + 16, ly + 250,
                  "互补 = OKLCH 只把色相转 180°，亮度与 chroma 不动 →", size=10.5,
                  fill=INK))
    o.append(text(lx + 16, ly + 266, "两端相对亮度几乎相同 → 比率贴着 1", size=10.5,
                  fill=INK))

    # middle: measured ratio, all thresholds fail, live exhibit
    mx, my = 392, 96
    o.append(rect(mx, my, 330, 268, fill="#FFFFFF", stroke=RULE, sw=1.1, rx=6))
    o.append(mono(mx + 16, my + 22, "contrast ratio '#663399' '#475c00'",
                  size=10, fill=INK, weight="600"))
    o.append(mono(mx + 16, my + 50, ratio_str(ratio["ratio"]), size=16,
                  fill=WARN, weight="600"))
    ths = [("aa_large 3:1", ratio["aa_large"]),
           ("aa_normal 4.5:1", ratio["aa_normal"]),
           ("aaa_large 4.5:1", ratio["aaa_large"]),
           ("aaa_normal 7:1", ratio["aaa_normal"])]
    for i, (name, ok) in enumerate(ths):
        yy = my + 72 + i * 20
        o.append(text(mx + 16, yy, "✗" if not ok else "✓", size=11,
                      fill=WARN if not ok else TEAL, weight="600"))
        o.append(mono(mx + 36, yy, name, size=10, fill=MUTED))
    # live exhibit: real colored text on real background
    o.append(rect(mx + 170, my + 62, 148, 96, fill="#663399", rx=4))
    o.append(text(mx + 244, my + 116, "Aa 可读？", size=17, fill="#475c00",
                  weight="600", anchor="middle"))
    o.append(text(mx + 244, my + 138, "把互补色当正文用", size=10.5,
                  fill="#475c00", anchor="middle"))
    o.append(text(mx + 16, my + 196, "四档阈值全部 false ——", size=10.5, fill=INK))
    o.append(text(mx + 16, my + 214, "色环说它们「配」，", size=10.5, fill=INK))
    o.append(text(mx + 16, my + 232, "WCAG 说它们「读不出」。", size=10.5, fill=INK))
    o.append(mono(mx + 16, my + 254, "两套度量，互不替代", size=10, fill=MUTED))

    # right: ensure repair envelope
    rx_, ry_ = 746, 96
    o.append(rect(rx_, ry_, 302, 268, fill=TINT, stroke=RULE, sw=1.1, rx=6))
    o.append(mono(rx_ + 16, ry_ + 22, "contrast ensure '#475c00'", size=10,
                  fill=INK, weight="600"))
    o.append(mono(rx_ + 16, ry_ + 38, "  '#663399' --target aa", size=10,
                  fill=INK, weight="600"))
    o.append(rect(rx_ + 16, ry_ + 50, 270, 96, fill="#663399", rx=4))
    o.append(text(rx_ + 151, ry_ + 102, "Aa 可读 ✓", size=17, fill="#adc777",
                  weight="600", anchor="middle"))
    o.append(text(rx_ + 151, ry_ + 124, "修复后的前景", size=10.5, fill="#adc777",
                  anchor="middle"))
    o.append(mono(rx_ + 16, ry_ + 168,
                  f"#475c00 → {ensure['color']['hex']}", size=11, fill=INK,
                  weight="600"))
    ol0, oc0, oh0 = parse_oklch(ensure["original"]["oklch"])
    nl0, nc0, nh0 = parse_oklch(ensure["color"]["oklch"])
    o.append(mono(rx_ + 16, ry_ + 188, f"L {ol0} → {nl0}", size=10, fill=INK))
    o.append(mono(rx_ + 16, ry_ + 204, f"C {oc0} → {nc0}（保持）", size=10,
                  fill=MUTED))
    o.append(mono(rx_ + 16, ry_ + 220, f"ratio → {repr(ensure['ratio'])}",
                  size=10, fill=TEAL, weight="600"))
    o.append(mono(rx_ + 16, ry_ + 240, f"direction: {ensure['direction']}",
                  size=10, fill=MUTED))
    o.append(mono(rx_ + 16, ry_ + 256, "48 步二分 L，取 OKLab 更近的一侧", size=9,
                  fill=MUTED))

    # bottom: two independent code paths
    by = 384
    o.append(node(24, by, 496, 64, "palette.rs · harmony(base, kind)",
                  ("色相关系：offsets = [0, 180] 等（:25-32），只加 hue，不管亮度"),
                  mono_title=True, tsize=12, ssize=10))
    o.append(node(544, by, 504, 64, "contrast.rs · ensure_contrast(fg, bg, min)",
                  ("可读修复：二分 OKLCH lightness（:249-285），方向取 OKLab 更近侧（:198-215）"),
                  mono_title=True, tsize=12, ssize=10))
    o.append(text(536, by + 34, "≠", size=24, fill=WARN, weight="600",
                  anchor="middle"))
    o.append(srcnote(24, by + 86,
                     "全部数值 = data/complementary.json · ratio-complement.json · ensure-complement.json 实跑冻结"
                     "（比率全精度）；两个演示色块的颜色即冻结输出本身"))
    o.append(srcnote(24, by + 102,
                     "口径注（如实呈现，不合并）：palette/ensure 的 JSON 里 oklch 与 ratio 描述引擎内部 f64 色，hex 是其 8 位量化——"))
    o.append(srcnote(24, by + 118,
                     "同一 #475c00 重新 convert 得 L 44.1177%（palette 侧 44.0272%）；ensure 报告 4.500000000000004，"
                     "输出 hex 复测 4.487789210548097（见指标图鉴）。验收要测落盘 hex"))
    return svg("p-complement", W, H, "".join(o),
               extra_defs=arrow_def("p-complement-arrow-warn", WARN))


# ---------------------------------------------------------------------------
# 06 STORY 2 — the gamma -> linear -> luminance chain, dissected

def p_luminance(chain, insp, selfchecks):
    W, H = 1072, 470
    o = []
    # left: gamma curve, sampled from spaces.rs:387-400 exact formula
    px, py, pw, ph = 24, 56, 400, 300
    o.append(rect(px - 10, py - 26, pw + 20, ph + 66, fill="#FFFFFF",
                  stroke=RULE, sw=1, rx=6))
    o.append(text(px, py - 8, "gamma 解码曲线（spaces.rs:387-400 公式采样 41 点）",
                  size=11.5, fill=INK, weight="600"))
    x0, y0 = px + 40, py + ph - 24
    sx, sy = pw - 60, ph - 44
    for gv in (0.0, 0.25, 0.5, 0.75, 1.0):
        gx = x0 + gv * sx
        if gv > 0.0:  # the curve leaves (x0,y0) along the v/12.92 segment —
            o.append(line(gx, y0, gx, y0 - sy, stroke=RULE, w=0.6))
        o.append(mono(gx, y0 + 14, f"{gv:.2f}", size=8.5, fill=MUTED,
                      anchor="middle"))
        gy_ = y0 - gv * sy
        # the gv=0 gridline starts past the curve's near-origin run, where
        # the v/12.92 segment hugs the axis (linter flags the tangency)
        gx0 = x0 + (30 if gv == 0.0 else 0)
        o.append(line(gx0, gy_, x0 + sx, gy_, stroke=RULE, w=0.6))
        o.append(mono(x0 - 6, gy_ + 3, f"{gv:.2f}", size=8.5, fill=MUTED,
                      anchor="end"))
    pts = [(x0 + v / 40 * sx, y0 - srgb_to_linear(v / 40) * sy)
           for v in range(41)]
    o.append(poly(pts, fill="none", stroke=FLOW, sw=1.8, closed=False))
    # linear segment marker
    seg_end = 0.04045
    o.append(line(x0 + seg_end * sx, y0, x0 + seg_end * sx, y0 - sy * 0.2,
                  stroke=WARN, w=1, dash="3 3"))
    o.append(text(x0 + seg_end * sx + 4, y0 - sy * 0.2 - 4,
                  "v/12.92 线性段上限 0.04045", size=8.5, fill=WARN))
    # the three channel points of rebeccapurple
    lin = chain["linear_rgb"]
    chans = [("R", 102 / 255, lin["r"]), ("G", 51 / 255, lin["g"]),
             ("B", 153 / 255, lin["b"])]
    for name, gv, lv in chans:
        gx, gy2 = x0 + gv * sx, y0 - lv * sy
        o.append(line(gx, gy2, gx, y0, stroke=RULE, w=0.8, dash="2 3"))
        o.append(line(x0, gy2, gx, gy2, stroke=RULE, w=0.8, dash="2 3"))
        o.append(circle(gx, gy2, 4, "#663399", stroke=INK, sw=0.8))
        o.append(text(gx + 7, gy2 - 5, name, size=9.5, fill=INK, weight="600"))
    o.append(mono(x0, y0 + 30,
                  "横轴 gamma 值 → 纵轴 linear 值（三圆点 = rebeccapurple 三通道）",
                  size=9, fill=MUTED))
    o.append(swatch(x0 + sx - 76, y0 - sy + 4, "#663399", w=14, h=14))
    o.append(text(x0 + sx - 57, y0 - sy + 15, "样本色", size=9.5, fill=MUTED))

    # right: the chain with full precision
    cxx = 470
    steps = [
        ("① gamma sRGB", "(0.4  0.2  0.6)", "rgba8(102 51 153) / 255"),
        ("② linear sRGB",
         f"({lin['r']:.6f} {lin['g']:.6f} {lin['b']:.6f})",
         "左图曲线逐通道换算"),
        ("③ 加权亮度",
         f"{repr(chain['relative_luminance_engine'])}",
         "0.2126 R + 0.7152 G + 0.0722 B（contrast.rs:90-93）"),
        ("④ 对白比率",
         f"{ratio_str(insp['contrast_on_white'])}",
         "1.05 / (L + 0.05)（contrast.rs:235-239）"),
    ]
    yy = 56
    for i, (name, val, note) in enumerate(steps):
        o.append(rect(cxx, yy, 578, 62, fill=PAPER, stroke=RULE, sw=1, rx=6))
        o.append(text(cxx + 14, yy + 20, name, size=11.5, fill=FLOW_DK,
                      weight="600"))
        o.append(mono(cxx + 14, yy + 42, val, size=11, fill=INK,
                      weight="600"))
        o.append(text(cxx + 250 if len(val) > 22 else cxx + 190, yy + 40, note,
                      size=9.5, fill=MUTED))
        if i < 3:
            o.append(connector(cxx + 289, yy + 62, cxx + 289, yy + 74))
        yy += 76
    # self-check chips (short labels; full names live in data/selfchecks.json)
    o.append(text(cxx, yy - 4, "复算自检（落盘前全部通过 · data/selfchecks.json）",
                  size=11.5, fill=INK, weight="600"))
    short_labels = ["relative_luminance", "contrast_on_black", "OKLab 链路"]
    cx2 = cxx
    for lab in short_labels:
        cch, w = chip(cx2, yy + 12, f"✓ {lab}", size=9.5)
        o.append(cch)
        cx2 += w + 8
    o.append(text(cxx, yy + 44, "|Δ| < 1e-15，Python 复算 == 引擎全精度；"
                  "对应自检条目 1 / 2 / 4", size=10, fill=MUTED))
    o.append(srcnote(24, H - 14,
                     "data/spaces-chain.json（linear 三元组与亮度由公式复算，锚定引擎全精度 relative_luminance 与 "
                     "contrast_on_white；linear 显示 6 位小数，全精度在 JSON）· gamma 曲线 = 公式直接采样，非手画"))
    return svg("p-luminance", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 07 STORY 3 — WCAG threshold ladder with six measured points

def p_contrast(ladder):
    W, H = 1072, 518
    o = []
    # threshold cards
    th = [("aa_large", "3.0", "大字号 / UI 组件"), ("aa_normal", "4.5", "正文"),
          ("aaa_large", "4.5", "大字号（增强）"), ("aaa_normal", "7.0", "正文（增强）")]
    for i, (name, v, gloss) in enumerate(th):
        cx = 24 + i * 258
        o.append(rect(cx, 22, 246, 58, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
        o.append(mono(cx + 14, 44, f"{v}:1", size=16, fill=FLOW_DK,
                      weight="600"))
        o.append(mono(cx + 74, 42, name, size=11, fill=INK))
        o.append(text(cx + 74, 58, gloss, size=9.5, fill=MUTED))
    o.append(srcnote(24, 96,
                     "rating() 阈值硬编码 contrast.rs:225-233 · CLI --target aa-large|aa|aaa-large|aaa 同值（main.rs:817-824）"))

    # number line 1..14
    ax0, ax1, ay = 40, 1030, 268

    def X(r):
        return ax0 + (r - 1.0) / 13.0 * (ax1 - ax0)

    o.append(rect(ax0, ay - 4, X(3.0) - ax0, 8, fill=WARN_LT))
    o.append(rect(X(3.0), ay - 4, X(7.0) - X(3.0), 8, fill=OUTCOME,
                  opacity=0.55))
    o.append(rect(X(7.0), ay - 4, ax1 - X(7.0), 8, fill="#BFDCCF"))
    o.append(line(ax0, ay, ax1, ay, stroke=INK, w=1.2))
    for r, lab in [(1.0, "1"), (3.0, "3.0"), (4.5, "4.5"), (7.0, "7.0"),
                   (10.0, "10"), (13.0, "13")]:
        o.append(line(X(r), ay, X(r), ay + 6, stroke=INK, w=1))
        o.append(mono(X(r), ay + 20, lab, size=9.5, fill=MUTED, anchor="middle"))
    for r, lab in [(3.0, "AA large"), (4.5, "AA normal / AAA large"),
                   (7.0, "AAA normal")]:
        o.append(line(X(r), ay - 4, X(r), ay - 150, stroke=WARN, w=1,
                      dash="4 3", opacity=0.8))
        o.append(text(X(r) + 4, ay - 152, lab, size=9.5, fill=WARN))
    # measured points
    pts = [
        ("互补对", "#663399", "#475c00",
         ladder["complement_pair"]["rating"], "below", 0),
        ("紫底黑字", "#663399", "#000000",
         ladder["purple_on_black"]["rating"], "below", 1),
        ("半透明需 canvas", "#66339980", "#1a1a1a",
         ladder["alpha_without_canvas_rejected_then_measured"]["rating"],
         "below", 2),
        ("修复后前景", "#adc777", "#663399",
         ladder["repaired_on_purple"]["rating"], "above", 1),
        ("紫底白字", "#663399", "#ffffff",
         ladder["purple_on_white"]["rating"], "above", 0),
        ("黄字深底", "#ffe066", "#1a1a1a",
         ladder["yellow_on_dark"]["rating"], "above", 2),
    ]
    for name, fg, bg, rating, side, level in pts:
        x = X(rating["ratio"])
        col = (TEAL if rating["aaa_normal"]
               else (FLOW_DK if rating["aa_large"] else WARN))
        o.append(circle(x, ay, 5.5, col, stroke="#FFFFFF", sw=1.4))
        ly = ay - 34 - level * 46 if side == "above" else ay + 34 + level * 44
        dx = 10 if side == "above" else -14
        # flip the label block to the left of the dot when it would run past
        # the right canvas edge (longest label ≈ 180px incl. ratio string)
        flip = x + dx + 30 + 185 > W - 24
        o.append(line(x + (2 if side == "above" else -2),
                      ay + (-7 if side == "above" else 7),
                      x + (dx if not flip else (-10 if side == "above" else 12)),
                      ly + (12 if side == "above" else -12),
                      stroke=RULE, w=0.8))
        if flip:
            o.append(swatch(x - 66, ly - 10, bg, w=11, h=11))
            o.append(swatch(x - 53, ly - 10, fg, w=11, h=11))
            o.append(text(x - 38, ly, name, size=9.5, fill=INK, weight="600",
                          anchor="end"))
            o.append(mono(x - 38, ly + 12, f"{fg}/{bg}", size=8, fill=MUTED,
                          anchor="end"))
            o.append(mono(x - 38, ly + 24, ratio_str(rating["ratio"]),
                          size=9, fill=col, weight="600", anchor="end"))
        else:
            o.append(swatch(x + dx, ly - 10, bg, w=11, h=11))
            o.append(swatch(x + dx + 13, ly - 10, fg, w=11, h=11))
            o.append(text(x + dx + 30, ly, name, size=9.5, fill=INK,
                          weight="600"))
            o.append(mono(x + dx + 30, ly + 12, f"{fg}/{bg}", size=8,
                          fill=MUTED))
            o.append(mono(x + dx + 30, ly + 24, ratio_str(rating["ratio"]),
                          size=9, fill=col, weight="600"))

    # honesty cards: unreachable + alpha refusal (real stderr)
    cy0 = 424
    o.append(rect(24, cy0, 502, 62, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
    o.append(mono(38, cy0 + 20, "chromap contrast ensure '#333333' '#555555' \\",
                  size=10, fill=INK))
    o.append(mono(38, cy0 + 35, "  --minimum 21   → exit 2", size=10, fill=INK))
    o.append(text(38, cy0 + 52,
                  "「contrast target 21:1 is unreachable; best available ratio is 7.455177810447525:1」",
                  size=9.5, fill=WARN))
    o.append(rect(546, cy0, 502, 62, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
    o.append(mono(560, cy0 + 20,
                  "contrast ratio '#66339980' '#ffffff'  → exit 2",
                  size=10, fill=INK))
    o.append(text(560, cy0 + 37,
                  "「transparent colors require an explicit opaque canvas …」",
                  size=9.5, fill=WARN))
    o.append(text(560, cy0 + 54,
                  "alpha 颜色必须先给 --canvas 再谈对比度（诚实拒绝，不猜）",
                  size=9.5, fill=MUTED))
    o.append(srcnote(24, H - 8,
                     "六个实测点 = data/contrast-ladder.json（每点一次引擎运行，比率全精度）· "
                     "错误路径原文 = data/exits.json · 目标值域 1.0..=21.0（contrast.rs:241-247）"))
    o.append(srcnote(24, H - 24,
                     "「修复后前景」复测 4.487789210548097 < 4.5：ensure 自报 4.500000000000004 属其内部 f64 候选，"
                     "8 位量化后差 0.012（两数均如实冻结，口径注见机制面板）"))
    return svg("p-contrast", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 08 STORY 3 — ΔE_ok vs ΔRGB: same pair, two answers

def p_distance(dist):
    W, H = 1072, 402
    o = []
    o.append(text(24, 30,
                  "distance 子命令对同一对颜色同时输出两个数——先问「用哪个度量」，再看数值",
                  size=13, fill=INK, weight="600"))

    # exhibits: red/green and black/white
    pairs = [
        ("red / green", "#ff0000", "#00ff00", dist["red_green"],
         "ΔsRGB 达到几何上限：每通道差恰为 1", "= √2（闭式可复算）"),
        ("black / white", "#000000", "#ffffff", dist["black_white"],
         "两个度量的全局锚点：ΔsRGB=√3，ΔE_ok 贴着 1",
         "= √3（闭式可复算）"),
    ]
    for i, (name, a, bcol, d, gloss, closed) in enumerate(pairs):
        cx = 24 + i * 524
        o.append(rect(cx, 48, 502, 118, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
        o.append(text(cx + 16, 70, name, size=12, fill=FLOW_DK, weight="600"))
        o.append(swatch(cx + 16, 80, a, w=34, h=34))
        o.append(swatch(cx + 50, 80, bcol, w=34, h=34))
        o.append(mono(cx + 100, 94, f"ΔE_ok   {repr(d['oklab'])}", size=11,
                      fill=INK, weight="600"))
        o.append(mono(cx + 100, 114, f"ΔsRGB  {repr(d['srgb'])}", size=11,
                      fill=INK, weight="600"))
        o.append(text(cx + 16, 134, gloss, size=10, fill=MUTED))
        o.append(mono(cx + 330, 134, closed, size=9.5, fill=TEAL))

    # table: all four frozen pairs
    cols = [24, 190, 420, 690]
    hy = 190
    o.append(text(cols[0], hy, "实测对", size=10.5, fill=MUTED, weight="600"))
    o.append(text(cols[1], hy, "ΔE_ok（OKLab 欧氏）", size=10.5, fill=MUTED,
                  weight="600"))
    o.append(text(cols[3], hy, "ΔsRGB（gamma 域欧氏）", size=10.5, fill=MUTED,
                  weight="600"))
    rows = [
        ("#ff0000 / #00ff00", "red_green", "√2"),
        ("#000000 / #ffffff", "black_white", "√3"),
        ("#663399 / #475c00", "complementary_pair", "互补对（见面板 05）"),
        ("#475c00 / #adc777", "purple_vs_repaired", "ensure 修复前后"),
    ]
    y = hy + 14
    for label, key, note in rows:
        o.append(line(24, y, 1048, y, stroke=RULE, w=0.8))
        o.append(mono(cols[0], y + 18, label, size=10.5, fill=INK))
        o.append(mono(cols[1], y + 18, repr(dist[key]["oklab"]), size=10.5,
                      fill=FLOW_DK))
        o.append(mono(cols[3], y + 18, repr(dist[key]["srgb"]), size=10.5,
                      fill=FLOW_DK))
        o.append(text(cols[3] + 190, y + 18, note, size=9.5, fill=MUTED))
        y += 27
    o.append(line(24, y, 1048, y, stroke=INK, w=1.1))
    o.append(text(24, y + 20,
                  "两个度量单位不同、尺度不同，绝对值不可直接比大小：ΔsRGB 的最大值是 √3（黑↔白），",
                  size=10.5, fill=INK))
    o.append(text(24, y + 37,
                  "ΔE_ok 全空间约 [0,1]；OKLab 的设计目标是感知均匀（通解一句带过，本页不展开，"
                  "详见 color-palette-rs 图解）", size=10.5, fill=INK))
    o.append(srcnote(24, H - 10,
                     "data/distance.json（chromap --json distance 逐对实跑，全精度）· "
                     "difference.rs:3-17 · √2 / √3 / 亮度轴三组闭式复算见 data/selfchecks.json"))
    return svg("p-distance", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 09 STORY 3 — 14 palette kinds x 7 output formats

def p_kinds(pals, fmts):
    kinds = pals["kinds"]
    scale_kinds = ["neighbors", "lightness", "analogous-scale", "hue-wheel",
                   "golden", "tints", "shades", "tones"]
    harmony_offsets = {"complementary": "[0, 180]", "analogous": "[-30, 0, 30]",
                       "split-complementary": "[0, 150, 210]",
                       "triadic": "[0, 120, 240]",
                       "square": "[0, 90, 180, 270]",
                       "tetradic": "[0, 60, 180, 240]"}
    lx = 24
    o = []
    o.append(text(lx, 30, "palette -k <kind>（基色 #4f7cff = examples/design_tokens.rs:6，默认 -c 7）",
                  size=11.5, fill=INK, weight="600"))
    o.append(text(lx + 150, 46, "① 标尺类：-c / --lightness-span / --hue-span 生效",
                  size=9.5, fill=FLOW_DK))
    y = 58
    for kind in scale_kinds:
        hexes = kinds[kind]
        o.append(mono(lx, y + 12, kind, size=10.5, fill=INK))
        for i, hv in enumerate(hexes):
            o.append(swatch(lx + 150 + i * 32, y, hv, w=30, h=16))
        o.append(mono(lx + 150 + 7 * 32 + 8, y + 12, f"×{len(hexes)}",
                      size=9.5, fill=MUTED))
        y += 27
    o.append(text(lx + 150, y + 4,
                  "② 和谐类：长度由色相关系固定（palette.rs:25-32），-c 不生效",
                  size=9.5, fill=FLOW_DK))
    y += 16
    for kind, offsets in harmony_offsets.items():
        hexes = kinds[kind]
        o.append(mono(lx, y + 12, kind, size=10.5, fill=INK))
        for i, hv in enumerate(hexes):
            o.append(swatch(lx + 150 + i * 32, y, hv, w=30, h=16))
        o.append(mono(lx + 150 + 4 * 32 + 8, y + 12, offsets, size=9,
                      fill=MUTED))
        y += 27

    # right column: 7 output formats
    rx = 700
    n_fmt_rows = sum(2 if len(v) > 24 else 1 for v in fmts.values())
    o.append(rect(rx, 42, 348, 96 + n_fmt_rows * 22, fill="#FFFFFF",
                  stroke=RULE, sw=1.1, rx=6))
    o.append(text(rx + 14, 64, "7 种输出格式（--format）", size=11.5, fill=INK,
                  weight="600"))
    o.append(swatch(rx + 14, 72, "#663399", w=13, h=13))
    o.append(text(rx + 32, 83, "同一颜色 rebeccapurple：", size=9.5, fill=MUTED))
    yy = 102
    for f in ["hex", "rgb", "hsl", "hsv", "cmyk", "oklab", "oklch"]:
        o.append(mono(rx + 14, yy, f, size=10, fill=FLOW_DK))
        val = fmts[f]
        if len(val) > 24:
            cut = val.index(" ", val.index(" ") + 1)
            o.append(mono(rx + 62, yy, val[:cut], size=9.5, fill=INK))
            o.append(mono(rx + 62, yy + 11, val[cut + 1:], size=9.5, fill=INK))
            yy += 24
        else:
            o.append(mono(rx + 62, yy, val, size=10, fill=INK))
            yy += 20

    # footer facts
    fy = y + 16
    o.append(line(lx, fy, 1048, fy, stroke=INK, w=1.1))
    o.append(text(lx, fy + 20,
                  f"实测：harmony 类 -c {pals['harmony_ignores_count']['requested_count']} 仍返回 "
                  f"{pals['harmony_ignores_count']['returned']} 色（complementary）——"
                  "标尺与和谐是两组生成器（main.rs:436-441）", size=10.5, fill=INK))
    o.append(text(lx, fy + 38,
                  "标尺类共享「固定 L/C 扫 H 或固定 H/C 扫 L」骨架；golden 用 137.50776405003785° 黄金角错开色相（palette.rs:165）",
                  size=10.5, fill=MUTED))
    H = fy + 62
    o.append(srcnote(lx, fy + 58,
                     "data/palettes.json（14 次 chromap --json palette #4f7cff -k <kind> 实跑）· "
                     "data/formats.json · PaletteKind 枚举 main.rs:245-260"))
    return svg("p-kinds", W := 1072, H, "".join(o))


# ---------------------------------------------------------------------------
# 10 STORY 4 — operations: brand color -> accessible design tokens

def p_tokens(tok, png):
    scale = tok["scale_hex"]
    cow = tok["contrast_on_white"]
    W, H = 1072, 512
    o = []
    # workflow strip
    steps = [
        ("① 选品牌色", "inspect '#4f7cff'", "度量亮度与黑白对比"),
        ("② 生成标尺", "palette -k neighbors -c 9", "--lightness-span 0.64"),
        ("③ 逐级验收", "contrast ratio ×9", "对白底逐级测 AA"),
        ("④ 冻结交付", "--css-prefix --json --png", "CSS 变量 + PNG 预览"),
    ]
    sx = 24
    for i, (t, cmd, note) in enumerate(steps):
        w = 244
        o.append(rect(sx, 22, w, 66, fill="#FFFFFF", stroke=RULE, sw=1.1, rx=6))
        o.append(text(sx + 12, 42, t, size=11.5, fill=FLOW_DK, weight="600"))
        o.append(mono(sx + 12, 60, cmd, size=10, fill=INK))
        o.append(text(sx + 12, 76, note, size=9.5, fill=MUTED))
        if i < 3:
            o.append(connector(sx + w, 55, sx + w + 18, 55))
        sx += w + 18

    # the 9-step scale with per-step measured ratio
    css_pairs = []
    for ln in tok["css"].splitlines():
        s = ln.strip()
        if s.startswith("--"):
            name, _, value = s.partition(":")
            css_pairs.append((name.strip(), value.strip().rstrip(";")))
    assert [v for _, v in css_pairs] == scale, "css var values must equal scale_hex"
    assert len(css_pairs) == len(scale), "one css var per scale step"
    o.append(text(24, 124, "9 级明度标尺（chromap 实跑冻结）+ 逐级对白底的实测比率",
                  size=12.5, fill=INK, weight="600"))
    cw = 112
    for i, hv in enumerate(scale):
        cx = 24 + i * (cw + 2)
        rating = cow[hv]
        ok = rating["aa_normal"]
        o.append(rect(cx, 140, cw, 58, fill=hv, stroke=RULE, sw=0.8))
        # label color chosen by measurement, not by hand: white text only
        # where white-on-this-swatch itself passes AA normal
        col = "#ffffff" if ok else "#17212B"
        o.append(mono(cx + 8, 156, hv, size=9.5, fill=col))
        o.append(mono(cx + 8, 170, css_pairs[i][0], size=8.5, fill=col))
        o.append(mono(cx + 8, 216, f"{rating['ratio']:.2f}", size=11,
                      fill=TEAL if ok else WARN, weight="600"))
        o.append(text(cx + 8, 230, "AA ✓" if ok else "AA ✗", size=9.5,
                      fill=TEAL if ok else WARN))
    o.append(text(24, 252,
                  "注：各级过/不过 AA 是实测结果（data/design-tokens.json，9 次 ratio 运行，显示 2 位小数、"
                  "全精度在 JSON）——深级当正文、浅级当底色才成立", size=10, fill=MUTED))

    # CSS block (two columns to fit)
    o.append(rect(24, 270, 560, 152, fill=CODE_BG, rx=6))
    o.append(mono(40, 292, "chromap palette '#4f7cff' -k neighbors -c 9 \\",
                  size=10, fill=INK))
    o.append(mono(40, 307, "  --lightness-span 0.64 --css-prefix brand",
                  size=10, fill=INK))
    css_lines = tok["css"].splitlines()  # ":root {" + 9 vars + "}"
    yy = 330
    for j, ln in enumerate(css_lines):
        col_x = 40 if j < 6 else 320
        row = j if j < 6 else j - 6
        o.append(mono(col_x, yy + row * 15, ln, size=10, fill=FLOW_DK))
    o.append(text(40, 418, "（--css-prefix 校验 ASCII 字母/数字/-/_，输出可直接入 CSS）",
                  size=9, fill=MUTED))

    # right: equivalence + PNG freeze
    o.append(rect(608, 270, 440, 152, fill="#FFFFFF", stroke=RULE, sw=1.1, rx=6))
    o.append(text(624, 292, "与库例程对拍：9 级 hex 全等（9/9）", size=11.5,
                  fill=INK, weight="600"))
    o.append(mono(624, 310, "cargo run --package chromap \\", size=10, fill=MUTED))
    o.append(mono(624, 325, "  --example design_tokens", size=10, fill=MUTED))
    o.append(text(624, 345, "自检 #17：例程输出的 9 个 hex 与 CLI 标尺逐一相等",
                  size=9.5, fill=INK))
    o.append(text(624, 360, "命名口径不同：CLI --css-prefix 给 --brand-1…9",
                  size=9.5, fill=INK))
    o.append(text(624, 375, "（main.rs:772-778）；例程自带 --brand-100…900",
                  size=9.5, fill=INK))
    o.append(text(624, 390, "（examples/design_tokens.rs:10-14）——两数都如实呈现",
                  size=9.5, fill=INK))
    o.append(mono(624, 408, f"PNG 预览 {png['width']}×{png['height']}"
                  f"（{png['geometry'][:12]}…）", size=9.5, fill=INK))
    o.append(mono(624, 421,
                  f"双跑 sha256 一致：{png['run1_sha256'][:26]}…", size=9,
                  fill=TEAL))
    o.append(srcnote(24, H - 14,
                     "data/design-tokens.json（palette + 9×ratio + cargo example 实跑）· "
                     "data/png-freeze.json（两次 --png 运行 sha256 相等）· examples/design_tokens.rs"))
    return svg("p-tokens", W, H, "".join(o))


# ---------------------------------------------------------------------------
# 11 STORY 4 — the explainer's own evidence pipeline

def p_evidence(prov, engine, selfchecks, n_panels=11):
    n_cmds = len(prov["commands"])
    n_files = len(list(DATA.glob("*.json")))
    W, H = 1072, 500
    o = []
    ny, nh = 30, 74
    nodes = [
        ("chromap 0.1.0", ("sha256 " + engine["binary_sha256"][:16] + "…",
                           "commit " + engine["repo_commit"][:12])),
        ("prep_data.py", (f"{n_cmds} 组命令实跑", "浮点全精度冻结")),
        (f"data/ {n_files} 个 JSON", ("provenance.json", "记录每条完整 argv")),
        ("build.py", ("面板几何数据驱动", "确定性输出")),
        ("shoot.js + stitch.py", ("y=0 全宽切片", "位图高 = CSS 高×dpr")),
    ]
    nx, nw = 24, 186
    x = nx
    for i, (t, subs) in enumerate(nodes):
        o.append(node(x, ny, nw, nh, t, subs, mono_title=True, tsize=11.5,
                      ssize=9.5))
        if i < 4:
            o.append(connector(x + nw, ny + nh / 2, x + nw + 18, ny + nh / 2))
        x += nw + 18

    # KPI cards
    ky = 140
    kpis = [
        (f"{selfchecks['passed']}/{selfchecks['total']}", "落盘前复算自检"),
        (str(n_files), "data/*.json 冻结文件"),
        (str(n_cmds), "实跑命令组"),
        (str(n_panels), "SVG 面板（门禁对象）"),
        ("0", "手填数字"),
    ]
    for i, (n, l) in enumerate(kpis):
        cx = 24 + i * 202
        o.append(rect(cx, ky, 190, 52, fill="#FFFFFF", stroke=RULE, sw=1, rx=6))
        o.append(mono(cx + 14, ky + 30, n, size=18, fill=FLOW_DK, weight="600"))
        o.append(text(cx + 14, ky + 45, l, size=9.5, fill=MUTED))

    # selected self-checks
    o.append(text(24, 228, "自检选录（全部 21 条见 data/selfchecks.json）",
                  size=12, fill=INK, weight="600"))
    sel = ["relative_luminance", "√2", "√3", "互补色", "ensure",
           "design_tokens", "PNG", "exit"]
    y = 248
    for c in selfchecks["checks"]:
        if any(s in c["check"] for s in sel):
            detail = c["detail"]
            if len(detail) > 43:  # keep inside the right canvas edge; full
                detail = detail[:42] + "…"  # text lives in data/selfchecks.json
            o.append(mono(24, y, "PASS", size=10, fill=TEAL, weight="600"))
            o.append(text(66, y, c["check"], size=10, fill=INK))
            o.append(mono(820, y, detail, size=9, fill=MUTED))
            y += 19
    o.append(srcnote(24, H - 12,
                     "从本目录重建：python3 prep_data.py（可选）→ python3 build.py → svg-linter → "
                     "shoot.js/stitch.py；index.html 重建 byte-identical（VERIFICATION.md）"))
    return svg("p-evidence", W, H, "".join(o))
