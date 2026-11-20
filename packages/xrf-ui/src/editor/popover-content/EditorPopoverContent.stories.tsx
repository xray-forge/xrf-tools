import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorPopoverCheckboxRow, EditorPopoverNote } from "#/editor/popover-row";
import { withOverlayRoom } from "#/testing/stories";

import { EditorPopoverContent } from "./EditorPopoverContent";

const GRID_ROWS: ReactElement = (
  <>
    <EditorPopoverCheckboxRow label={"Minor lines"} isChecked={true} onChange={fn()} />
    <EditorPopoverCheckboxRow label={"Axes"} isChecked={false} onChange={fn()} />
    <EditorPopoverNote>Lines every metre, heavier every ten.</EditorPopoverNote>
  </>
);

const meta = {
  title: "Components/Editor/EditorPopoverContent",
  component: EditorPopoverContent,
  decorators: [withOverlayRoom],
  args: {
    label: "Grid",
    children: GRID_ROWS,
  },
} satisfies Meta<typeof EditorPopoverContent>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** What a toolbar popover holds, at the one width every popover takes so their rows line up. */
export const Default: TStory = {};

export const WithReset: TStory = {
  args: { onReset: fn(), resetLabel: "Reset the grid" },
};

/** For rows of several named choices. */
export const Wide: TStory = {
  args: { isWide: true },
};
