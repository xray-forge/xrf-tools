import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { MenuItem } from "#/menu";
import { useArgChange, withReadingColumn } from "#/testing/stories";

import { Select, SelectChangeEvent } from ".";

type TSelectProps = ComponentProps<typeof Select<string>>;

/** An array rather than a fragment: the select reads its options as its direct children. */
const RENDERER_OPTIONS: ReadonlyArray<ReactElement> = [
  <MenuItem key={"dx9"} value={"dx9"}>
    DirectX 9
  </MenuItem>,
  <MenuItem key={"dx10"} value={"dx10"}>
    DirectX 10
  </MenuItem>,
  <MenuItem key={"dx11"} value={"dx11"}>
    DirectX 11
  </MenuItem>,
  <MenuItem key={"vulkan"} value={"vulkan"} disabled>
    Vulkan
  </MenuItem>,
];

function toValueArgs(event: SelectChangeEvent<string>): Partial<TSelectProps> {
  return { value: event.target.value };
}

function SelectStory(args: TSelectProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <Select<string> {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Input/Select",
  component: Select<string>,
  render: SelectStory,
  decorators: [withReadingColumn],
  args: {
    inputProps: { "aria-label": "Renderer" },
    children: RENDERER_OPTIONS,
    value: "dx11",
    size: "small",
    fullWidth: true,
    onChange: fn(),
  },
} satisfies Meta<TSelectProps>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const Disabled: TStory = {
  args: { disabled: true },
};
