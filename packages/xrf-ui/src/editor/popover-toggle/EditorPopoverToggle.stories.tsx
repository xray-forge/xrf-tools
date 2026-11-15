import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorPopoverSwitchRow } from "#/editor/popover-row";
import { GridOnIcon } from "#/icons/grid-on";
import { useArgChange, withFrameStrip } from "#/testing/stories";

import { EditorPopoverToggle, IEditorPopoverToggleProps } from "./EditorPopoverToggle";

const GRID_SETTINGS: ReactElement = (
  <>
    <EditorPopoverSwitchRow label={"Minor lines"} isChecked={true} onChange={fn()} />
    <EditorPopoverSwitchRow label={"Axes"} isChecked={false} onChange={fn()} />
  </>
);

/** The handler is told only that a click happened, so the turn reads what it turns over from the args. */
function toToggledArgs({ isOn }: IEditorPopoverToggleProps): Partial<IEditorPopoverToggleProps> {
  return { isOn: !isOn };
}

function EditorPopoverToggleStory(args: IEditorPopoverToggleProps): ReactElement {
  const onToggle = useArgChange(args.onToggle, toToggledArgs);

  return <EditorPopoverToggle {...args} onToggle={onToggle} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverToggle",
  component: EditorPopoverToggle,
  render: EditorPopoverToggleStory,
  decorators: [withFrameStrip],
  args: {
    label: "Grid",
    description: "Ground grid",
    icon: <GridOnIcon />,
    isOn: true,
    toggleLabel: "Show grid",
    children: GRID_SETTINGS,
    onToggle: fn(),
  },
} satisfies Meta<typeof EditorPopoverToggle>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A click turns it over, a right click opens its settings under the toggle heading them. */
export const On: TStory = {};

export const Off: TStory = {
  args: { isOn: false },
};

/** The settings change nothing now, which the popover says in their place; the toggle still turns over. */
export const Locked: TStory = {
  args: { lockedReason: "The grid draws only in the top view." },
};

/** It cannot turn over: a click opens its settings, where the description says why. */
export const Disabled: TStory = {
  args: { isDisabled: true, description: "Nothing is loaded to draw a grid under" },
};
