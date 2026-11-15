import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorPopoverContent } from "#/editor/popover-content";
import { EditorPopoverNote } from "#/editor/popover-row";
import { TuneIcon } from "#/icons/tune";
import { withFrameStrip } from "#/testing/stories";

import { EditorPopoverAction } from "./EditorPopoverAction";

const DISPLAY_SETTINGS: ReactElement = (
  <EditorPopoverContent label={"Display"}>
    <EditorPopoverNote>The settings this trigger opens, under the popover&apos;s name.</EditorPopoverNote>
  </EditorPopoverContent>
);

const meta = {
  title: "Components/Editor/EditorPopoverAction",
  component: EditorPopoverAction,
  decorators: [withFrameStrip],
  argTypes: {
    kind: { control: "inline-radio", options: ["popover", "toggle"] },
    placement: { control: "inline-radio", options: ["bottom", "top"] },
  },
  args: {
    label: "Display",
    description: "Opens the display settings",
    icon: <TuneIcon />,
    children: DISPLAY_SETTINGS,
  },
} satisfies Meta<typeof EditorPopoverAction>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A corner triangle says a click opens it; click or right-click to open. */
export const Popover: TStory = {};

/** Painted as a toggle that is on: the value behind it differs from its default. */
export const Active: TStory = {
  args: { isActive: true },
};

/** A corner dot says a click switches it; a right click still opens its settings. */
export const Toggle: TStory = {
  args: { kind: "toggle", onClick: fn() },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
