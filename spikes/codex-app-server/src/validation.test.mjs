import assert from "node:assert/strict";
import { test } from "node:test";
import { validateSchema } from "./candidate-validation.mjs";
import { validateSemanticCandidate } from "./probe.mjs";

const sample = { sourceRef: "p0-08-0001", text: "😀我完成了。" };
const candidate = () => ({
  schemaVersion: "p0-08-candidate-v1",
  sourceRef: sample.sourceRef,
  nodes: [{
    id: "act-1", kind: "Act", actType: "other", content: "我完成了。",
    sourceRefs: [sample.sourceRef], sourceSpan: [1, 6], derivation: "verbatim",
    epistemicType: "user_self_report", resolutionState: "resolved",
    representationVersion: "p0-08-candidate-v1"
  }]
});

test("original schema accepts the expected metadata and disallows extra fields", () => {
  const value = candidate();
  assert.equal(validateSchema(value).valid, true);
  value.nodes[0].hiddenState = "unregistered";
  assert.equal(validateSchema(value).valid, false);
});

test("source spans use Unicode code points and reject an out of range endpoint", () => {
  const value = candidate();
  assert.equal(validateSemanticCandidate(value, sample), true);
  value.nodes[0].sourceSpan = [1, 7];
  assert.equal(validateSchema(value).valid, true);
  assert.equal(validateSemanticCandidate(value, sample), false);
});

test("valid schema alone does not establish source binding or unique node identity", () => {
  const value = candidate();
  value.nodes[0].sourceRefs = ["another-source"];
  assert.equal(validateSchema(value).valid, true);
  assert.equal(validateSemanticCandidate(value, sample), false);
  value.nodes[0].sourceRefs = [sample.sourceRef];
  value.nodes.push(structuredClone(value.nodes[0]));
  assert.equal(validateSchema(value).valid, true);
  assert.equal(validateSemanticCandidate(value, sample), false);
});

test("the original schema retains unique source references and typed roles", () => {
  const value = candidate();
  value.nodes[0].sourceRefs.push(sample.sourceRef);
  assert.equal(validateSchema(value).valid, false);
  const claim = candidate();
  claim.nodes = [{ ...claim.nodes[0], kind: "Claim", predicateLabel: "completion", roles: [{ role: "actor", value: 17 }], value: true, polarity: "positive", modality: "asserted", time: null }];
  delete claim.nodes[0].actType;
  delete claim.nodes[0].content;
  assert.equal(validateSchema(claim).valid, false);
});
