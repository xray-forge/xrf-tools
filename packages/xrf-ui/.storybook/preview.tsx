import type { Decorator, Preview } from "@storybook/react-vite";
import { ReactElement, ReactNode, useEffect } from "react";

import { ColorScheme } from "#/theme";
import { getPreloadThemeCss, ThemeRoot } from "#/theme/root";
import { useColorScheme } from "#/theme/styles";

import { COLOR_SCHEME_STORAGE_KEY } from "./color-scheme-storage";
import { PRELOAD_STYLE_ID } from "./preview-head";

/**
 * The preview head already carries this stylesheet from the server's start; written again here, into the same element,
 * so a hot update to the theme reaches the page without restarting Storybook.
 */
function installPreloadTheme(): void {
  const style: HTMLStyleElement =
    (document.getElementById(PRELOAD_STYLE_ID) as HTMLStyleElement | null) ?? document.createElement("style");

  style.id = PRELOAD_STYLE_ID;
  style.textContent = getPreloadThemeCss();
  document.head.prepend(style);
}

installPreloadTheme();

interface IColorSchemeSyncProps {
  scheme: ColorScheme;
  children: ReactNode;
}

/** Hands the toolbar's scheme to MUI, which writes the document attribute every scheme-dependent rule reads. */
function ColorSchemeSync({ scheme, children }: IColorSchemeSyncProps): ReactNode {
  const { setMode } = useColorScheme();

  useEffect(() => {
    setMode(scheme);
  }, [scheme, setMode]);

  return children;
}

/** Every story under the kit's theme on the reading plane, in the scheme the toolbar picks. */
function withThemeRoot(...[Story, context]: Parameters<Decorator>): ReactElement {
  return (
    <ThemeRoot colorSchemeStorageKey={COLOR_SCHEME_STORAGE_KEY}>
      <ColorSchemeSync scheme={context.globals.scheme === "light" ? "light" : "dark"}>
        <div className={"min-h-screen surface-content p-6 text-text-primary"}>
          <Story />
        </div>
      </ColorSchemeSync>
    </ThemeRoot>
  );
}

const preview: Preview = {
  decorators: [withThemeRoot],
  globalTypes: {
    scheme: {
      description: "Colour scheme",
      toolbar: {
        title: "Scheme",
        icon: "mirror",
        items: [
          { value: "dark", title: "Dark", icon: "moon" },
          { value: "light", title: "Light", icon: "sun" },
        ],
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: {
    scheme: "dark",
  },
  parameters: {
    layout: "fullscreen",
    controls: { expanded: true },
  },
};

export default preview;
