import { Divider, MenuList, Paper, Popper } from "@xrf/ui/material";
import { KeyboardEvent, ReactElement } from "react";

import { IMenubarItem, IMenubarMenu } from "./menubar-menu";
import { MenubarRow } from "./MenubarRow";

/** Sets the menu a little below its button, as a popover keeps clear of what opened it. */
const POPUP_MODIFIERS: Array<{ name: string; options: { offset: [number, number] } }> = [
  { name: "offset", options: { offset: [0, 4] } },
];

interface IMenubarPopupProps {
  menu: IMenubarMenu;
  anchor: HTMLElement;
  /** Puts the caret on the first row, for a menu opened from the keyboard; a pointer opening focuses only the list. */
  isFirstRowFocused: boolean;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onChoose: (item: IMenubarItem) => void;
}

/**
 * The open menu under its menubar button: its sections, a divider between each.
 */
export function MenubarPopup({
  menu,
  anchor,
  isFirstRowFocused,
  onKeyDown,
  onChoose,
}: IMenubarPopupProps): ReactElement {
  // Rows and dividers as one flat list: `MenuList` walks its direct children to find the row to focus, and refuses a
  // fragment among them.
  const rows: Array<ReactElement> = menu
    .getSections()
    .flatMap((items: ReadonlyArray<IMenubarItem>, index: number) => [
      ...(index > 0 ? [<Divider key={`divider-${index}`} className={"my-1"} />] : []),
      ...items.map((item: IMenubarItem) => <MenubarRow key={item.id} item={item} onChoose={onChoose} />),
    ]);

  return (
    <Popper
      open={true}
      anchorEl={anchor}
      placement={"bottom-start"}
      modifiers={POPUP_MODIFIERS}
      sx={{ zIndex: "modal" }}
    >
      <Paper className={"rounded-surface border border-divider"} elevation={8}>
        <MenuList
          className={"min-w-44 py-1"}
          aria-label={menu.label}
          autoFocus={!isFirstRowFocused}
          autoFocusItem={isFirstRowFocused}
          dense={true}
          variant={"menu"}
          onKeyDown={onKeyDown}
        >
          {rows}
        </MenuList>
      </Paper>
    </Popper>
  );
}
