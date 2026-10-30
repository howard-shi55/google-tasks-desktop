// Run again after full navigation; existing CSS also covers SPA-created elements.
(() => {
  // Recheck after eval scheduling so a navigation race cannot theme the login page.
  if (
    location.origin !== "https://tasks.google.com" ||
    !location.pathname.startsWith("/embed/")
  ) {
    return;
  }

  const id = "google-tasks-desktop-theme";
  const style = document.getElementById(id) || document.createElement("style");
  style.id = id;
  // ponytail: clear all page surface colors to avoid unstable Google class names;
  // add targeted exceptions here if Google introduces controls that lose state cues.
  style.textContent = `
    :root { color-scheme: dark; }
    html, body, body * {
      background-color: transparent !important;
      color: #ffffff !important;
      caret-color: #ffffff !important;
      text-shadow: 0 1px 3px rgba(0, 0, 0, 0.8) !important;
    }
    html, body { background-image: none !important; }
    input::placeholder, textarea::placeholder {
      color: #ffffff !important;
      opacity: 0.65 !important;
    }
    /* Floating surfaces must remain legible over the task text behind them. */
    [role="dialog"], [role="dialog"] *,
    [role="alertdialog"], [role="alertdialog"] *,
    [role="menu"], [role="menu"] *,
    [role="listbox"], [role="listbox"] *,
    [role="tooltip"], [role="tooltip"] * {
      background-color: #202124 !important;
    }
    /* Keep keyboard focus and selected controls visible after clearing fills. */
    :focus-visible {
      outline: 2px solid #ffffff !important;
      outline-offset: 2px;
    }
    [aria-checked="true"], [aria-selected="true"] {
      box-shadow: inset 0 0 0 1px #ffffff !important;
    }
  `;

  // A single style node avoids duplicate rules after repeated load callbacks.
  if (!style.isConnected) {
    document.head.appendChild(style);
  }

  // Install once per document even if the load callback reinjects the theme.
  if (!document.__desktopDragInstalled) {
    document.__desktopDragInstalled = true;
    document.addEventListener("mousedown", async (event) => {
      // ponytail: the current Tasks/My Tasks header is 80 CSS pixels high;
      // replace this bound with a confirmed DOM selector if Google changes its layout.
      if (
        !event.isTrusted ||
        event.defaultPrevented ||
        event.button !== 0 ||
        event.ctrlKey ||
        event.altKey ||
        event.shiftKey ||
        event.metaKey ||
        event.clientY < 0 ||
        event.clientY >= 80 ||
        event.target.closest(
          'button, a, input, textarea, select, [contenteditable]:not([contenteditable="false"]), [role="button"], [role="combobox"], [role="menu"], [role="listbox"], [role="dialog"], [role="alertdialog"]',
        )
      ) {
        return;
      }
      const api = window.__TAURI__?.window;
      if (!api) return;
      // Only the scoped native drag permission is exposed to this remote document.
      event.preventDefault();
      try {
        await api.getCurrentWindow().startDragging();
      } catch (error) {
        console.error("Could not drag task window:", error);
      }
    });
  }
})();
