# P2-06 Blender preflight (2026-10-23)

Technical check only. The standard material / emo A profile are unselected test fixtures. No tail/springs step, final export, promotion, or aesthetic approval is implied.

The three official `asset:step` invocations and their source/output paths are recorded in `01-import.log`, `05-expressions.log`, `06-materials.log`. `resolved-build.json` is the exact normalized configuration. It can be parsed by the existing YAML config loader. Input candidate SHA256 and source preservation are recorded in `input.json`.

Each stage ran as a separate `blender -b --python` process through the pipeline scheduler with no competing Blender, Electron or Runner.Worker. The initial seed was the existing P2-03 cached sample import; `import_vrm.py` replaces its scene objects. The material readback additionally verifies that every configured material is actually used by imported scene meshes, rather than merely existing as an orphan datablock.

After material processing, the main agent acquired the same `tools/asset-pipeline/.cache/pipeline.lock` with exclusive CreateNew semantics and ran:

```powershell
& $env:NEMU_BLENDER -b tools/asset-pipeline/.cache/P2-06-preflight/06-materials.blend --python-exit-code 1 --python tools/asset-pipeline/expressions/inspect_blender.py -- --config reports/P2-06/pipeline-preflight/resolved-build.json --report reports/P2-06/pipeline-preflight/readback.json
```

The lock was released in `finally`; the inspector is read-only and does not save the scene. It verifies all configured morph/color/texture-transform binds, preset overrides, material use and MToon values. Numeric values must match within float-storage precision (1e-6 relative / 1e-7 absolute); names, counts and enums match exactly. Readback and new Python static checks passed. Full asset-lint results from existing preview models remain in earlier evidence; this preflight does not replace the required complete production build after user selection and P2-05 exporter integration.
