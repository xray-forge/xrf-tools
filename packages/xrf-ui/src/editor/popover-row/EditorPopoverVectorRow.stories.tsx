import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverVectorRow, IEditorPopoverVectorRowProps } from "./EditorPopoverVectorRow";

function toValueArgs(value: Array<number>): Partial<IEditorPopoverVectorRowProps> {
  return { value };
}

function EditorPopoverVectorRowStory(args: IEditorPopoverVectorRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverVectorRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverVectorRow",
  component: EditorPopoverVectorRow,
  render: EditorPopoverVectorRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Wind direction",
    description: "wind_direction",
    value: [0.6, 0, -0.8],
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverVectorRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Each component typed as it is held, unclamped. */
export const Vector: TStory = {};

/** A colour brighter than one: its swatch clamps, its intensity scales it, and the swatch opens a picker. */
export const Color: TStory = {
  args: { label: "Sun colour", description: "sun_color", value: [2.4, 1.9, 1.2], isColor: true },
};

/** A colour with a fourth component, kept as it is when the picker or the intensity moves the rest. */
export const ColorWithAlpha: TStory = {
  args: { label: "Fog colour", description: "fog_color", value: [0.42, 0.45, 0.5, 1], isColor: true },
};
