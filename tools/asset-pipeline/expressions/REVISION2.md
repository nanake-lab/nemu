# P2-06 revision 2

All appearance values are in `content/_candidates/P2-06/revision2/expressions.candidates.yaml`. The three variants are different parameter sets at full preview weight: sleepy is fully closed in every set. Sparkle A and Blush C retain the user's previous choices. The other selected r1 fields were captured from the open Edge tab before any refresh; revised fields await a new choice.

```powershell
$env:PYTHONPYCACHEPREFIX = 'D:/Dev/cache/P2-06/pycache'
& 'D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe' tools/asset-pipeline/expressions/prepare_revision2.py
& 'D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe' tools/asset-pipeline/expressions/verify_revision2.py
node tools/asset-pipeline/expressions/capture_revision2.mjs
node tools/asset-pipeline/expressions/gallery_revision2.mjs
& 'D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe' tools/asset-pipeline/expressions/review_server.py --port 5187
```

Only run rendering in an exclusive free window; the helper checks Blender, Electron and Runner.Worker before each job. Cache/temp files for this revision live in `D:/Dev/cache/P2-06/`. Do not overwrite existing evidence when making another revision. Do not rerun the first-generation compiler over the new directory.

The lower-lip line is a surface-shading crease, not the chin outline. The YAML selects a small material/position region for normal smoothing. Base vertex positions, UVs and texture bytes remain unchanged. Angry brow candidates remove the source morph's overall displacement and apply bounded relative tilt; all sizes and bounds are explicit parameters. Sources, permissions, skin data and spring configuration are checked read-only. YAML files over 400 lines contain full schema-valid variant data; production helper code stays below 400 lines.

The local review server only binds 127.0.0.1 and only accepts same-origin, version-matching choice payloads with known fields. Submissions append JSON receipts in `content/_candidates/P2-06/review-submissions/`; existing receipts cannot be overwritten through the API. Browser drafts are saved by candidate fingerprint. `/api/p206/latest` recovers the last submitted record after restart. The footer button gives a success receipt or an explicit error. A submission records preferences/change requests; it does not promote assets or bypass the final 12-combination sign-off.

`test_review_server.py` uses temporary storage. `verify_review_page.mjs` intercepts POSTs with test responses, verifies reload/payload/error behavior and leaves user choices untouched. No runtime dependency added.
