# P5-43 黄金向量覆盖

基线取自 main 的 `e79dce8853b76ae6731bba74bb76a3e36016ddce`。14 个 IDL 文件逐字节相同。四组 main 向量与当前向量的对应前缀逐条相等，基线清单的计数及 SHA-256 均一致。

| 集合 | main | 当前 | 追加 |
| --- | ---: | ---: | ---: |
| illegal | 1073 | 1205 | 132 |
| ingress | 63 | 63 | 0 |
| legal | 779 | 856 | 77 |
| unicode | 18 | 18 | 0 |

| 消息 | 新增合法 | 新增非法 |
| --- | ---: | ---: |
| ActV2 | 3 | 3 |
| AttributeRule | 7 | 6 |
| AttributeRuleTable | 3 | 6 |
| ClaimV2 | 3 | 6 |
| ConceptLink | 3 | 3 |
| ConceptRegistry | 3 | 6 |
| ConceptRevision | 3 | 6 |
| ConceptRole | 3 | 3 |
| EventAttributesRecordV2 | 4 | 15 |
| RefV2 | 3 | 3 |
| ScopedVerification | 3 | 6 |
| SemanticGraphV2 | 3 | 6 |
| SourceField | 4 | 3 |
| SourceLocation | 5 | 6 |
| SourceMetadataV2 | 4 | 9 |
| SourceTransform | 3 | 3 |
| StageAdoption | 5 | 12 |
| StageCoverage | 3 | 6 |
| TargetPremise | 3 | 6 |
| TextAdapterEntry | 3 | 6 |
| TextAdapterTable | 3 | 6 |
| VerificationRequirement | 3 | 6 |

新增的 132 条非法向量包括 66 条版本或字段错误，以及 22 个语义反例各自对应的对象、CanonicalV1 和 Protobuf 输入。后两种输入由只检查结构的编码器生成，使三端解码后仍须核对语义。
