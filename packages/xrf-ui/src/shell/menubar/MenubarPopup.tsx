import { PointerEventHandler, ReactElement, ReactNode } from "react";

import { Paper } from "#/material/paper";
import { Popper } from "#/material/popper";

/** Where an open menu stands: under its menubar button, or beside the row opening it as a submenu. */
export type TMenubarPopupPlacement = "bottom-start" | "right-start";

/**
 * Sets a menu a little below its button, as a popover keeps clear of what opened it, and a submenu's first row level
 * with the row opening it, past the list's padding and the surface's border.
 */
const POPUP_MODIFIERS: Record<
  TMenubarPopupPlacement,
  Array<{ name: string; options: { offset: [number, number] } }>
> = {
  "bottom-start": [{ name: "offset", options: { offset: [0, 4] } }],
  "right-start": [{ name: "offset", options: { offset: [-5, 2] } }],
};

interface IMenubarPopupProps {
  anchor: HTMLElement;
  placement: TMenubarPopupPlacement;
  children: ReactNode;
  onPointerEnter?: PointerEventHandler<HTMLElement>;
}

/**
 * The surface an open menu is drawn on, over everything and beside what opened it.
 */
export function MenubarPopup({ anchor, placement, children, onPointerEnter }: IMenubarPopupProps): ReactElement {
  return (
    <Popper
      open={true}
      anchorEl={anchor}
      placement={placement}
      modifiers={POPUP_MODIFIERS[placement]}
      sx={{ zIndex: "modal" }}
    >
      <Paper className={"rounded-surface border border-divider"} elevation={8} onPointerEnter={onPointerEnter}>
        {children}
      </Paper>
    </Popper>
  );
}
