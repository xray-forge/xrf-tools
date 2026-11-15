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

/** One tile's width, as the launcher and the picker screens lay their cards out side by side. */
export function withTileSlot(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"w-72"}>
      <Story />
    </div>
  );
}

/** Room above and beside a small subject, for what it floats over itself: a tooltip, a popper. */
export function withOverlayRoom(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div className={"px-24 pt-16"}>
      <Story />
    </div>
  );
}

/** A toolbar popover's paper at its one width, for what sits inside one: a section, a row, a note. */
export function withPopoverSurface(...[Story]: Parameters<Decorator>): ReactElement {
  return (
    <div
      className={"flex w-popover flex-col gap-1 rounded-surface border border-divider bg-surface-overlay px-3 py-3"}
      style={{ boxShadow: "var(--xrf-shadow-overlay)" }}
    >
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
