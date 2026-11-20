import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement, SyntheticEvent } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { Tab, Tabs } from ".";

type TTabsProps = ComponentProps<typeof Tabs>;

/** An array rather than a fragment: the tabs read their tabs as their direct children. */
const VIEW_TABS: ReadonlyArray<ReactElement> = [
  <Tab key={"source"} value={"source"} label={"Source"} />,
  <Tab key={"sections"} value={"sections"} label={"Sections"} />,
  <Tab key={"includes"} value={"includes"} label={"Includes"} />,
  <Tab key={"problems"} value={"problems"} label={"Problems"} disabled />,
];

function toValueArgs(_event: SyntheticEvent, value: unknown): Partial<TTabsProps> {
  return { value };
}

function TabsStory(args: TTabsProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <Tabs {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Tabs/Tabs",
  component: Tabs,
  render: TabsStory,
  decorators: [withReadingColumn],
  args: {
    "aria-label": "File views",
    value: "source",
    children: VIEW_TABS,
    onChange: fn(),
  },
} satisfies Meta<typeof Tabs>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** At the toolbar's height, in sentence case at the list text's size. */
export const Default: TStory = {};
