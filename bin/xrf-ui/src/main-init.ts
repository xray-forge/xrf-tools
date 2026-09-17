/**
 * Everything that has to happen before the document is first painted, inlined into `index.html` and run above the
 * stylesheets so that the rules reading `data-color-scheme` are parsed with the answer already in place.
 */
((): void => {
  function getStoredThemeMode(): string {
    const mode = window.localStorage.getItem("xrf.preference.theme");

    if (mode === "light" || mode === "dark") {
      return mode;
    } else if (mode === "system") {
      return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }

    return "dark";
  }

  const element: HTMLElement = document.documentElement;

  try {
    element.setAttribute("data-color-scheme", getStoredThemeMode());
  } catch {
    element.setAttribute("data-color-scheme", "dark");
  }
})();
