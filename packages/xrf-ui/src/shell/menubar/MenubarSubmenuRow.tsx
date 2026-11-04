import { KeyboardEvent, MouseEvent, PointerEventHandler, ReactElement, RefCallback } from "react";

import { ChevronRightIcon } from "#/icons/chevron-right";
import { ListItemText } from "#/material/list";
import { MenuItem } from "#/material/menu";
import { cn } from "#/theme";

import { IMenubarSubmenu } from "./menubar-menu";

/** How a submenu was asked to open, which decides whether its first row takes the caret. */
export interface IMenubarSubmenuOpenOptions {
  isFromKeyboard: boolean;
}

interface IMenubarSubmenuRowProps {
  submenu: IMenubarSubmenu;
  /** Given the row's element, which its submenu is placed beside and the caret comes back to as it closes. */
  rowRef: RefCallback<HTMLLIElement>;
  isOpen: boolean;
  onOpen: (options: IMenubarSubmenuOpenOptions) => void;
  onPointerEnter: PointerEventHandler<HTMLElement>;
}

/** A row opening a submenu: its icon, the label and an arrow at the end, marked while its submenu is open. */
export function MenubarSubmenuRow({
  submenu,
  rowRef,
  isOpen,
  onOpen,
  onPointerEnter,
}: IMenubarSubmenuRowProps): ReactElement {
  return (
    <MenuItem
      ref={rowRef}
      aria-expanded={isOpen}
      aria-haspopup={"menu"}
      className={cn("mx-1 min-h-7 gap-2 rounded-control px-2 py-0.5", isOpen ? "bg-action-selected" : null)}
      dense={true}
      disabled={submenu.isDisabled}
      role={"menuitem"}
      // Enter and Space click the row as well, and a click the keyboard raises carries no press count.
      onClick={(event: MouseEvent<HTMLElement>) => onOpen({ isFromKeyboard: event.detail === 0 })}
      onKeyDown={(event: KeyboardEvent<HTMLElement>) => {
        if (event.key === "ArrowRight") {
          event.preventDefault();
          event.stopPropagation();
          onOpen({ isFromKeyboard: true });
        }
      }}
      onPointerEnter={onPointerEnter}
    >
      <span aria-hidden={true} className={"flex w-4 shrink-0 justify-center text-text-secondary [&>svg]:text-base"}>
        {submenu.icon}
      </span>

      <ListItemText className={"my-0"} primary={submenu.label} slotProps={{ primary: { variant: "body2" } }} />

      <ChevronRightIcon aria-hidden={true} className={"ml-auto shrink-0 text-base text-text-secondary"} />
    </MenuItem>
  );
}
