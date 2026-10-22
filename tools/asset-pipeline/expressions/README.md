# P2-06 candidate preview tools

Appearance parameters live only in `content/_candidates/P2-06/*.yaml`.
`prepare_candidates.py` reads the immutable P2-05 short-thick-standard preview,
adds a local mouth morph and four head-weighted glyphs, and writes three preview
VRMs. It preserves the original base geometry, tail and spring configuration.
The glyphs are procedural geometry, not downloaded textures. Blush alone uses
alpha blending with vertex alpha; the other glyphs use opaque unlit materials
so VRM transparent face materials cannot sort over them.

Run from the repository root using the existing asset Python environment:

```powershell
& $env:NEMU_ASSET_PYTHON tools/asset-pipeline/expressions/prepare_candidates.py
node tools/asset-pipeline/expressions/capture.mjs
node tools/asset-pipeline/expressions/gallery.mjs
```

Node uses existing stage dependencies and Playwright. No runtime dependencies
are added. Prepare normal workspace dependencies before rendering. Check that
Blender, Electron, Runner.Worker and quiet-window measurements are idle before
rendering; serialize this with the asset pipeline. The preview freezes time,
uses fixed camera/light settings and captures each frame twice to assert exact
PNG equality. White, black and coloured gradient material views are included.
`P206_MATCH` optionally filters screenshot jobs; existing unmodified screenshot
records are retained by filename.

The GLB assembly is explicitly a **candidate preview**, not the production
Blender pipeline and not a completed P2-06 acceptance run. After user selection:
prepare the selected procedural source in cache, apply selected `expressions`
and `materials` fragments and `profile.emotionMap`, then run the complete locked
Blender pipeline, all-rule asset-lint and final stage checks. P2-05's ADR 0015
exporter change must first be available on the production task branch; do not
copy its implementation or edit its tail/springs candidates here.

The 12 emo combinations currently reference full-strength custom expressions
(C); selecting A or B for a custom expression requires regenerating and signing
those combinations again. User choices are not inferred from the gallery.
Do not promote any files to `content/nemu/` before explicit selection.
