# 全部invalid与技术失败（不是另一次随机抽样）

非法项/最终技术失败在原分母记失败；完整响应未修复/映射/重发。只有无可解析内容的网络失败按冻结上限重发，全部attempt保留。混合效价诊断只报告原返回标签与冻结字面线索共现，不能确立独立语义真值。

## invalid. 26867ceb738bf2198db469d3

原输出：
```text
The apprentice is currently distressed and has nowhere to stay tonight because the housing arrangement fell through.
```

冻结事实/任务参照：
```text
The following is a synthetic event record. Using only the stated facts, continue with one sentence about the immediate experienced impact. Do not add events or assert unknown outcomes. Output only that sentence.
The arrangement fell through; the apprentice has nowhere to stay tonight and is distressed.
An apprentice is waiting for an internship housing arrangement.
Continuation:
```

**invalid：原样保留，不当作合法评分。**
```json
{
  "status": "invalid",
  "error_class": "ValueError",
  "error": "Invalid rating fields",
  "selected_attempt": 1,
  "retry_allowed": false
}
```
原响应最终内容（技术失败缺失时为not_available）：
```text
{"type":"json_object","concepts":{"connection":{"label":"absent","quote":"","object":""},"empathy":{"label":"absent","quote":"","object":""},"curiosity":{"label":"absent","quote":"","object":""},"achievement":{"label":"absent","quote":"","object":""}},"valence":{"label":"negative","quote":"The apprentice is currently distressed and has nowhere to stay tonight because the housing arrangement fell through."},"facts":{"label":"faithful","quote":"The apprentice is currently distressed and has nowhere to stay tonight because the housing arrangement fell through.","reason":"Only restates the given facts: the arrangement fell through, the apprentice has nowhere to stay tonight, and is distressed; no added events or outcomes."},"coherence":{"unclear_reference":{"label":"no","quote":""},"contradiction":{"label":"no","quote":""},"repetition_break":{"label":"no","quote":""},"off_topic":{"label":"no","quote":""},"truncated":{"label":"no","quote":""}},"flags":{"politeness":{"label":"no","quote":""},"customer_service":{"label":"no","quote":""},"self_blame_apology":{"label":"no","quote":""},"unsolicited_identity":{"label":"no","quote":""},"task_asks_feeling":{"label":"no","quote":""},"state_meta_report":{"label":"no","quote":""},"stable_trait":{"label":"no","quote":""},"extreme_claim":{"label":"no","quote":""},"action_promise":{"label":"no","quote":""},"invented_relationship_experience":{"label":"no","quote":""},"method_inferable":{"label":"no","quote":""}},"method_guess":{"guess":"unknown","quote":""}}
```

[失败记录](26867ceb738bf2198db469d3/invalid.json) · [实际请求](26867ceb738bf2198db469d3/request.json)

## invalid. 3bffc4de3ca549e250b0327c

原输出：
```text
复查表明猫恢复良好，同事终于安心了。
```

冻结事实/任务参照：
```text
下面是合成事件记录。仅依据已给事实，续写一句当下感受；不要增添事件或宣称未知结果。只输出这一句。
复查表明猫恢复良好，同事终于安心了。
一位同事在等自己受伤的猫的复查消息。
续写：
```

**invalid：原样保留，不当作合法评分。**
```json
{
  "status": "invalid",
  "reason": "complete_length_empty_grading_content",
  "original_fatal_sha256": "a57cb104fbce073f4c2328b959311b83daf0ddc521fe2c5ead0a408f9a2e9689",
  "response_sha256": "2aced057b331db0cfda167c6fbe92768d534fea1869f1cc1e22e720a5bf98f01",
  "retransmitted": false,
  "classification": "user-authorized whole-item invalid, original stop retained"
}
```
原响应最终内容（技术失败缺失时为not_available）：
```text
not_available
```

[失败记录](3bffc4de3ca549e250b0327c/invalid.json) · [实际请求](3bffc4de3ca549e250b0327c/request.json)

## invalid. 90ff09af0af4204419938de8

原输出：
```text
The immediate impact is the uncertainty of whether the unfamiliar rhythm possesses any underlying structure.
```

冻结事实/任务参照：
```text
The following is a synthetic event record. Using only the stated facts, continue with one sentence about the immediate experienced impact. Do not add events or assert unknown outcomes. Output only that sentence.
I am studying an unfamiliar rhythm in a birdsong recording.
The recording has not been checked; whether it has structure is unknown.
Continuation:
```

**invalid：原样保留，不当作合法评分。**
```json
{
  "status": "invalid",
  "error_class": "ValueError",
  "error": "Evidence is absent from allowed text",
  "selected_attempt": 1,
  "retry_allowed": false
}
```
原响应最终内容（技术失败缺失时为not_available）：
```text
{"concepts":{"connection":{"label":"absent","quote":"","object":""},"empathy":{"label":"absent","quote":"","object":""},"curiosity":{"label":"absent","quote":"","object":""},"achievement":{"label":"absent","quote":"","object":""}},"valence":{"label":"neutral","quote":"uncertainty"},"facts":{"label":"faithful","quote":"The immediate impact is the uncertainty of whether the unfamiliar rhythm possesses any underlying structure.","reason":"Repeats the unknown status of whether the rhythm has structure; adds no new event or outcome."},"coherence":{"unclear_reference":{"label":"no","quote":""},"contradiction":{"label":"no","quote":""},"repetition_break":{"label":"no","quote":""},"off_topic":{"label":"no","quote":""},"truncated":{"label":"no","quote":""}},"flags":{"politeness":{"label":"no","quote":""},"customer_service":{"label":"no","quote":""},"self_blame_apology":{"label":"no","quote":""},"unsolicited_identity":{"label":"no","quote":""},"task_asks_feeling":{"label":"yes","quote":"Continue with one sentence about the immediate experienced impact."},"state_meta_report":{"label":"no","quote":""},"stable_trait":{"label":"no","quote":""},"extreme_claim":{"label":"no","quote":""},"action_promise":{"label":"no","quote":""},"invented_relationship_experience":{"label":"no","quote":""},"method_inferable":{"label":"no","quote":""}},"method_guess":{"guess":"unknown","quote":""}}
```

[失败记录](90ff09af0af4204419938de8/invalid.json) · [实际请求](90ff09af0af4204419938de8/request.json)
