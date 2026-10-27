import { EmotionCache } from "@emotion/cache";
import { CacheProvider } from "@emotion/react";
import { ReactElement, ReactNode, useMemo } from "react";

import { CssBaseline } from "#/material/css-baseline";
import { Theme, ThemeProvider } from "#/material/styles";

import "./tailwind.css";
import "./base.css";
import "./syntax.css";
import { createApplicationStyleCache } from "./cache";
import { createApplicationTheme } from "./creation";

interface IThemeRootProps {
  /** Local storage key the chosen colour scheme is kept under. */
  colorSchemeStorageKey: string;
  children?: ReactNode;
}

/**
 * Everything a tree needs to draw in the application's look: its stylesheets, its theme in both colour schemes, the
 * style cache MUI injects through, and the baseline. It starts dark, keeps the scheme a viewer picks, and never animates the
 * switch.
 */
export function ThemeRoot({ colorSchemeStorageKey, children }: IThemeRootProps): ReactElement {
  const theme: Theme = useMemo(() => createApplicationTheme(), []);
  const cache: EmotionCache = useMemo(() => createApplicationStyleCache(), []);

  return (
    <CacheProvider value={cache}>
      <ThemeProvider
        defaultMode={"dark"}
        disableTransitionOnChange={true}
        modeStorageKey={colorSchemeStorageKey}
        noSsr={true}
        theme={theme}
      >
        <CssBaseline enableColorScheme={true} />

        {children}
      </ThemeProvider>
    </CacheProvider>
  );
}
