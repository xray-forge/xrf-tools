import { afterEach, describe, expect, it, jest } from "@jest/globals";

import { readColorScheme, toColorSchemeSelector, watchColorScheme } from "@/core/theme/color-scheme";
import { ColorScheme } from "@/core/theme/tokens";

async function flush(): Promise<void> {
  await Promise.resolve();
}

describe("color-scheme", () => {
  afterEach(() => {
    document.documentElement.removeAttribute("data-color-scheme");
  });

  it("reads the scheme shown, dark until one is set", () => {
    expect(readColorScheme()).toBe("dark");

    document.documentElement.setAttribute("data-color-scheme", "light");

    expect(readColorScheme()).toBe("light");
    expect(document.documentElement.matches(toColorSchemeSelector("light"))).toBe(true);
  });

  it("tells each change of scheme until stopped", async () => {
    const listener = jest.fn<(scheme: ColorScheme) => void>();
    const stop: () => void = watchColorScheme(listener);

    document.documentElement.setAttribute("data-color-scheme", "light");
    await flush();

    expect(listener).toHaveBeenCalledWith("light");

    stop();
    document.documentElement.setAttribute("data-color-scheme", "dark");
    await flush();

    expect(listener).toHaveBeenCalledTimes(1);
  });
});
