import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { runInNewContext } from "node:vm";

const script = readFileSync(
  new URL("../src-tauri/src/theme.js", import.meta.url),
  "utf8",
);

// Check the trust boundary and repeated navigation without a Google login.
test("theme is isolated, repeatable and only drags non-interactive header space", async () => {
  for (const address of [
    "https://accounts.google.com/",
    "http://tasks.google.com/embed/",
    "https://tasks.google.com.evil.example/embed/",
    "https://tasks.google.com/tasks/",
    "https://tasks.google.com/embed-evil/",
    "https://tasks.google.com/embedded/",
  ]) {
    runInNewContext(script, {
      location: new URL(address),
      get document() {
        assert.fail(`Unexpected DOM access on ${address}`);
      },
    });
  }

  const nodes = new Map();
  const listeners = [];
  let drags = 0;
  const document = {
    getElementById: (id) => nodes.get(id),
    createElement: () => ({ isConnected: false }),
    // Repeated injections must not register duplicate native-drag callbacks.
    addEventListener(type, handler) {
      assert.equal(type, "mousedown");
      listeners.push(handler);
    },
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
    window: {
      __TAURI__: {
        window: {
          getCurrentWindow: () => ({
            // Count native requests without moving the real desktop window.
            async startDragging() {
              drags++;
            },
          }),
        },
      },
    },
  };
  runInNewContext(script, context);
  const original = nodes.get("google-tasks-desktop-theme");
  // Real Tasks navigates into list subroutes; those must pass the inner JS guard too.
  for (const address of [
    "https://tasks.google.com/embed/?authuser=0",
    "https://tasks.google.com/embed/list/~default",
    "https://tasks.google.com/embed/list/another-list?authuser=1",
  ]) {
    context.location = new URL(address);
    original.textContent = "stale rules";
    runInNewContext(script, context);
    assert.match(original.textContent, /background-color: transparent/);
  }
  assert.equal(nodes.size, 1);
  assert.equal(nodes.get("google-tasks-desktop-theme"), original);
  assert.match(original.textContent, /background-color: transparent/);
  assert.match(original.textContent, /color: #ffffff/);
  assert.equal(listeners.length, 1);

  // Only a trusted primary-button press on blank header space may start dragging.
  const event = {
    isTrusted: true,
    button: 0,
    clientY: 40,
    target: { closest: () => null },
    preventDefault() {},
  };
  for (const overrides of [
    { isTrusted: false },
    { defaultPrevented: true },
    { button: 2 },
    { clientY: -1 },
    { clientY: 80 },
    { clientY: 300 },
    { ctrlKey: true },
    { altKey: true },
    { shiftKey: true },
    { metaKey: true },
    { target: { closest: () => ({ role: "button" }) } },
  ]) {
    await listeners[0]({ ...event, ...overrides });
  }
  assert.equal(drags, 0);
  await listeners[0](event);
  assert.equal(drags, 1);

  // Missing native API should leave normal browser mouse behavior untouched.
  context.window.__TAURI__ = undefined;
  await listeners[0]({
    ...event,
    preventDefault() {
      assert.fail("Cannot consume a mouse event without the native drag API");
    },
  });
});
