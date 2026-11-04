import { Nullable } from "@xrf/types";
import { FocusEvent, KeyboardEvent, ReactElement, useCallback, useEffect, useRef, useState } from "react";

import { Divider } from "#/material/divider";
import { MenuList } from "#/material/menu";

import { IMenubarItem, IMenubarSubmenu, isMenubarSubmenu, TMenubarEntry } from "./menubar-menu";
import { MenubarPopup } from "./MenubarPopup";
import { MenubarRow } from "./MenubarRow";
import { IMenubarSubmenuOpenOptions, MenubarSubmenuRow } from "./MenubarSubmenuRow";

/** What a list does with the keyboard as it appears: takes it on the list, on its first row, or leaves it. */
export type TMenubarListFocus = "list" | "first-row" | "none";

interface ISubmenuOpening extends IMenubarSubmenuOpenOptions {
  id: string;
}

/**
 * How long the pointer rests on a row before the submenu under it opens or the open one closes, so crossing a row on
 * the way into an open submenu leaves it open.
 */
const SUBMENU_HOVER_DELAY_MS: number = 200;

/**
 * @param sections - A list's sections.
 * @param id - The submenu row wanted.
 * @returns That row, or null once the list no longer holds it.
 */
function findSubmenu(sections: ReadonlyArray<ReadonlyArray<TMenubarEntry>>, id: string): Nullable<IMenubarSubmenu> {
  for (const entries of sections) {
    for (const entry of entries) {
      if (isMenubarSubmenu(entry) && entry.id === id) {
        return entry;
      }
    }
  }

  return null;
}

interface IMenubarListProps {
  /** Names the list for assistive technology. */
  label: string;
  sections: ReadonlyArray<ReadonlyArray<TMenubarEntry>>;
  focus: TMenubarListFocus;
  /** Puts this list away and hands the caret back to the row that opened it; unset for a menubar's own menu. */
  onClose?: () => void;
  /** Keys the list leaves to its menubar: the arrows across, Escape and Tab. */
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  onChoose: (item: IMenubarItem) => void;
}

/**
 * One open menu's rows, its sections set apart by dividers, with the submenu one row has open beside it: it opens when
 * the pointer rests on its row, on a click or on the right arrow, and closes on the left arrow or Escape.
 */
export function MenubarList({ label, sections, focus, onClose, onKeyDown, onChoose }: IMenubarListProps): ReactElement {
  const list = useRef<Nullable<HTMLUListElement>>(null);
  const rows = useRef<Map<string, HTMLElement>>(new Map());
  const hoverTimer = useRef<Nullable<ReturnType<typeof setTimeout>>>(null);

  const [opening, setOpening] = useState<Nullable<ISubmenuOpening>>(null);

  const isSubmenu: boolean = onClose !== undefined;
  const submenu: Nullable<IMenubarSubmenu> = opening ? findSubmenu(sections, opening.id) : null;
  const anchor: Nullable<HTMLElement> = submenu ? (rows.current.get(submenu.id) ?? null) : null;

  const cancelHover = useCallback(() => {
    if (hoverTimer.current !== null) {
      clearTimeout(hoverTimer.current);
      hoverTimer.current = null;
    }
  }, []);

  const openSubmenu = useCallback(
    (next: ISubmenuOpening) => {
      cancelHover();
      setOpening(next);
    },
    [cancelHover]
  );

  const closeSubmenu = useCallback(() => {
    cancelHover();
    setOpening(null);

    if (opening) {
      rows.current.get(opening.id)?.focus();
    }
  }, [cancelHover, opening]);

  const onRowPointerEnter = useCallback(
    (id: Nullable<string>) => {
      cancelHover();

      if ((opening?.id ?? null) !== id) {
        hoverTimer.current = setTimeout(
          () => setOpening(id === null ? null : { id, isFromKeyboard: false }),
          SUBMENU_HOVER_DELAY_MS
        );
      }
    },
    [cancelHover, opening]
  );

  // The caret leaving the row whose submenu is open closes that submenu, so it never shows beside another row.
  const onFocus = useCallback(
    (event: FocusEvent<HTMLElement>) => {
      if (opening && list.current?.contains(event.target) && event.target !== rows.current.get(opening.id)) {
        cancelHover();
        setOpening(null);
      }
    },
    [cancelHover, opening]
  );

  const onListKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      // A submenu's key reaches here through the tree it is drawn in, already answered or let pass by that submenu.
      if (!list.current?.contains(event.target as Node)) {
        onKeyDown?.(event);

        return;
      }

      if (isSubmenu && (event.key === "ArrowLeft" || event.key === "Escape")) {
        event.preventDefault();
        event.stopPropagation();
        onClose?.();

        return;
      }

      // The right arrow still steps to the menubar's next menu and Tab still puts the bar away; any other key is this
      // submenu's alone, so the list that opened it neither moves its own caret nor searches its own rows by it.
      if (isSubmenu && event.key !== "ArrowRight" && event.key !== "Tab") {
        event.stopPropagation();

        return;
      }

      onKeyDown?.(event);
    },
    [isSubmenu, onClose, onKeyDown]
  );

  // Rows and dividers as one flat list: `MenuList` walks its direct children to find the row to focus, and refuses a
  // fragment among them.
  const children: Array<ReactElement> = sections.flatMap((entries: ReadonlyArray<TMenubarEntry>, index: number) => [
    ...(index > 0 ? [<Divider key={`divider-${index}`} className={"my-1"} />] : []),
    ...entries.map((entry: TMenubarEntry) =>
      isMenubarSubmenu(entry) ? (
        <MenubarSubmenuRow
          key={entry.id}
          submenu={entry}
          rowRef={(element: Nullable<HTMLLIElement>) => {
            if (element) {
              rows.current.set(entry.id, element);
            } else {
              rows.current.delete(entry.id);
            }
          }}
          isOpen={opening?.id === entry.id}
          onOpen={(options: IMenubarSubmenuOpenOptions) => openSubmenu({ id: entry.id, ...options })}
          onPointerEnter={() => onRowPointerEnter(entry.id)}
        />
      ) : (
        <MenubarRow key={entry.id} item={entry} onChoose={onChoose} onPointerEnter={() => onRowPointerEnter(null)} />
      )
    ),
  ]);

  useEffect(() => cancelHover, [cancelHover]);

  return (
    <MenuList
      ref={list}
      // The list takes focus to hear the keys of a menu opened by pointing; the surface around it marks it, not a ring.
      className={"min-w-44 py-1 outline-none"}
      aria-label={label}
      autoFocus={focus === "list"}
      autoFocusItem={focus === "first-row"}
      dense={true}
      variant={"menu"}
      onFocus={onFocus}
      onKeyDown={onListKeyDown}
    >
      {children}

      {submenu && anchor ? (
        <MenubarPopup key={submenu.id} anchor={anchor} placement={"right-start"} onPointerEnter={cancelHover}>
          <MenubarList
            label={submenu.label}
            sections={submenu.sections}
            focus={opening?.isFromKeyboard ? "first-row" : "none"}
            onClose={closeSubmenu}
            onChoose={onChoose}
          />
        </MenubarPopup>
      ) : null}
    </MenuList>
  );
}
