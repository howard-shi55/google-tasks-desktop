import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { runInNewContext } from "node:vm";

const script = readFileSync(
  new URL("../src-tauri/src/theme.js", import.meta.url),
  "utf8",
);

// Check the trust boundary and repeated navigation without a Google login.
test("theme only touches Tasks embed and updates a single style node", () => {
  for (const address of [
    "https://accounts.google.com/",
    "http://tasks.google.com/embed/",
    "https://tasks.google.com.evil.example/embed/",
    "https://tasks.google.com/tasks/",
  ]) {
    runInNewContext(script, {
      location: new URL(address),
      get document() {
        assert.fail(`Unexpected DOM access on ${address}`);
      },
    });
  }

  const nodes = new Map();
  const document = {
    getElementById: (id) => nodes.get(id),
    createElement: () => ({ isConnected: false }),
    head: {
      // Model insertion so a second eval has to reuse the connected node.
      appendChild(node) {
        assert.equal(nodes.size, 0);
        node.isConnected = true;
        nodes.set(node.id, node);
      },
    },
  };
  const context = {
    document,
    location: new URL("https://tasks.google.com/embed/?authuser=0"),
  };
  runInNewContext(script, context);
  const original = nodes.get("google-tasks-desktop-theme");
  original.textContent = "stale rules";
  runInNewContext(script, context);
  assert.equal(nodes.size, 1);
  assert.equal(nodes.get("google-tasks-desktop-theme"), original);
  assert.match(original.textContent, /background-color: transparent/);
  assert.match(original.textContent, /color: #ffffff/);
});
