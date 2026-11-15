import { EmotionCache } from "@emotion/cache";
import { CacheProvider } from "@emotion/react";
import { ReactElement, ReactNode, useMemo } from "react";

import { CssBaseline, Theme, ThemeProvider } from "#/theme/styles";

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
 * The React half of the application's look: its stylesheets, its theme in both colour schemes, the style cache MUI
 * injects through, and the baseline. It starts dark, keeps the scheme a viewer picks, and never animates the switch.
 *
 * It does not publish the `--xrf-*` variables the kit's utilities read (`surface-content`, `panel-section`, `bg-well`
 * and the rest): those come only from `getPreloadThemeCss`, which the page hosting the tree must inline into its head,
 * as the application's `index.html` and the Storybook preview head do. Without it those utilities resolve to nothing.
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
