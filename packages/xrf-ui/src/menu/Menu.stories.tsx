import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { ContentCopyIcon } from "#/icons/content-copy";
import { DeleteOutlinedIcon } from "#/icons/delete-outlined";
import { Divider } from "#/layout/stack";
import { ListItemIcon, ListItemText } from "#/list";
import { useArgChange } from "#/testing/stories";

import { Menu, MenuItem } from ".";

type TMenuProps = ComponentProps<typeof Menu>;

/** An array rather than a fragment: the menu reads its rows as its direct children for keyboard focus. */
const ENTRY_ACTIONS: ReadonlyArray<ReactElement> = [
  <MenuItem key={"copy"} onClick={fn()}>
    <ListItemIcon>
      <ContentCopyIcon fontSize={"small"} />
    </ListItemIcon>
    <ListItemText>Copy path</ListItemText>
  </MenuItem>,
  <MenuItem key={"reveal"} onClick={fn()} selected>
    <ListItemText inset>Reveal in tree</ListItemText>
  </MenuItem>,
  <Divider key={"divider"} />,
  <MenuItem key={"delete"} onClick={fn()} disabled>
    <ListItemIcon>
      <DeleteOutlinedIcon fontSize={"small"} />
    </ListItemIcon>
    <ListItemText>Delete</ListItemText>
  </MenuItem>,
];

function toClosedArgs(): Partial<TMenuProps> {
  return { open: false };
}

function MenuStory(args: TMenuProps): ReactElement {
  const onClose = useArgChange(args.onClose, toClosedArgs);

  return <Menu {...args} onClose={onClose} />;
}

const meta = {
  title: "Components/Menu/Menu",
  component: Menu,
  render: MenuStory,
  args: {
    open: true,
    anchorReference: "anchorPosition",
    anchorPosition: { top: 48, left: 48 },
    children: ENTRY_ACTIONS,
    onClose: fn(),
  },
} satisfies Meta<typeof Menu>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The overlay level with its border and shadow; a chosen row takes the neutral `selected` scrim, not the accent. */
export const Open: TStory = {};
