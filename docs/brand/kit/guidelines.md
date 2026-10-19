# minshot — visual identity

The mark is a squared lowercase **m** built from three stems and one cap bar. It reads as
"minshot" (the shortest written form of *minutes*) and survives 16 px. Monochrome by design —
the product is a minimal terminal tool, and the category is saturated with blue.

## Files (`docs/brand/kit/`)

| File | Use |
| --- | --- |
| `minshot-symbol-black.svg` / `-white.svg` | The m mark, tight master (256×256, mark 224×176). Black on light, white on dark. |
| `minshot-wordmark-black.svg` / `-white.svg` | "minshot" set in Inter SemiBold, outlined to paths. |
| `minshot-lockup-horizontal-black.svg` / `-white.svg` | Mark + wordmark (579×256). Default lockup. |
| `minshot-lockup-stacked-black.svg` / `-white.svg` | Stacked lockup (308×256) for square-ish spaces. |
| `minshot-icon.svg` | App icon master: white m on the brand tile (22 % corner radius). |
| `minshot-icon-mono.svg` | One-colour app icon (pure black / pure white). |
| `app-icon-1024.png` | 1024 px render — input for `npm run tauri -- icon`. |
| `web/` | `favicon.ico`, `favicon.svg`, `favicon-16/32/48.png`, `apple-touch-icon.png`, `icon-192/512.png`, `maskable-512.png`, `site.webmanifest`, `head-snippet.html` + standard `-black` / `-white` / `-square` / `-favicon` / `-app-icon` variants. |

## Colour

| Token | Hex | Notes |
| --- | --- | --- |
| Brand black | `#18181b` | Symbol, wordmark, icon tile. |
| Paper | `#fafafa` | Reversed symbol/wordmark; the m on the tile. |
| Pure black / white | `#000000` / `#ffffff` | One-colour print only (`minshot-icon-mono.svg`). |

Pair contrast: `#fafafa` on `#18181b` = 17.7:1 (AAA). No accent colour — the app's green
status dot (`#4ade80`) is UI, not brand. Every master is already one-colour, so greyscale,
fax and engraving versions are the black or white files with no redraw.

## Clear space and minimum sizes

- **Clear space:** keep at least one stem width free on all sides = **20 % of the mark's
  height** (one stem is 36/176 of the master's mark).
- **Minimum sizes:** symbol and icon 16 px (4 mm print); horizontal lockup 120 px wide
  (32 mm); stacked lockup 96 px wide (26 mm). Below that, use the symbol alone.

## Typography

The wordmark is **Inter SemiBold (600)**, optical size 14, tracking **−2 %**, converted to
outlines — Inter is under the SIL Open Font License, so the outlined files ship freely.
Never re-typeset the wordmark from live text in a logo context. UI copy keeps the app's
existing Inter/system stack; the app header already uses the inline symbol.

## Don't

- Don't stretch, rotate, skew, outline, shadow or otherwise restyle the mark or wordmark.
- Don't recolour outside the palette (black, paper, pure black/white).
- Don't re-typeset or re-kern the wordmark.
- Don't put the black mark on mid-grey — switch to the white file.
- Don't crop the mark, or rebuild the tile with a different corner radius (56/256 = 22 %).

## Wired into the app

- `index.html` — header tile (inline SVG, `currentColor`) + `public/favicon.svg`.
- `src-tauri/icons/` — full Tauri icon set regenerated from `app-icon-1024.png`;
  `source.png` is kept in sync with that master.

## Regeneration

```
py -3 docs/brand/tools/make_brand_assets.py <Inter-Variable.ttf> docs/brand/kit
py -3 <logo-design>/scripts/export_variants.py docs/brand/kit/minshot-symbol-black.svg ^
  --name minshot --icon-bg "#18181b" --icon-scale 0.64 ^
  --favicon-source docs/brand/kit/minshot-icon.svg --web-icons --out-dir docs/brand/kit/web
npm run tauri -- icon docs/brand/kit/app-icon-1024.png
```

Font: Inter v4 variable — <https://rsms.me/inter> (SIL OFL 1.1).

## Known limits

- No trademark clearance was performed.
- The Inter TTF is not committed; only outlines. Re-download it to regenerate the wordmark.
