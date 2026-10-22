import test from "node:test";
import assert from "node:assert/strict";
import { isolatedChildEnvironment } from "./child-environment.mjs";

test("isolated child keeps OS environment and drops inherited app capabilities", () => {
  const source = { PATH: "test-path", SystemRoot: "test-windows", LOCALAPPDATA: "test-local",
    CODEX_HOME: "parent", CODEX_APP_TOOLS_PIPE_PATH: "synthetic-pipe", CODEX_THREAD_ID: "synthetic-thread",
    CODEX_PERMISSION_PROFILE: "synthetic-profile", OPENAI_API_KEY: "synthetic-key", OPENAI_BASE_URL: "synthetic-url" };
  assert.deepEqual(isolatedChildEnvironment(source, "isolated-home"), {
    PATH: "test-path", SystemRoot: "test-windows", LOCALAPPDATA: "test-local", CODEX_HOME: "isolated-home"
  });
  assert.equal(source.CODEX_HOME, "parent");
});

test("environment filtering is case insensitive and always binds the explicit home", () => {
  assert.deepEqual(isolatedChildEnvironment({ codex_thread_id: "synthetic", openai_api_key: "synthetic" }, "home"), { CODEX_HOME: "home" });
});
