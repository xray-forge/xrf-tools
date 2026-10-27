import { render, RenderResult } from "@testing-library/react";
import { createElement, Fragment, PropsWithChildren, ReactElement, ReactNode } from "react";

import { ThemeRoot } from "../theme/ThemeRoot";

export interface IRenderWithThemeOptions {
  /** Mount everything twice, the way a development build does, for subjects that hold something across a remount. */
  isStrict?: boolean;
}

/** The key a test's colour scheme is kept under, apart from the application's own. */
const TEST_COLOR_SCHEME_STORAGE_KEY: string = "xrf-ui-test-color-scheme";

function ThemeWrapper({ children }: PropsWithChildren): ReactElement {
  return createElement(ThemeRoot, { colorSchemeStorageKey: TEST_COLOR_SCHEME_STORAGE_KEY }, children);
}

/**
 * Renders a kit component the way it draws in any tree: under `ThemeRoot` and nothing else, so a component that
 * needs more than the theme fails here rather than in the application.
 *
 * @param ui - Content to render.
 * @param options - Render options.
 * @param options.isStrict - Whether to mount twice, as a development build does.
 * @returns The Testing Library render result.
 */
export function renderWithTheme(ui: ReactNode, { isStrict = false }: IRenderWithThemeOptions = {}): RenderResult {
  const result: RenderResult = render(createElement(Fragment, null, ui), {
    wrapper: ThemeWrapper,
    reactStrictMode: isStrict,
  });

  // Re-wrapped the way the first render was: handed the subject bare, React would see a different element in that
  // position and remount it, losing whatever the subject was keeping across the update being tested.
  return { ...result, rerender: (next: ReactNode): void => result.rerender(createElement(Fragment, null, next)) };
}
