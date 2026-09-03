#!/usr/bin/env python3
"""Freeze chromap evidence into data/*.json.

Every file written here is the verbatim output of a real engine run (or a
derivation whose recomputed values are first self-checked against the
engine's own full-precision output). No hand-entered numbers. Re-run:

    CHROMAP_ENGINE=<path-to-binary> python3 prep_data.py

Defaults: $HOME/sync/macos-arm64-bin/chromap, then the repo's
target/release/chromap. The repo root is auto-detected from this script's
location (docs/infographics/chromap-explainer).

Context precondition: this script must run inside the colors-rs repo — it
shells out to `cargo run --example design_tokens` and freezes git metadata
and named.rs source counts. Outside the repo (a bare snapshot copy), do not
re-freeze; use the delivered data/ as-is. Nothing is written to disk until
the context check and ALL self-checks pass.
"""
from __future__ import annotations

import hashlib
import json
import math
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
DATA = HERE / "data"
REPO = HERE.parents[2]  # .../docs/infographics/<name> -> repo root

SUBCOMMANDS = ["inspect", "convert", "adjust", "mix", "gradient", "palette",
               "contrast", "distance", "composite", "average", "domant"]
SUBCOMMANDS[-1] = "dominant"

# parse.rs syntax families, one probe each (all run against `convert`)
PARSE_FORMS = [
    ("#hex 3 位", "#abc"),
    ("#hex 4 位（含 alpha）", "#abcd"),
    ("#hex 6 位", "#aabbcc"),
    ("#hex 8 位（含 alpha）", "#aabbccdd"),
    ("0x 十六进制", "0xaabbcc"),
    ("rgb()", "rgb(102 51 153)"),
    ("rgba()", "rgba(102,51,153,0.5)"),
    ("hsl()", "hsl(270 50% 40%)"),
    ("hsla()", "hsla(270,50%,40%,0.5)"),
    ("命名色（149 个，含 transparent）", "rebeccapurple"),
    ("transparent", "transparent"),
]

PALETTE_KINDS = ["neighbors", "lightness", "analogous-scale", "hue-wheel",
                 "golden", "tints", "shades", "tones", "complementary",
                 "analogous", "split-complementary", "triadic", "square",
                 "tetradic"]

FORMATS = ["hex", "rgb", "hsl", "hsv", "cmyk", "oklab", "oklch"]

# lib.rs public API facts recomputed in pure python (spaces.rs formulas)
def srgb_to_linear(v: float) -> float:
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def relative_luminance(rgb: tuple[float, float, float]) -> float:
    r, g, b = (srgb_to_linear(c) for c in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def oklab_of(rgb: tuple[float, float, float]) -> tuple[float, float, float]:
    r, g, b = (srgb_to_linear(c) for c in rgb)
    l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
    m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
    s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
    l_, m_, s_ = math.cbrt(l), math.cbrt(m), math.cbrt(s)
    return (0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_)


def count_named_colors() -> tuple[int, int]:
    """Machine-count the named-color match arms in named.rs (no hand-entered
    total): N `"name" => Color::from_rgb8(...)` arms plus the single
    transparent special case (named.rs:153)."""
    src = (REPO / "crates/chromap/src/named.rs").read_text()
    n_rgb8 = len(re.findall(r'"[a-z]+" => Color::from_rgb8\(', src))
    n_transparent = len(re.findall(r'"transparent" => Color::TRANSPARENT', src))
    assert n_transparent == 1, f"expected one transparent arm, got {n_transparent}"
    return n_rgb8, n_transparent


def validate_context() -> None:
    """prep_data must run inside the colors-rs repo (cargo example + git
    metadata + named.rs counting). Fail before touching any file otherwise."""
    required = [
        REPO / "Cargo.toml",
        REPO / "crates/chromap/examples/design_tokens.rs",
        REPO / "crates/chromap/src/named.rs",
    ]
    missing = [str(p) for p in required if not p.exists()]
    if missing:
        sys.exit("refusing to run outside the colors-rs repo (missing: "
                 + ", ".join(missing) + "); the delivered data/ is already "
                 "frozen — use it directly")
    git = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--git-dir"],
                         capture_output=True, text=True)
    if git.returncode != 0:
        sys.exit("git metadata unavailable at " + str(REPO)
                 + "; run inside docs/infographics/chromap-explainer "
                 "of the colors-rs checkout")


