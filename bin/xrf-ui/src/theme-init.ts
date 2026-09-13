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
