import { ReactElement } from "react";

import { XrfMark } from "@/core/brand/XrfMark";

/**
 * The application mark, where a window's own icon sits.
 *
 * Named rather than decorative: it is the only thing identifying the window, the caption text having been dropped.
 */
export function ApplicationTitleBarIcon(): ReactElement {
  return <XrfMark data-testid={"application-title-bar-icon"} size={16} title={"XRF tools"} sx={{ marginX: 1 }} />;
}
