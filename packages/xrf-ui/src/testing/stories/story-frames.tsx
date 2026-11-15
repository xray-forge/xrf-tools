import type { Decorator } from "@storybook/react-vite";
import { ReactElement } from "react";

import { PANEL } from "#/theme";

/** A form row or a short block at the width the application reads it, so lines wrap where they do there. */
export function withReadingColumn(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"max-w-reading"}>
      <Story />
    </div>
  );
}

/** A bordered card of the reading plane at the reading width, as an open file's view sits in the workspace. */
export function withReadingCard(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"max-w-reading rounded-surface border border-divider surface-content"}>
      <Story />
    </div>
  );
}

/** A pane of fixed height for a listing that grows to fill what it is given, as a grid fills an editor's body. */
export function withListingPane(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"flex h-140 max-w-full"}>
      <Story />
    </div>
  );
}

/** A docked panel's slot: the frame level at the panel's default width, ruled off from the reading plane. */
export function withPanelSlot(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"h-80 border border-divider surface-frame"} style={{ width: PANEL.defaultWidth }}>
      <Story />
    </div>
  );
}

/** A strip of the frame level, as a toolbar or the caption carries its actions. */
export function withFrameStrip(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"flex w-fit items-center gap-0.5 surface-frame p-1"}>
      <Story />
    </div>
  );
}
