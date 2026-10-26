import { ReactElement } from "react";

import { XrfMark } from "@/core/brand/XrfMark";
import { PRODUCT_NAME } from "@/core/configs";

/**
 * The application mark, where a window's own icon sits.
 *
 * Named rather than decorative: it is the only thing identifying the window, the caption text having been dropped.
 */
export function ApplicationTitleBarIcon(): ReactElement {
  // 8 + 20 + 8 is the rail's width, so the mark centres over the rail's buttons and what follows starts at its edge.
  return <XrfMark data-testid={"application-title-bar-icon"} className={"mx-2"} size={20} title={PRODUCT_NAME} />;
}
