/**
 * Shows the window once the webview has had time to put its first frame on screen, inlined at the end of `index.html`.
 */
((): void => {
  setTimeout(() => {
    const tauriInternals = (window as Record<string, any>)["__TAURI_INTERNALS__"];
    const label = tauriInternals?.metadata?.currentWindow?.label;

    if (!tauriInternals?.invoke || !label) {
      return;
    }

    tauriInternals
      .invoke("plugin:window|is_visible", { label })
      .then((isVisible: boolean) =>
        isVisible
          ? undefined
          : tauriInternals
              .invoke("plugin:window|show", { label })
              .then(() => tauriInternals.invoke("plugin:window|set_focus", { label }))
      )
      .catch(console.error);
  }, 10);
})();
