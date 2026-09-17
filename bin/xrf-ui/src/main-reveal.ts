/**
 * Shows the window as soon as the skeleton below it is parsed, inlined at the end of `index.html`.
 */
((): void => {
  interface ITauriInternals {
    metadata?: { currentWindow?: { label?: string } };
    invoke?: (command: string, args: Record<string, unknown>) => Promise<unknown>;
  }

  const internals: ITauriInternals | undefined = (window as { __TAURI_INTERNALS__?: ITauriInternals })
    .__TAURI_INTERNALS__;

  const label: string | undefined = internals?.metadata?.currentWindow?.label;

  if (!internals?.invoke || label === undefined) {
    return;
  }

  internals
    .invoke("plugin:window|show", { label })
    .then(() => internals.invoke?.("plugin:window|set_focus", { label }))
    .catch(console.error);
})();
