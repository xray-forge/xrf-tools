import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactNode } from "react";

import { withPopoverSurface } from "#/testing/stories";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

/** A plain control, handed the attributes the row names and describes it by. */
function renderFieldControl(control: IEditorPopoverRowControlProps): ReactNode {
  return <input {...control} className={"w-full rounded-control border border-divider bg-well px-2 py-0.5 text-xs"} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverRow",
  component: EditorPopoverRow,
  decorators: [withPopoverSurface],
  args: {
    label: "Exposure",
    description: "Scales the scene's light before tone mapping",
    children: renderFieldControl,
  },
} satisfies Meta<typeof EditorPopoverRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The label in the column every popover row shares, the control beside it; the description reads on hover. */
export const Inline: TStory = {};

/** For a control too wide to share the row. */
export const Stacked: TStory = {
  args: { isStacked: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
