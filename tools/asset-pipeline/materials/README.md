# P2-06 skin candidates and colour audit

These helpers are candidate previews. They do not promote content or run the production Blender build. Read `reports/P2-06/SKIN-COLOR-AUDIT.md` for the findings and remaining user choices.

Use the existing Python environment with PyYAML/jsonschema; never download packages for this task:

```powershell
$env:NEMU_ASSET_PYTHON = 'D:/Dev/nemu/tools/asset-pipeline/.venv/Scripts/python.exe'
& $env:NEMU_ASSET_PYTHON tools/asset-pipeline/materials/prepare_skin_candidates.py
foreach ($candidate in @('A','B','C')) {
  pnpm asset:lint "tools/asset-pipeline/.cache/P2-06-skin-lint/skin-$candidate.vrm" --profile "tools/asset-pipeline/.cache/P2-06-skin-lint/skin-$candidate.profile.json" --manifest tools/asset-pipeline/.cache/P2-06-skin-lint/manifest.json --json
  if ($LASTEXITCODE -ne 0) { throw "Candidate $candidate failed lint" }
}
node tools/asset-pipeline/materials/run-audit.mjs
node tools/asset-pipeline/materials/capture-skin.mjs
node tools/asset-pipeline/materials/verify-skin.mjs
node tools/asset-pipeline/materials/skin-gallery.mjs
```

Run browser jobs serially in a free resource window. The helpers reject existing Blender, Electron or Runner.Worker processes; they use port 5188 and close it when finished. Coordinate with P2-05 before starting; do not run during a quiet measurement.

The harness imports the production AvatarRenderer and loads the exact linted cache models through GLTFLoader/VRMLoaderPlugin. It freezes time, uses a fixed camera and stage-light intensities, and mirrors light positions for left/right comparison. It does not recolour a screenshot. `capture-skin` records loaded material values and requires two byte-identical screenshots; `verify-skin` rejects candidate images identical to their control after excluding the label region and checks runtime shade values against the declared parameters. Static sheets contain four columns: control/A/B/C.

Only the two named skin materials change. The generator asserts unchanged base-colour factors, binary/texture bytes, non-skin materials, expression/spring extensions and mesh data; SHA256 provenance is saved with the report. Candidate GLBs remain in ignored cache; selected YAML will later be applied through the production pipeline. Existing tail/springs and source files are read-only.

No runtime dependency added. Local Python checking reuses pytest/mypy/ruff in `workers/drive/.venv` and existing pipeline runtime packages through PYTHONPATH; initial missing-package attempts are retained in the report logs.
