import { describe, expect, it } from "@jest/globals";
import { createTheme, getContrastRatio } from "@mui/material/styles";

import { getSyntaxSx } from "@/core/syntax/components/syntax.styles";
import { ESyntaxToken } from "@/core/syntax/lib";
import { createApplicationTheme } from "@/core/theme/creation";
import { SURFACE } from "@/core/theme/tokens";

describe("syntax colors", () => {
  it("emits both CSS schemes even when the theme defaults to dark", () => {
    const theme = createApplicationTheme();

    expect(theme.palette.mode).toBe("dark");
    expect(getSyntaxSx(theme)).toMatchObject({
      '& [data-syntax-token="keyword"]': { color: "#0000c0" },
      ...theme.applyStyles("dark", {
        '& [data-syntax-token="keyword"]': { color: "#569cd6" },
      }),
    });
  });

  it.each(["light", "dark"] as const)("keeps small syntax text readable on %s code surfaces", (scheme) => {
    const styles = getSyntaxSx(createTheme({ palette: { mode: scheme } }));
    const tokens = Object.values(ESyntaxToken).filter((token) => token !== ESyntaxToken.PLAIN);

    expect(Object.keys(styles ?? {})).toEqual(
      expect.arrayContaining(tokens.map((token) => `& [data-syntax-token="${token}"]`))
    );

    for (const style of Object.values(styles ?? {})) {
      expect(style).toEqual({ color: expect.any(String) });

      if (style && typeof style === "object" && "color" in style && typeof style.color === "string") {
        for (const surface of [SURFACE.default, SURFACE.paper]) {
          expect(getContrastRatio(style.color, surface[scheme])).toBeGreaterThanOrEqual(4.5);
        }
      }
    }
  });
});
