import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withFrameStrip } from "#/testing/stories";

import { EditorToolbarCrumb } from "./EditorToolbarCrumb";

const meta = {
  title: "Components/Editor/EditorToolbarCrumb",
  component: EditorToolbarCrumb,
  decorators: [withFrameStrip],
  args: {
    label: "Archives",
    accessibleName: "Back to Archives",
    hint: "Back to Archives, closing what is open",
    onClick: fn(),
  },
} satisfies Meta<typeof EditorToolbarCrumb>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A crumb that leads back to the tool, closing what is open. */
export const Interactive: TStory = {};

/** Without `onClick` the crumb is the tool's name and nothing more. */
export const Inert: TStory = {
  args: { onClick: undefined, accessibleName: undefined, hint: undefined },
};

/** Held while the tool is busy, so a click cannot leave mid-run. */
export const Busy: TStory = {
  args: { isDisabled: true },
};
