import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withTileSlot } from "#/testing/stories";

import { IPickedColor } from "./color-intensity";
import { ColorPicker, IColorPickerProps } from "./ColorPicker";

const SUNSET: IPickedColor = { r: 245, g: 170, b: 77 };

function toValueArgs(value: IPickedColor): Partial<IColorPickerProps> {
  return { value };
}

function ColorPickerStory(args: IColorPickerProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <ColorPicker {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Color/ColorPicker",
  component: ColorPicker,
  render: ColorPickerStory,
  decorators: [withTileSlot],
  args: {
    value: SUNSET,
    onChange: fn(),
  },
} satisfies Meta<typeof ColorPicker>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A saturation square over a hue bar; a drag reports at most once an interval, and once more when let go. */
export const Default: TStory = {};
