import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverSliderRow, IEditorPopoverSliderRowProps } from "./EditorPopoverSliderRow";

function formatMetres(value: number): string {
  return `${value} m`;
}

function toValueArgs(value: number): Partial<IEditorPopoverSliderRowProps> {
  return { value };
}

function EditorPopoverSliderRowStory(args: IEditorPopoverSliderRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverSliderRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverSliderRow",
  component: EditorPopoverSliderRow,
  render: EditorPopoverSliderRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Draw distance",
    description: "How far the world is drawn.",
    hint: "r__far_plane",
    value: 450,
    min: 50,
    max: 1500,
    step: 10,
    format: formatMetres,
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverSliderRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Nothing rated: the slider and what it reads. */
export const Plain: TStory = {};

/** The recommended band on the rail; outside it the slider and the readout take the warning colour. */
export const Recommended: TStory = {
  args: { recommended: [200, 800], value: 1000 },
};

/** Past the measured useful end, drawn red on the rail. */
export const Wasted: TStory = {
  args: { recommended: [200, 800], wastedAbove: 1200, value: 1300 },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
