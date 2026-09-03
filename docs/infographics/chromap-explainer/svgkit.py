#!/usr/bin/env python3
"""SVG primitives for the chromap explainer. Literal hex only (no CSS var()
in SVG presentation attributes — keeps svg-linter findings meaningful).
Connector grammar: orthogonal elbows r=8, labels beside strokes, own defs per
svg (no cross-svg marker borrowing). Role palette identical to the
lut-ops / pm-file-parser explainer system (light paper, blue editorial).
"""
from __future__ import annotations

# Bound role palette (same visual language as lut-ops / pm-file-parser)
PAPER = "#F7F4EE"
INK = "#17212B"
MUTED = "#5D6873"
RULE = "#D9E1E3"
FLOW = "#356A79"
FLOW_DK = "#28505C"
FLOW_LT = "#7FA3AD"
FLOW_XLT = "#B9CDD3"
TEAL = "#2A9D8F"
TINT = "#DDF2EC"
WARN = "#E76F51"
WARN_LT = "#F4C7B8"
OUTCOME = "#E9C46A"
CODE_BG = "#F1ECE2"

FONT_DISPLAY = "'Source Han Serif SC','PingFang SC',serif"
FONT_BODY = "'Source Han Sans SC','PingFang SC',sans-serif"
FONT_MONO = "'0xProto Nerd Font','SF Mono',Menlo,monospace"


def esc(s: str) -> str:
    return (s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            .replace('"', "&quot;"))


def el(tag: str, attrs: dict | None = None, *children: str) -> str:
    a = " ".join(f'{k}="{esc(str(v)) if not str(v).startswith("url") else v}"'
                 for k, v in (attrs or {}).items())
    inner = "".join(children)
    if not children:
        return f"<{tag} {a}/>"
    return f"<{tag} {a}>{inner}</{tag}>"


def text_width(s: str, size: float, mono=False) -> float:
    """Conservative advance-width estimate (ASCII 0.65em mono / 0.55em body;
    CJK & fullwidth punctuation 1em). Mono factor sits above the ~0.633em the
    svg-linter font measurement reports, so auto-sized chips never collide.
    Used only for chip sizing and spacing."""
    w = 0.0
    for c in s:
        if ord(c) > 0x2E7F:          # CJK + fullwidth forms
            w += 1.0
        elif mono:
            w += 0.65
        else:
            w += 0.55
    return w * size


def text(x, y, s, size=13, fill=INK, family=FONT_BODY, weight="400",
         anchor="start", style=None, spacing=None):
    attrs = {"x": round(x, 2), "y": round(y, 2), "font-size": size,
             "fill": fill, "font-family": family, "font-weight": weight,
             "text-anchor": anchor}
    if style:
        attrs["font-style"] = style
    if spacing:
        attrs["letter-spacing"] = spacing
    return el("text", attrs, esc(s))


def mono(x, y, s, size=12, fill=INK, weight="400", anchor="start"):
    return text(x, y, s, size=size, fill=fill, family=FONT_MONO,
                weight=weight, anchor=anchor)


def line(x1, y1, x2, y2, stroke=RULE, w=1, dash=None, cap="butt", opacity=None):
    a = {"x1": round(x1, 2), "y1": round(y1, 2), "x2": round(x2, 2),
         "y2": round(y2, 2), "stroke": stroke, "stroke-width": w,
         "stroke-linecap": cap}
    if dash:
        a["stroke-dasharray"] = dash
    if opacity:
        a["opacity"] = opacity
    return el("line", a)


def rect(x, y, w, h, fill="none", stroke=None, sw=1, rx=0, dash=None,
         opacity=None):
    a = {"x": round(x, 2), "y": round(y, 2), "width": round(w, 2),
         "height": round(h, 2), "fill": fill, "rx": rx}
    if stroke:
        a["stroke"] = stroke
        a["stroke-width"] = sw
    if dash:
        a["stroke-dasharray"] = dash
    if opacity:
        a["opacity"] = opacity
    return el("rect", a)