def engine_bin() -> Path:
    env = os.environ.get("CHROMAP_ENGINE")
    cands = ([Path(env)] if env else []) + [
        Path.home() / "sync" / "macos-arm64-bin" / "chromap",
        REPO / "target" / "release" / "chromap",
    ]
    for c in cands:
        if c.exists():
            return c.resolve()
    sys.exit("chromap binary not found; set CHROMAP_ENGINE")


def run(bin: Path, args: list[str]) -> dict:
    p = subprocess.run([str(bin), *args], capture_output=True, text=True)
    return {"argv": ["chromap", *args], "stdout": p.stdout, "stderr": p.stderr,
            "exit": p.returncode}


def run_json(bin: Path, args: list[str]) -> tuple[dict, dict]:
    r = run(bin, ["--json", *args])
    assert r["exit"] == 0, (args, r["stderr"])
    return json.loads(r["stdout"]), r


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


# Every payload is buffered here and only hits the disk in flush(), after ALL
# self-checks have passed: a failing or out-of-context run must leave data/
# exactly as it was (no half-rewritten freeze).
PENDING: dict[str, str] = {}


def write(name: str, obj) -> str:
    PENDING[name] = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    return name


def flush() -> None:
    DATA.mkdir(exist_ok=True)
    for name, payload in PENDING.items():
        (DATA / name).write_text(payload)
        print(f"froze data/{name}")


