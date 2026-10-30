// Run again after full navigation; existing CSS also covers SPA-created elements.
(() => {
  // Recheck after eval scheduling so a navigation race cannot theme the login page.
  if (
    location.origin !== "https://tasks.google.com" ||
    location.pathname !== "/embed/"
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
})();