def poly(pts, fill="none", stroke=None, sw=1, dash=None, opacity=None,
         closed=True, join="round", cap="round"):
    d = "M " + " L ".join(f"{round(x, 2)},{round(y, 2)}" for x, y in pts)
    if closed:
        d += " Z"
    a = {"d": d, "fill": fill, "stroke-linejoin": join, "stroke-linecap": cap}
    if stroke:
        a["stroke"] = stroke
        a["stroke-width"] = sw
    if dash:
        a["stroke-dasharray"] = dash
    if opacity:
        a["opacity"] = opacity
    return el("path", a)


def circle(cx, cy, r, fill, stroke=None, sw=1):
    a = {"cx": round(cx, 2), "cy": round(cy, 2), "r": r, "fill": fill}
    if stroke:
        a["stroke"] = stroke
        a["stroke-width"] = sw
    return el("circle", a)


def arrow_def(defs_id, color, size=5):
    """Own defs per diagram (extraction-safe: no cross-svg borrowing)."""
    return (f'<marker id="{defs_id}" viewBox="0 0 10 10" refX="8.5" refY="5" '
            f'markerWidth="{size}" markerHeight="{size}" '
            f'orient="auto-start-reverse">'
            f'<path d="M 0,1 L 9,5 L 0,9 z" fill="{color}"/></marker>')


def elbow(x1, y1, x2, y2, r=8, mid=None):
    """Two-bend orthogonal path with quarter-arc corners.
    Horizontal-first when travel is mainly horizontal; vertical-first otherwise."""
    if mid is None:
        mid = (x1 + x2) / 2 if abs(x2 - x1) >= abs(y2 - y1) else (y1 + y2) / 2
    if abs(x2 - x1) >= abs(y2 - y1):  # horizontal dominant: H-V-H
        d = (f"M {x1:.2f},{y1:.2f} H {mid - r if x2 >= x1 else mid + r:.2f} "
             f"Q {mid:.2f},{y1:.2f} {mid:.2f},{y1 + (r if y2 >= y1 else -r):.2f} ")
        ysign = 1 if y2 >= y1 else -1
        d += f"V {y2 - ysign * r:.2f} Q {mid:.2f},{y2:.2f} "
        d += f"{mid + (r if x2 >= x1 else -r):.2f},{y2:.2f} H {x2:.2f}"
        return d
    # vertical dominant: V-H-V
    xsign = 1 if x2 >= x1 else -1
    ysign = 1 if y2 >= y1 else -1
    midy = mid if isinstance(mid, float) else (y1 + y2) / 2
    d = (f"M {x1:.2f},{y1:.2f} V {midy - ysign * r:.2f} "
         f"Q {x1:.2f},{midy:.2f} {x1 + xsign * r:.2f},{midy:.2f} "
         f"H {x2 - xsign * r:.2f} Q {x2:.2f},{midy:.2f} {x2:.2f},{midy + ysign * r:.2f} "
         f"V {y2:.2f}")
    return d


def connector(x1, y1, x2, y2, stroke=FLOW, w=1.6, marker="arrow", r=8,
              mid=None, dash=None, opacity=None):
    if x1 == x2 or y1 == y2:  # straight segment — no degenerate bends
        d = f"M {round(x1,2)},{round(y1,2)} L {round(x2,2)},{round(y2,2)}"
    else:
        d = elbow(x1, y1, x2, y2, r=r, mid=mid)
    a = {"d": d, "fill": "none", "stroke": stroke, "stroke-width": w}
    if marker:
        a["marker-end"] = f"url(#{marker})"
    if dash:
        a["stroke-dasharray"] = dash
    if opacity:
        a["opacity"] = opacity
    return el("path", a)


def svg(svg_id, width, height, body, extra_defs=""):
    # connector() emits a generic url(#arrow); rewrite to this svg's own defs id
    body = body.replace("url(#arrow)", f"url(#{svg_id}-arrow)")
    body = body.replace("url(#arrow-warn)", f"url(#{svg_id}-arrow-warn)")
    return (f'<svg id="{svg_id}" xmlns="http://www.w3.org/2000/svg" '
            f'viewBox="0 0 {width} {height}" width="{width}" height="{height}" '
            f'role="img">'
            f"<defs>{arrow_def(f'{svg_id}-arrow', FLOW)}{extra_defs}</defs>"
            f"{body}</svg>")
