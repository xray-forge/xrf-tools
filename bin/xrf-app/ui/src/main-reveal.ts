/**
 * Asks the backend to put the window on screen, inlined into `index.html` after the themed skeleton and before the
 * module bundle, so the window appears as soon as there is something themed to show. The backend reveals each window
 * once: a page reloaded over a shown window changes nothing.
 */
((): void => {
  // No module graph exists yet, so the generated command bindings cannot be imported: this is the invocation they make.
  (window as Record<string, any>)["__TAURI_INTERNALS__"]?.invoke("plugin:system|reveal_window").catch(console.error);
})();
