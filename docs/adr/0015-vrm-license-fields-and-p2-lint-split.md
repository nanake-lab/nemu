# ADR 0015：VRM 许可字段，以及 P2-05 与 P2-06 的 lint 验收划分

- 状态：提议。Claude 起草的推荐做法，用户合并本 ADR 即采纳。
- 日期：2026-10-21
- 关联：CONTRACTS.md 第 1.5 节；任务卡 P2-03（PR #43 审查意见第 2、4 条）、P2-05、P2-06；VRM 1.0 规范 `VRMC_vrm` 的 meta

## 背景

- **许可字段的冲突：** CONTRACTS 第 1.5 节写「`licenseUrl` 指向内容包的许可文件」。VRM 1.0 规范规定 `licenseUrl` 必填，而且必须是指向 VRM 公共许可的唯一地址，VRM 1.0 只接受 `https://vrm.dev/licenses/1.0/`；其他许可写在 `otherLicenseUrl`。管线锁定的 VRM 扩展 4.10.2 按规范固定导出这个地址。
- **冲突的后果：**
  - P2-03 的管线把内容包许可写进 `otherLicenseUrl`，同时要求导出的 `licenseUrl` 与配置相同。正式内容一旦使用自己的许可地址，就无法导出（PR #43 审查意见第 2 条）。
  - asset-lint 的 A1.5 只检查 `licenseUrl` 不为空。
- **lint 验收的划分：** P2-05 的验收写「Nemu 中期版本的 `asset-lint` 0 错误」。P2-02 交付的 VRM 现有 6 个错误：
  - 4 个属于表情（A1.3：三个 overrideBlink、缺少 8 个自定义表情），由 P2-06 处理；
  - 2 个属于弹簧骨碰撞体（A1.4）。

  P2-05 单独完成时达不到 0 错误。卡片第 6 步本来也只要求猫耳、头发、尾巴相关的规则通过。

## 决定

### 1. `licenseUrl` 与 `otherLicenseUrl` 按 VRM 1.0 的字段含义使用

- **`licenseUrl`：** 固定为 `https://vrm.dev/licenses/1.0/`。不等于这个地址时报错（E）。
- **`otherLicenseUrl`：** 写内容包自己的许可。缺少或为空时报错（E）。`build.yaml` 的 `meta.contentLicenseUrl` 写入这个字段。
- **地址写法：** 内容包目前不公开，先写相对内容包根目录的 `LICENSE.md`。以后如果对外分发 Nemu 的模型，在 P7 改成公开地址。
- **使用权限字段：** VRM 公共许可里的 avatarPermission、commercialUsage、allowRedistribution、modification 等字段，保留用户在 VRoid 导出时的设置，管线不放宽；没有设置的取规范默认值，也就是最严格的一档。Nemu 内容的具体许可条款由用户在 P7 之前决定。
- **契约文本：** CONTRACTS.md 第 1.5 节按「只增不改」追加一句澄清，原句保留。
- **代码改动放在 P2-05，** 因为它是第一张导出正式内容的卡：
  - `tools/asset-pipeline/export_vrm.py` 的导出后检查改为：`licenseUrl` 等于 VRM 公共许可地址，`otherLicenseUrl` 等于 `contentLicenseUrl`；
  - asset-lint 的 A1.5 按上面两条报错，修复建议同步修改；
  - 测试夹具和 `build.example.yaml` 随之更新，并各补一条负向测试。

### 2. P2-05 与 P2-06 的 lint 验收划分

- **P2-05：** 在 Nemu 中期版本上，A1.4（弹簧骨与碰撞体）和 A1.5（元数据与清单）的错误为 0。A1.3（表情）的错误留给 P2-06。
- **P2-06：** 验收不变，asset-lint 全部规则 0 错误。
- **临时小节：** P2-05 运行完整管线时，`build.yaml` 里属于其他卡的小节（如 expressions、materials）先用不改变源模型的最小配置，放进 `content/_candidates/` 并标为临时，不替用户决定审美参数。P2-06 再把它们换成用户选定的参数。

## 备选方案

- **让管线绕过扩展，强行把 `licenseUrl` 写成内容包许可：** 不采纳。这违反 VRM 1.0 规范，其他 VRM 软件可能拒读或误读。
- **取消 `licenseUrl` 必填：** 不采纳，规范要求必填。
- **P2-05 等 P2-06 做完一起验收：** 不采纳。两张卡由不同的用户选择驱动，绑在一起会互相等待。

## 后果

- **契约：** CONTRACTS 第 1.5 节追加澄清。不涉及进程间消息，版本号不变。
- **任务卡：** P2-05、P2-06 追加说明，`tasks/cards.json` 不变。
- **差异登记：** 不涉及 Echo Core 规格，不登记。
