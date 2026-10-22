import test from "node:test";
import assert from "node:assert/strict";
import { strictToolSurfaceViolation } from "./probe.mjs";

test("strict surface rejects calls and unknown raw capture", () => {
  assert.equal(strictToolSurfaceViolation({ rawToolCallNames: [] }), false);
  assert.equal(strictToolSurfaceViolation({ rawToolCallNames: ["list_mcp_resources"] }), true);
  assert.equal(strictToolSurfaceViolation({}), true);
});

test("isolated surface requires complete post-turn inventory with zero servers", () => {
  const clean = { rawToolCallNames: [], mcpServerCountAfter: 0, mcpHasNextPageAfter: false };
  assert.equal(strictToolSurfaceViolation(clean, true), false);
  for (const update of [{mcpServerCountAfter:3}, {mcpServerCountAfter:null}, {mcpHasNextPageAfter:true}, {mcpHasNextPageAfter:null}]) {
    assert.equal(strictToolSurfaceViolation({...clean,...update},true),true);
  }
});
