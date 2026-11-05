import { PointerEventHandler, ReactElement, ReactNode, useMemo } from "react";

import { Paper } from "#/material/paper";
import { Popper } from "#/material/popper";

/** Where an open menu stands: under its menubar button, or beside the row opening it as a submenu. */
export type TMenubarPopupPlacement = "bottom-start" | "right-start";

/** How far an open menu keeps from the window's edges. */
const POPUP_EDGE_GAP: number = 8;

/** How far a menu stands below its menubar button, as a popover keeps clear of what opened it. */
const POPUP_MENU_OFFSET: number = 4;

/** Keeps a menu inside the window, moved along the row or button that opened it rather than off the edge. */
const POPUP_OVERFLOW_MODIFIER = { name: "preventOverflow", options: { padding: POPUP_EDGE_GAP } } as const;

/**
 * Sets a menu a little below its button and a submenu's first row level with the row opening it, past the list's
 * padding and the surface's border.
 */
const POPUP_MODIFIERS: Record<TMenubarPopupPlacement, Array<{ name: string; options: object }>> = {
  "bottom-start": [{ name: "offset", options: { offset: [0, POPUP_MENU_OFFSET] } }, POPUP_OVERFLOW_MODIFIER],
  "right-start": [{ name: "offset", options: { offset: [-5, 2] } }, POPUP_OVERFLOW_MODIFIER],
};

interface IMenubarPopupProps {
  anchor: HTMLElement;
  placement: TMenubarPopupPlacement;
  children: ReactNode;
  onPointerEnter?: PointerEventHandler<HTMLElement>;
}

/**
 * The surface an open menu is drawn on, over everything and beside what opened it, scrolling a menu taller than the
 * window rather than running past it. A submenu draws in a layer of its own, so the scroll never clips one.
 */
export function MenubarPopup({ anchor, placement, children, onPointerEnter }: IMenubarPopupProps): ReactElement {
  // The tallest the menu draws before it scrolls: from under its button to the window's foot for a menubar's menu,
  // which cannot move up past it, or the window's height for a submenu, moved up beside its row to fit. Measured once,
  // as what opened it stays put while it is open.
  const maxHeight: string = useMemo(() => {
    const top: number =
      placement === "bottom-start" ? anchor.getBoundingClientRect().bottom + POPUP_MENU_OFFSET : POPUP_EDGE_GAP;

    return `calc(100vh - ${top + POPUP_EDGE_GAP}px)`;
  }, [anchor, placement]);

  return (
    <Popper
      // The list inside is the menu; a popper is a tooltip to assistive technology unless told otherwise.
      role={"presentation"}
      open={true}
      anchorEl={anchor}
      placement={placement}
      modifiers={POPUP_MODIFIERS[placement]}
      sx={{ zIndex: "modal" }}
    >
      <Paper
        className={"overflow-y-auto rounded-surface border border-divider"}
        elevation={8}
        style={{ maxHeight }}
        onPointerEnter={onPointerEnter}
      >
        {children}
      </Paper>
    </Popper>
  );
}
