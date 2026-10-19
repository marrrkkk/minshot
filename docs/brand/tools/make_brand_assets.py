"""Generate minshot brand assets as outlined SVG paths.

The wordmark is set in Inter (SIL Open Font License), weight 600, optical size 14,
then converted to paths so no fonts are needed to use the files.
Run: py -3 docs/brand/tools/make_brand_assets.py <Inter-Variable.ttf> docs/brand/kit
"""
import math
import re
import sys
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

FONT_PATH, OUT = sys.argv[1], Path(sys.argv[2])
TEXT, WEIGHT, OPSZ = "minshot", 600, 14
F = 100.0            # wordmark master font size, in SVG units
TRACKING = -0.02     # em
INK, PAPER = "#18181b", "#fafafa"

font = instantiateVariableFont(TTFont(FONT_PATH), {"wght": WEIGHT, "opsz": OPSZ})
upm = font["head"].unitsPerEm
cmap, glyphset, hmtx = font.getBestCmap(), font.getGlyphSet(), font["hmtx"]
scale = F / upm


def num(n):
    return f"{n:.2f}".rstrip("0").rstrip(".")


def cleanup(d):
    return re.sub(r"-?\d+\.\d+", lambda m: num(float(m.group())), d)


# --- wordmark path in text space: baseline y=0, first glyph starts at x=0 ---
x, d = 0.0, ""
minx = miny = 1e9
maxx = maxy = -1e9
for ch in TEXT:
    gname = cmap[ord(ch)]
    sp = SVGPathPen(glyphset)
    glyphset[gname].draw(TransformPen(sp, (scale, 0, 0, -scale, x, 0)))
    d += sp.getCommands() + " "
    bp = BoundsPen(glyphset)
    glyphset[gname].draw(TransformPen(bp, (scale, 0, 0, -scale, x, 0)))
    x0, y0, x1, y1 = bp.bounds
    minx, miny = min(minx, x0), min(miny, y0)
    maxx, maxy = max(maxx, x1), max(maxy, y1)
    x += hmtx[gname][0] * scale + TRACKING * F
width = x - TRACKING * F
d = cleanup(d)

SYMBOL = "M16 40 H240 V216 H204 V76 H146 V216 H110 V76 H52 V216 H16 Z"


def svg(w, h, body, title):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {num(w)} {num(h)}" '
        f'width="{num(w)}" height="{num(h)}" role="img" aria-labelledby="title">\n'
        f'  <title id="title">{title}</title>\n{body}</svg>\n'
    )


def write(name, w, h, body, title):
    (OUT / name).write_text(svg(w, h, body, title), encoding="utf-8")


def wordmark_group(font_size, tx, ty, fill):
    s = font_size / F
    return (
        f'    <g transform="translate({num(tx)} {num(ty)}) scale({num(s)})">\n'
        f'      <path fill="{fill}" d="{d}"/>\n'
        f'    </g>\n'
    )


OUT.mkdir(parents=True, exist_ok=True)
W, H = maxx - minx, maxy - miny

# 1. wordmark alone (tight box, 2-unit breathing room)
pad = 4
write("minshot-wordmark-black.svg", math.ceil(W + 2 * pad), math.ceil(H + 2 * pad),
      f'  <g transform="translate({num(pad - minx)} {num(pad - miny)})">\n'
      f'    <path fill="{INK}" d="{d}"/>\n  </g>\n', "minshot wordmark")
write("minshot-wordmark-white.svg", math.ceil(W + 2 * pad), math.ceil(H + 2 * pad),
      f'  <g transform="translate({num(pad - minx)} {num(pad - miny)})">\n'
      f'    <path fill="{PAPER}" d="{d}"/>\n  </g>\n', "minshot wordmark, reversed")

# 2. symbol, both treatments
write("minshot-symbol-black.svg", 256, 256,
      f'  <path fill="{INK}" d="{SYMBOL}"/>\n', "minshot symbol")
write("minshot-symbol-white.svg", 256, 256,
      f'  <path fill="{PAPER}" d="{SYMBOL}"/>\n', "minshot symbol, reversed")

# 3. horizontal lockup: symbol 88 tall (visual), wordmark at 88, baseline 158
LM, LT, LF, LBASE, LGAP = 0.5, 64, 88, 158, 200
LW = math.ceil(LGAP + maxx * (LF / F) + 48)
lock = (
    f'  <g id="symbol" transform="translate(40 {LT}) scale({num(LM)})">\n'
    f'    <path fill="{{fill}}" d="{SYMBOL}"/>\n  </g>\n'
    f'  <g id="wordmark">\n{wordmark_group(LF, LGAP, LBASE, "{fill}")}  </g>\n'
)
write("minshot-lockup-horizontal-black.svg", LW, 256, lock.format(fill=INK), "minshot lockup")
write("minshot-lockup-horizontal-white.svg", LW, 256, lock.format(fill=PAPER), "minshot lockup, reversed")

# 4. stacked lockup: symbol above the wordmark, both centred
LF2 = 56
vis_w = W * (LF2 / F)
SW = round(max(112, width * (LF2 / F)) + 96)
SYM_X = (SW - 112) / 2
WM_X = (SW - vis_w) / 2 - minx * (LF2 / F)
stack = (
    f'  <g id="symbol" transform="translate({num(SYM_X)} 30) scale({num(LM)})">\n'
    f'    <path fill="{{fill}}" d="{SYMBOL}"/>\n  </g>\n'
    f'  <g id="wordmark">\n{wordmark_group(LF2, WM_X, 206, "{fill}")}  </g>\n'
)
write("minshot-lockup-stacked-black.svg", SW, 256, stack.format(fill=INK), "minshot lockup, stacked")
write("minshot-lockup-stacked-white.svg", SW, 256, stack.format(fill=PAPER), "minshot lockup, stacked, reversed")

# 5. app icon: white m on the brand tile (22% corner radius)
s = 0.64
gw, gh = 224 * s, 176 * s
gx, gy = (256 - gw) / 2 - 16 * s, (256 - gh) / 2 - 40 * s
icon = (
    f'  <rect width="256" height="256" rx="56" fill="{INK}"/>\n'
    f'  <g transform="translate({num(gx)} {num(gy)}) scale({num(s)})">\n'
    f'    <path fill="{PAPER}" d="{SYMBOL}"/>\n  </g>\n'
)
write("minshot-icon.svg", 256, 256, icon, "minshot app icon")
write("minshot-icon-mono.svg", 256, 256,
      icon.replace(INK, "#000000").replace(PAPER, "#ffffff"), "minshot app icon, one colour")

print(f"wordmark: {W:.1f} x {H:.1f}; horizontal lockup {LW:.1f} x 256; stacked {SW} x 256; base glyph LSB {minx:.2f}")
