import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

import { ESyntaxToken } from "@/core/syntax/lib";

/** Colours belong to the syntax feature; plain text inherits its surface's foreground. */
const SYNTAX_COLORS: Record<Exclude<ESyntaxToken, ESyntaxToken.PLAIN>, { light: string; dark: string }> = {
  comment: { light: "#376b28", dark: "#6a9955" },
  string: { light: "#a31515", dark: "#ce9178" },
  number: { light: "#116644", dark: "#b5cea8" },
  keyword: { light: "#0000c0", dark: "#569cd6" },
  type: { light: "#166775", dark: "#4ec9b0" },
  directive: { light: "#8f0e9e", dark: "#c586c0" },
  section: { light: "#7a5c00", dark: "#dcdcaa" },
  key: { light: "#04517a", dark: "#9cdcfe" },
  operator: { light: "#606060", dark: "#909090" },
};

/**
 * Scoped token styles that follow the active CSS scheme without rerendering the source.
 *
 * @param theme - Theme providing the active color scheme selector.
 * @returns Descendant token styles for a syntax surface.
 */
export function getSyntaxSx(theme: Theme): SystemStyleObject<Theme> {
  const light: Record<string, { color: string }> = {};
  const dark: Record<string, { color: string }> = {};

  for (const [token, colors] of Object.entries(SYNTAX_COLORS)) {
    const selector: string = `& [data-syntax-token="${token}"]`;

    light[selector] = { color: colors.light };
    dark[selector] = { color: colors.dark };
  }

  return { ...light, ...theme.applyStyles("dark", dark) };
}
