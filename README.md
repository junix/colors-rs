# chromap

`chromap` 是一个 Rust 颜色工具 workspace：核心库负责颜色模型、转换、感知式调色、配色、透明合成和可访问性；独立 CLI 用于脚本、设计 token 与批处理。

核心设计是把两个常被混淆的概念分开：

- **互补色 / 色彩和谐**：色相环关系，使用 `harmony`、`analogous_scale`；
- **可读前景色**：按相对亮度与对比度实际测量，使用 `best_foreground`、`best_black_or_white`、`ensure_contrast`。

互补色不等于可读色。浅黄背景的色相互补色仍可能不适合作为小字号文本。

## 能力

| 类别 | 能力 |
|---|---|
| 解析 | `#rgb`、`#rgba`、`#rrggbb`、`#rrggbbaa`、`0x...`、`rgb()`、`rgba()`、`hsl()`、`hsla()`、CSS 命名色、`transparent` |
| 格式化 | Hex、CSS RGB/HSL、HSV、CMYK、OKLab、OKLCH |
| 转换 | gamma sRGB、linear sRGB、HSL、HSV、CMYK、OKLab、OKLCH |
| 调节 | OKLCH 明度/色度/色相、HSL 调节、alpha、灰度化、反色 |
| 混色 | sRGB、linear sRGB、OKLab、OKLCH；四种 hue path |
| 配色 | 渐变、相邻明度、相邻色相、tints、shades、tones、六种 harmony、黄金角 |
| 可访问性 | relative luminance、ratio、AA/AAA、候选色、黑白色、自动修复 |
| 图片基础 | alpha 合成、linear-light 平均色、确定性 OKLab 主色聚类 |
| CLI | ANSI True Color/256 色块、稳定文本、JSON、PNG、CSS custom properties |

## 构建

```bash
cargo build --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo install --path crates/chromap-cli
```

也可以从 npm 安装 CLI：

```bash
npm install --global chromap
chromap --version

# 或者不做全局安装
npx chromap inspect rebeccapurple
```

npm 包会携带完整的 Rust workspace 源码、测试和 examples。第一次执行命令时会在用户缓存目录构建原生二进制，因此需要 Node.js 18+、Rust 1.81+ 和 Cargo；后续执行会直接复用已构建的二进制。可用 `CHROMAP_CACHE_DIR` 自定义缓存位置。

## Library 示例

```rust
use chromap::{parse_color, to_hex, HexFormat};

fn main() -> Result<(), chromap::ColorError> {
    let base = parse_color("hsl(220 100% 65%)")?;
    let output = base
        .adjust_lightness(-0.08)?
        .adjust_saturation(0.20)?
        .rotate_hue(12.0)?;

    println!("{}", to_hex(output, HexFormat::Auto));
    Ok(())
}
```

`adjust_lightness(0.1)` 给 OKLCH lightness 增加 0.1；`adjust_saturation(0.25)` 将 chroma 乘以 1.25；`-1` 去掉全部 chroma。需要严格 HSL 语义时使用 `adjust_hsl_lightness` / `adjust_hsl_saturation`。

### 相邻渐变色

```rust
use chromap::{neighboring_lightness_scale, parse_color};

fn main() -> Result<(), chromap::ColorError> {
    let brand = parse_color("#4f7cff")?;
    for color in neighboring_lightness_scale(brand, 9, 0.64)? {
        println!("{color}");
    }
    Ok(())
}
```

### 选择可读字体颜色

```rust
use chromap::{best_foreground, parse_color, Color};

fn main() -> Result<(), chromap::ColorError> {
    let background = parse_color("#fff2a8")?;
    let candidates = [Color::WHITE, Color::BLACK, parse_color("#332b00")?];
    let choice = best_foreground(background, &candidates, 4.5)?;
    println!("{}  {:.3}:1", choice.color, choice.ratio);
    Ok(())
}
```

### 自动修复现有前景色

```rust
use chromap::{ensure_contrast, parse_color};

fn main() -> Result<(), chromap::ColorError> {
    let fixed = ensure_contrast(
        parse_color("white")?,
        parse_color("#fff2a8")?,
        4.5,
    )?;
    assert!(fixed.ratio >= 4.5);
    println!("{}", fixed.color);
    Ok(())
}
```

修复器同时搜索更亮和更暗的 OKLCH lightness，选择 OKLab 变化较小的解。不可达目标返回 `UnreachableContrast`。

## CLI 示例

```bash
chromap inspect 'rgb(79 124 255 / 80%)'
chromap --json inspect rebeccapurple

chromap --format oklch convert '#4f7cff'
chromap adjust '#4f7cff' --lightness 0.08 --saturation 0.2 --hue 15

chromap mix '#ff4d6d' '#4f7cff' --weight 0.35 --space oklab
chromap gradient '#ff4d6d' '#4f7cff' --steps 9 --space oklch

chromap palette '#4f7cff' --kind neighbors --count 9 --lightness-span 0.64
chromap palette '#4f7cff' --kind split-complementary --css-prefix accent

chromap contrast ratio white '#4f7cff'
chromap contrast pick '#fff2a8' white black '#332b00' --minimum 4.5
chromap contrast black-white '#fff2a8'
chromap contrast ensure white '#fff2a8' --target aa

chromap composite '#ff000080' white --space srgb
chromap average black white
chromap dominant --count 2 red red red blue
chromap distance '#4f7cff' '#587ff4'
```

### 终端色块与 PNG

交互式终端默认在颜色文本前显示 ANSI 色块；True Color 不可用时自动映射到 ANSI 256 色。输出被 pipe、设置了 `NO_COLOR`，或使用 `--plain` / `--json` 时不会混入 ANSI 转义序列。

```bash
# TTY 中自动显示色块
chromap palette '#4f7cff' --kind hue-wheel --count 8

# 强制或禁用 ANSI 色块
chromap --color always convert '#4f7cff'
chromap --color never palette '#4f7cff'

# 面向 shell 脚本的稳定纯文本
chromap --plain gradient red blue --steps 7

# 写入棋盘底 PNG；已存在文件默认拒绝覆盖
chromap palette '#4f7cff80' --count 9 --png palette.png
chromap palette '#4f7cff80' --count 9 --png palette.png --force

# 只验证 PNG 编码与目标路径，不写盘
chromap --json gradient red blue --png gradient.png --dry-run

# `-` 表示只把 PNG 二进制写到 stdout
chromap gradient red blue --png - > gradient.png
```

半透明颜色的终端预览会并排显示深色与浅色背景下的结果；PNG 使用棋盘背景表达 alpha。PNG 一次最多渲染 256 色，诊断与写入摘要走 stderr，`--json` 的 stdout 始终保持合法 JSON。

## 边界

1. canonical storage 是 normalized gamma-encoded sRGBA，不是 HDR/Display-P3。
2. parser 是实用 CSS 子集，不实现 `var()`、`calc()`、`color-mix()`、Display-P3 或完整浏览器 grammar。
3. formatter 可输出 OKLab/OKLCH；程序化输入使用结构体。
4. CMYK 是无 ICC profile 的数学近似。
5. OKLab distance 不宣称是 CIEDE2000。
6. 透明色对比度必须提供实际 canvas。
7. 当前不实现 APCA、ICC、HDR、光谱/颜料混色。

## 验证

仓库包含 library/CLI 集成测试以及 rustfmt、Clippy、test、rustdoc、npm pack CI。CLI 测试覆盖 ANSI True Color/256 色降级、plain/JSON 无装饰契约、PNG 文件/stdout、dry-run 与覆盖保护。

## License

MIT OR Apache-2.0.
