# P2-02 VRoid 角色源文件 — 报告

## 结果
用户验收通过。交付物已推送到私有 `nemu-content` 仓库；主仓库通过本卡 PR 更新子模块指针，等待 CI 与用户批准。

## 交付物
- `content/nemu/source/nemu.vroid`：用户验收通过的最终 VRoid 源文件；未提交中间版本。
- `content/nemu/source/nemu-vroid-export.vrm`：对应 VRM 导出文件；本卡未修改 VRM 内容。
- `content/nemu/source/progress/正面图.png`：过程截图。
- BOOTH 素材：用户确认未使用，因此不需要 `content/nemu/source/licenses/`。
- 私有内容仓库提交：`513df15175673c1ac06b57d485f7986562e0aa31`（`feat(P2-02): add accepted VRoid assets`）。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 用户验收 | 通过 | 用户明确确认 P2-02 已验收通过。 |
| 交付范围与仓库提交 | 通过 | `nemu-content` 提交仅含最终 `.vroid`、导出 `.vrm` 和过程截图；未提交中间版本。 |
| Git LFS 规则与远端对象 | 通过 | `content/.gitattributes` 已有 `*.vroid`、`*.vrm`、`*.png` LFS 规则；远端提交中的三个文件均为 LFS 指针，且 `git lfs fetch --refetch origin main` 成功取回对象。 |
| 子模块指针与主仓库 PR | 通过 | [PR #26](https://github.com/nanake-lab/nemu/pull/26) 已更新子模块指针并带 `needs-user` 标签；[CI run 39200983634](https://github.com/nanake-lab/nemu/actions/runs/39200983634) 中 Ubuntu 与 Windows 检查均通过。 |
| `pnpm asset:lint content/nemu/source/nemu-vroid-export.vrm --json` | 结果作为后续卡输入 | 原始完整输出见下方。该 lint 结果交给 P2-03、P2-05、P2-06；不是 P2-02 的验收门槛，本卡不修复其中问题。 |

## 新增依赖
无。

## 待用户确认
- [ ] （阻塞）请用户批准 PR #26（`needs-user`）。

## 后续建议（不在本任务范围内）
- P2-03 写出「VRoid 导出设置」后，使用本报告交付的 `.vroid` 重新导出一次，并由 P2-03/P2-05/P2-06 处理下方 lint 结果。

## 下一步
等待用户批准 PR #26；批准后再按看板安排下一张卡。

## Asset lint 原始结果
以下保留 `pnpm asset:lint content/nemu/source/nemu-vroid-export.vrm --json` 的完整终端输出，作为 P2-03、P2-05、P2-06 的输入：

```text
$ node tools/asset-lint/launch.mjs content/nemu/source/nemu-vroid-export.vrm --json
{
  "file": "D:\\Dev\\nemu\\content\\nemu\\source\\nemu-vroid-export.vrm",
  "errors": 6,
  "warnings": 5,
  "diagnostics": [
    {
      "rule": "A1.1-texture-count",
      "severity": "warning",
      "message": "28 textures exceed the 16-texture target.",
      "repair": "Combine or remove textures where the material appearance is preserved."
    },
    {
      "rule": "A1.1-material-count",
      "severity": "warning",
      "message": "18 materials exceed the 12-material target.",
      "repair": "Combine compatible materials where the appearance is preserved."
    },
    {
      "rule": "A1.3-happy-override",
      "severity": "error",
      "message": "happy must set overrideBlink to \"blend\".",
      "repair": "Set overrideBlink to \"blend\"."
    },
    {
      "rule": "A1.3-relaxed-override",
      "severity": "error",
      "message": "relaxed must set overrideBlink to \"blend\".",
      "repair": "Set overrideBlink to \"blend\"."
    },
    {
      "rule": "A1.3-surprised-override",
      "severity": "error",
      "message": "surprised must set overrideBlink to \"block\".",
      "repair": "Set overrideBlink to \"block\"."
    },
    {
      "rule": "A1.3-custom-expressions",
      "severity": "error",
      "message": "Required Nemu expressions are missing: nemuSmug, nemuCatMouth, nemuSparkle, nemuBlush, nemuTeary, nemuSleepy, nemuPout, nemuAngryMark.",
      "repair": "Add the eight custom expressions listed in CONTRACTS.md 1.3."
    },
    {
      "rule": "A1.4-cat-ear-spring-bones",
      "severity": "warning",
      "message": "Cat ear spring chains could not be identified without a profile.",
      "repair": "Pass --profile nemu.profile.json so the ear bone names can be checked."
    },
    {
      "rule": "A1.4-tail-spring-bones",
      "severity": "warning",
      "message": "Tail spring chain could not be identified without a profile.",
      "repair": "Pass --profile nemu.profile.json so tail bones can be checked."
    },
    {
      "rule": "A1.4-required-colliders",
      "severity": "error",
      "message": "Required spring colliders are missing: J_Bip_C_Neck (capsule), J_Bip_C_UpperChest (capsule), J_Bip_L_UpperArm (capsule), J_Bip_R_UpperArm (capsule), J_Bip_L_LowerArm (capsule), J_Bip_R_LowerArm (capsule), J_Bip_C_Hips (capsule), J_Bip_L_UpperLeg (capsule), J_Bip_R_UpperLeg (capsule).",
      "repair": "Attach a sphere to head and capsules to neck, upperChest, both upper/lower arms, hips, and both upper legs."
    },
    {
      "rule": "A1.4-spring-colliders",
      "severity": "error",
      "message": "Spring chain Hair does not reference a collider group containing every required body collider.",
      "repair": "Create a collider group with all required colliders and reference it from each hair, ear, tail, and skirt spring."
    },
    {
      "rule": "A1.5-manifest-model",
      "severity": "warning",
      "message": "Manifest D:\\Dev\\nemu\\content\\nemu\\manifest.json has no entry for nemu-vroid-export.vrm.",
      "repair": "Add a model manifest entry with source, license, and redistributable fields."
    }
  ],
  "metrics": {
    "nodeCount": 268,
    "meshCount": 3,
    "triangleCount": 36983,
    "textureCount": 28,
    "imageCount": 28,
    "materialCount": 18,
    "estimatedTextureBytes": 205871445,
    "skinJointCount": 215,
    "maximumMeshMorphTargetCount": 57,
    "accessorBoundsHeightM": 1.5842378516542999,
    "fileBytes": 17026776
  }
}
[ELIFECYCLE] Command failed with exit code 1.
```
