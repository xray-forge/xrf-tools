import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange } from "#/testing/stories";
import { Typography } from "#/typography";

import { Popover } from ".";

type TPopoverProps = ComponentProps<typeof Popover>;

const POPOVER_BODY: ReactElement = (
  <Typography className={"w-popover p-3"} variant={"body2"}>
    What a toolbar trigger opens: settings that belong to the view, one width for every popover.
  </Typography>
);

function toClosedArgs(): Partial<TPopoverProps> {
  return { open: false };
}

function PopoverStory(args: TPopoverProps): ReactElement {
  const onClose = useArgChange(args.onClose, toClosedArgs);

  return <Popover {...args} onClose={onClose} />;
}

const meta = {
  title: "Components/Overlay/Popover",
  component: Popover,
  render: PopoverStory,
  args: {
    open: true,
    anchorReference: "anchorPosition",
    anchorPosition: { top: 48, left: 48 },
    children: POPOVER_BODY,
    onClose: fn(),
  },
} satisfies Meta<typeof Popover>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Both levels hold white in light, so the theme's border and overlay shadow are the whole of what says "above". */
export const Open: TStory = {};
