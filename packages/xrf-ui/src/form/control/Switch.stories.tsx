import type { Meta, StoryObj } from "@storybook/react-vite";
import { ChangeEvent, ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange } from "#/testing/stories";

import { FormControlLabel, Switch } from ".";

type TSwitchProps = ComponentProps<typeof Switch>;

function toCheckedArgs(_event: ChangeEvent<HTMLInputElement>, checked: boolean): Partial<TSwitchProps> {
  return { checked };
}

function SwitchStory(args: TSwitchProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <FormControlLabel label={"Overwrite existing files"} control={<Switch {...args} onChange={onChange} />} />;
}

const meta = {
  title: "Components/Form/Switch",
  component: Switch,
  render: SwitchStory,
  argTypes: {
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    checked: true,
    size: "small",
    onChange: fn(),
  },
} satisfies Meta<typeof Switch>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Labelled through `FormControlLabel`, the family's other pass-through. */
export const On: TStory = {};

export const Off: TStory = {
  args: { checked: false },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