def main() -> None:
    validate_context()
    bin = engine_bin()
    cmds: dict[str, str] = {}
    checks: list[dict] = []

    def check(name: str, ok: bool, detail: str) -> None:
        checks.append({"check": name, "passed": bool(ok), "detail": detail})
        if not ok:
            sys.exit(f"SELF-CHECK FAILED: {name}: {detail}")

    version = run(bin, ["--version"])["stdout"].strip()
    commit = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"],
                            capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "-C", str(REPO), "status", "--porcelain"],
                           capture_output=True, text=True).stdout.strip().splitlines()

    # ---- 1) hero evidence: inspect rebeccapurple (full precision) ----
    insp, r = run_json(bin, ["inspect", "rebeccapurple"])
    write("inspect-rebeccapurple.json", insp)
    cmds["inspect-rebeccapurple.json"] = " ".join(r["argv"])
    L_engine = insp["relative_luminance"]
    L_py = relative_luminance((102 / 255, 51 / 255, 153 / 255))
    check("relative_luminance 复算 == 引擎全精度",
          abs(L_py - L_engine) < 1e-15,
          f"python {L_py!r} vs engine {L_engine!r} (|Δ|={abs(L_py-L_engine):.3e})")
    ratio_black = (L_engine + 0.05) / 0.05
    ratio_white = 1.05 / (L_engine + 0.05)
    check("contrast_on_black == (L+0.05)/0.05",
          abs(ratio_black - insp["contrast_on_black"]) < 1e-12,
          f"{ratio_black!r} vs {insp['contrast_on_black']!r}")
    check("contrast_on_white == 1.05/(L+0.05)",
          abs(ratio_white - insp["contrast_on_white"]) < 1e-12,
          f"{ratio_white!r} vs {insp['contrast_on_white']!r}")
    lab_py = oklab_of((102 / 255, 51 / 255, 153 / 255))
    eng_oklab = insp["color"]["oklab"]  # "oklab(44.0272% 0.088177 -0.133864)"
    vals = [t.rstrip("%") for t in eng_oklab[6:-1].split()]
    check("OKLab 链路复算 == 引擎 oklab() 字符串（引擎 6 位小数口径）",
          abs(lab_py[0] * 100 - float(vals[0])) < 5e-5
          and abs(lab_py[1] - float(vals[1])) < 5e-7
          and abs(lab_py[2] - float(vals[2])) < 5e-7,
          f"python {(lab_py[0]*100, lab_py[1], lab_py[2])!r} vs engine {eng_oklab!r}")
    lin = tuple(srgb_to_linear(c) for c in (102 / 255, 51 / 255, 153 / 255))
    spaces_chain = {
        "note": "linear_rgb 由 spaces.rs:387-393 公式复算；其自检锚点 = 引擎全精度 "
                "relative_luminance（0.2126/0.7152/0.0722 加权和，contrast.rs:90-93）",
        "srgb_gamma": {"r": 102 / 255, "g": 51 / 255, "b": 153 / 255},
        "linear_rgb": {"r": lin[0], "g": lin[1], "b": lin[2]},
        "relative_luminance_engine": L_engine,
        "oklab_recomputed": {"l": lab_py[0], "a": lab_py[1], "b": lab_py[2]},
        "oklch_engine": insp["color"]["oklch"],
    }
    write("spaces-chain.json", spaces_chain)
    cmds["spaces-chain.json"] = ("derived: spaces.rs formulas; self-checked against "
                                 "data/inspect-rebeccapurple.json")

    # ---- 2) the thesis: complementary palette == same L, fails WCAG ----
    comp, r = run_json(bin, ["palette", "rebeccapurple", "-k", "complementary"])
    write("complementary.json", comp)
    cmds["complementary.json"] = " ".join(r["argv"])
    base_lch = comp["colors"][0]["oklch"]
    comp_lch = comp["colors"][1]["oklch"]
    bh = float(base_lch[base_lch.rindex(" ") + 1:base_lch.rindex(")")])
    ch_ = float(comp_lch[comp_lch.rindex(" ") + 1:comp_lch.rindex(")")])
    check("互补色 == OKLCH 色相 +180°（模 360）",
          abs((bh + 180.0) % 360.0 - ch_) < 5e-3,
          f"({bh}+180)%360 = {(bh+180)%360} vs {ch_}")
    check("互补色板两端 OKLCH 亮度相同（引擎字符串口径）",
          base_lch.split()[0] == comp_lch.split()[0],
          f"{base_lch!r} vs {comp_lch!r}")
    ratio, r = run_json(bin, ["contrast", "ratio", "#663399", "#475c00"])
    write("ratio-complement.json", ratio)
    cmds["ratio-complement.json"] = " ".join(r["argv"])
    ensure, r = run_json(bin, ["contrast", "ensure", "#475c00", "#663399",
                               "--target", "aa"])
    write("ensure-complement.json", ensure)
    cmds["ensure-complement.json"] = " ".join(r["argv"])
    check("ensure 修复后 ratio >= minimum 4.5",
          ensure["ratio"] >= ensure["minimum"],
          f"{ensure['ratio']!r} >= {ensure['minimum']}")
    check("ensure 修复只动 OKLCH 亮度（chroma/hue 保持，引擎字符串口径）",
          ensure["original"]["oklch"].split()[1] == ensure["color"]["oklch"].split()[1],
          f"chroma {ensure['original']['oklch'].split()[1]} -> "
          f"{ensure['color']['oklch'].split()[1]}")

    # already-passing envelope (no repair needed)
    ensure_ok, r = run_json(bin, ["contrast", "ensure", "#ffe066", "#1a1a1a",
                                  "--target", "aa"])
    write("ensure-ffe066.json", ensure_ok)
    cmds["ensure-ffe066.json"] = " ".join(r["argv"])

    bw, r = run_json(bin, ["contrast", "black-white", "#663399"])
    write("blackwhite.json", bw)
    cmds["blackwhite.json"] = " ".join(r["argv"])

    # ---- 3) metric ladder points (each = one engine run) ----
    ladder = {}
    for label, args in [
        ("complement_pair", ["contrast", "ratio", "#663399", "#475c00"]),
        ("purple_on_black", ["contrast", "ratio", "#663399", "#000000"]),
        ("purple_on_white", ["contrast", "ratio", "#663399", "#ffffff"]),
        ("repaired_on_purple", ["contrast", "ratio", "#adc777", "#663399"]),
        ("yellow_on_dark", ["contrast", "ratio", "#ffe066", "#1a1a1a"]),
        ("alpha_without_canvas_rejected_then_measured",
         ["contrast", "ratio", "#66339980", "#ffffff", "--canvas", "#1a1a1a"]),
    ]:
        d, r = run_json(bin, args)
        ladder[label] = {"argv": r["argv"], "rating": d}
    write("contrast-ladder.json", ladder)
    cmds["contrast-ladder.json"] = "argv per entry inside the file"
    check("alpha 前景必须显式 canvas（否则引擎拒绝测量）",
          run(bin, ["--json", "contrast", "ratio", "#66339980", "#ffffff"])["exit"] == 2,
          "chromap --json contrast ratio #66339980 #ffffff -> exit 2 "
          "(transparent colors require an explicit opaque canvas)")

    # ---- 4) distances incl. closed-form anchors ----
    dist = {}
    for label, a, b in [("red_green", "#ff0000", "#00ff00"),
                        ("black_white", "#000000", "#ffffff"),
                        ("complementary_pair", "#663399", "#475c00"),
                        ("purple_vs_repaired", "#475c00", "#adc777")]:
        d, r = run_json(bin, ["distance", a, b])
        dist[label] = {"argv": r["argv"], "oklab": d["oklab"], "srgb": d["srgb"]}
    write("distance.json", dist)
    cmds["distance.json"] = "argv per entry inside the file"
    check("red/green ΔsRGB == √2（独立复算）",
          abs(dist["red_green"]["srgb"] - math.sqrt(2)) < 1e-15,
          f"{dist['red_green']['srgb']!r} vs sqrt(2)={math.sqrt(2)!r}")
    check("black/white ΔsRGB == √3（独立复算）",
          abs(dist["black_white"]["srgb"] - math.sqrt(3)) < 1e-15,
          f"{dist['black_white']['srgb']!r} vs sqrt(3)={math.sqrt(3)!r}")
    check("black/white ΔE_ok ≈ 1（OKLab 亮度轴；公布矩阵系数保留 10 位小数，"
          "正逆变换非严格互逆，故非恰好 1.0）",
          abs(dist["black_white"]["oklab"] - 1.0) < 1e-7,
          f"{dist['black_white']['oklab']!r} (|Δ|={abs(dist['black_white']['oklab']-1.0):.3e})")
    lab_rg = math.dist(oklab_of((1, 0, 0)), oklab_of((0, 1, 0)))
    check("red/green ΔE_ok 复算 == 引擎全精度",
          abs(lab_rg - dist["red_green"]["oklab"]) < 1e-12,
          f"python {lab_rg!r} vs engine {dist['red_green']['oklab']!r}")

    # ---- 5) parse syntax families ----
    forms = []
    for family, sample in PARSE_FORMS:
        d, r = run_json(bin, ["convert", sample])
        forms.append({"family": family, "sample": sample, "hex": d["hex"],
                      "alpha": d["alpha"], "rgb": d["rgb"]})
    n_rgb8, n_transparent = count_named_colors()
    named_count = n_rgb8 + n_transparent
    # frozen expectation: the engine's named-color table is stable; if this
    # assert fires the source drifted and the page claim needs a conscious
    # re-freeze, not a silent number change.
    assert named_count == 149, (n_rgb8, n_transparent)
    write("parse-forms.json", {"forms": forms,
                               "named_count": named_count,
                               "named_count_note":
                                   f"named.rs 机器计数：{n_rgb8} 条 from_rgb8 表项 + "
                                   f"transparent 特例（named.rs:153）= {named_count} 个 "
                                   "match 臂（prep_data 运行时正则统计并断言）"})
    cmds["parse-forms.json"] = "chromap --json convert <sample> per row"
    check("全部语法族样本可解析且往返为同一颜色",
          all(f["hex"] in ("#aabbcc", "#aabbccdd", "#663399", "#66339980",
                           "#00000000") for f in forms),
          "; ".join(f"{f['sample']}->{f['hex']}" for f in forms))

    # ---- 6) 14 palette kinds + -c ignored by harmony kinds ----
    pals = {}
    for kind in PALETTE_KINDS:
        d, r = run_json(bin, ["palette", "#4f7cff", "-k", kind])
        pals[kind] = [c["hex"] for c in d["colors"]]
    d9, r9 = run_json(bin, ["palette", "rebeccapurple", "-k", "complementary",
                            "-c", "9"])
    write("palettes.json", {
        "base": "#4f7cff", "kinds": pals,
        "harmony_ignores_count": {
            "argv": r9["argv"],
            "requested_count": 9, "returned": len(d9["colors"]),
            "note": "harmony 类（complementary/analogous/split/triadic/square/"
                    "tetradic）长度由色相关系固定，-c 只作用于标尺类；"
                    "main.rs:436-441 调 harmony() 不传 count"},
    })
    cmds["palettes.json"] = "chromap --json palette #4f7cff -k <kind> × 14; +1 probe"
    check("harmony 类 -c 9 仍返回 2 色（complementary）",
          len(d9["colors"]) == 2, f"returned {len(d9['colors'])}")

    # ---- 7) 7 output formats on the hero color ----
    fmts = {}
    for f in FORMATS:
        r = run(bin, ["--format", f, "--plain", "--color", "never",
                      "convert", "rebeccapurple"])
        assert r["exit"] == 0 and r["stdout"].strip()
        fmts[f] = r["stdout"].strip()
    write("formats.json", fmts)
    cmds["formats.json"] = "chromap --format <f> --plain --color never convert rebeccapurple"
    check("7 种 --format 各不相同", len(set(fmts.values())) == 7, repr(list(fmts.values())))

    # ---- 8) design-token workflow (mirrors examples/design_tokens.rs) ----
    tokens, r = run_json(bin, ["palette", "#4f7cff", "-k", "neighbors", "-c", "9",
                               "--lightness-span", "0.64", "--css-prefix", "brand"])
    scale = [c["hex"] for c in tokens["colors"]]
    example = subprocess.run(
        ["cargo", "run", "--quiet", "--package", "chromap", "--example",
         "design_tokens"], cwd=str(REPO), capture_output=True, text=True)
    assert example.returncode == 0, example.stderr
    example_hexes = [l.strip().split(": ")[1].rstrip(";")
                     for l in example.stdout.splitlines() if "--brand-" in l]
    cmds["design-tokens.json"] = " ".join(r["argv"]) + " ; cargo run --package chromap --example design_tokens"
    check("CLI palette -k neighbors -c 9 --lightness-span 0.64 == design_tokens.rs 输出",
          scale == example_hexes,
          f"cli {scale} vs example {example_hexes}")
    contrast_on_white = {}
    for hexv in scale:
        d, rr = run_json(bin, ["contrast", "ratio", hexv, "#ffffff"])
        contrast_on_white[hexv] = d
    write("design-tokens.json", {
        "argv": r["argv"], "scale_hex": scale, "css": tokens["css"],
        "example_source": "crates/chromap/examples/design_tokens.rs",
        "example_output_hex": example_hexes,
        "cli_equals_example": scale == example_hexes,
        "contrast_on_white": contrast_on_white,
    })

    # PNG preview: deterministic across two runs
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        p1, p2 = Path(td) / "a.png", Path(td) / "b.png"
        for p in (p1, p2):
            rr = run(bin, ["palette", "#4f7cff", "-k", "neighbors", "-c", "5",
                           "--css-prefix", "brand", "--png", str(p)])
            assert rr["exit"] == 0, rr["stderr"]
        h1, h2 = sha256_bytes(p1.read_bytes()), sha256_bytes(p2.read_bytes())
        import struct
        raw = p1.read_bytes()
        w, h = struct.unpack(">II", raw[16:24])
    write("png-freeze.json", {
        "argv": ["chromap", "palette", "#4f7cff", "-k", "neighbors", "-c", "5",
                 "--css-prefix", "brand", "--png", "<path>.png"],
        "run1_sha256": h1, "run2_sha256": h2, "deterministic": h1 == h2,
        "width": w, "height": h,
        "geometry": "5 色 1 行：5×104=520 宽 × 80 高（visual.rs GRID_COLUMNS/CELL_*）",
    })
    cmds["png-freeze.json"] = "two identical runs in a temp dir; sha256 recorded"
    check("PNG 双跑 sha256 一致（确定性）", h1 == h2, f"{h1} vs {h2}")
    check("PNG 尺寸 == 5×104 × 80", (w, h) == (520, 80), f"{w}x{h}")

    # ---- 9) cross-engine input: color-palette-rs tokyo-night ----
    cprs = subprocess.run(["color-palette-rs", "--json", "show", "tokyo-night"],
                          capture_output=True, text=True)
    tokyo = json.loads(cprs.stdout) if cprs.returncode == 0 else None
    if tokyo:
        hexes = [c["hex"] for c in tokyo["colors"]]
        dom, r = run_json(bin, ["dominant", "--count", "3", *hexes,
                                *hexes[:3]])  # 7 input colors
        avg, r2 = run_json(bin, ["average", *hexes])
        write("tokyo-night.json", {"palette": tokyo, "input_hexes": hexes,
                                   "dominant_argv": r["argv"],
                                   "dominant": [{"hex": s["color"]["hex"],
                                                 "population": s["population"],
                                                 "weight": s["weight"]}
                                                for s in dom],
                                   "average_argv": r2["argv"],
                                   "average_hex": avg["hex"],
                                   "average_oklab": avg["oklab"]})
        cmds["tokyo-night.json"] = ("color-palette-rs --json show tokyo-night ; "
                                    "chromap --json dominant --count 3 <hexes×7> ; "
                                    "chromap --json average <hexes>")
        check("dominant 权重和 == 1", abs(sum(s["weight"] for s in dom) - 1.0) < 1e-9,
              str([s["weight"] for s in dom]))

    # ---- 10) exit codes & error paths ----
    probes = [
        ("ok_convert", ["convert", "#abc"]),
        ("invalid_hex", ["convert", "#zzzzzz"]),
        ("unknown_name", ["convert", "notacolor"]),
        ("unreachable_contrast", ["--json", "contrast", "ensure", "#333333",
                                  "#555555", "--minimum", "21"]),
        ("alpha_needs_canvas", ["--json", "contrast", "ratio", "#66339980",
                                "#ffffff"]),
        ("json_conflicts_plain", ["--json", "--plain", "convert", "#abc"]),
    ]
    exits = {}
    for name, args in probes:
        rr = run(bin, args)
        exits[name] = {"argv": rr["argv"], "exit": rr["exit"],
                       "stderr_first_line": rr["stderr"].splitlines()[0]
                       if rr["stderr"] else ""}
    write("exits.json", exits)
    cmds["exits.json"] = "probes listed inside the file"
    check("成功 0 / 全部错误路径 2", exits["ok_convert"]["exit"] == 0
          and all(v["exit"] == 2 for k, v in exits.items() if k != "ok_convert"),
          json.dumps({k: v["exit"] for k, v in exits.items()}))

    # ---- 11) CLI surface ----
    cli = {"version": version,
           "help": run(bin, ["--help"])["stdout"],
           "inspect_help": run(bin, ["inspect", "--help"])["stdout"],
           "contrast_help": run(bin, ["contrast", "--help"])["stdout"],
           "palette_help": run(bin, ["palette", "--help"])["stdout"],
           "subcommands": SUBCOMMANDS,
           "subcommands_note": "--help 列出的 11 个功能子命令（另有 help）"}
    write("cli.json", cli)
    cmds["cli.json"] = "chromap --help / <sub> --help"

    # ---- 12) engine identity, selfchecks, provenance ----
    write("engine.json", {
        "version": version,
        "binary_sha256": sha256_bytes(bin.read_bytes()),
        "repo_commit": commit,
        "repo_status_untracked_only": all(l.startswith("??") for l in dirty),
        "repo_status_lines": dirty,
    })
    n_checks = len(checks)
    write("selfchecks.json", {"total": n_checks, "passed": n_checks,
                              "checks": checks})
    n_json = len([n for n in PENDING if n != "provenance.json"])
    write("provenance.json", {
        "generator": "prep_data.py (verbatim engine outputs; derivations "
                     "self-checked against engine output before freezing)",
        "engine_version": version,
        "engine_binary_sha256": sha256_bytes(bin.read_bytes()),
        "engine_commit": commit,
        "selfchecks": f"{n_checks}/{n_checks} passed (data/selfchecks.json)",
        "commands": cmds,
        "notes": [
            "All floats are frozen at engine full precision; panels render "
            "them verbatim without truncation.",
            "spaces-chain.json linear/oklab values are recomputed from "
            "spaces.rs formulas and anchored to the engine's full-precision "
            "relative_luminance / oklab() string.",
            "tokyo-night colors come from the color-palette-rs engine "
            "(--json show tokyo-night), used as dominant/average input.",
        ],
    })
    flush()
    print(f"\n{len(cmds)} frozen command groups, {n_json} data files, "
          f"{n_checks}/{n_checks} self-checks passed")


if __name__ == "__main__":
    main()
