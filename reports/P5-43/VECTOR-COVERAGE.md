# P5-43 黄金向量覆盖

基线取自 main 的 `e79dce8853b76ae6731bba74bb76a3e36016ddce`。14 个 IDL 文件逐字节相同。四组 main 向量与当前向量的对应前缀逐条相等，基线清单的计数及 SHA-256 均一致。

| 集合 | main | 当前 | 追加 |
| --- | ---: | ---: | ---: |
| illegal | 1073 | 1161 | 88 |
| ingress | 63 | 63 | 0 |
| legal | 779 | 856 | 77 |
| unicode | 18 | 18 | 0 |

| 消息 | 新增合法 | 新增非法 |
| --- | ---: | ---: |
| ActV2 | 3 | 3 |
| AttributeRule | 7 | 4 |
| AttributeRuleTable | 3 | 4 |
| ClaimV2 | 3 | 4 |
| ConceptLink | 3 | 3 |
| ConceptRegistry | 3 | 4 |
| ConceptRevision | 3 | 4 |
| ConceptRole | 3 | 3 |
| EventAttributesRecordV2 | 4 | 7 |
| RefV2 | 3 | 3 |
| ScopedVerification | 3 | 4 |
| SemanticGraphV2 | 3 | 4 |
| SourceField | 4 | 3 |
| SourceLocation | 5 | 4 |
| SourceMetadataV2 | 4 | 5 |
| SourceTransform | 3 | 3 |
| StageAdoption | 5 | 6 |
| StageCoverage | 3 | 4 |
| TargetPremise | 3 | 4 |
| TextAdapterEntry | 3 | 4 |
| TextAdapterTable | 3 | 4 |
| VerificationRequirement | 3 | 4 |
