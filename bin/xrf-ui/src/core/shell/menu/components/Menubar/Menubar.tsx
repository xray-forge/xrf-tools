import { ClickAwayListener, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { KeyboardEvent, ReactElement, useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { flushSync } from "react-dom";

import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { IMenubarItem, IMenubarMenu } from "./menubar-menu";
import { MenubarPopup } from "./MenubarPopup";

interface IMenubarOpening {
  index: number;
  /** A menu opened from the keyboard puts the caret on its first row; one opened by the pointer does not. */
  isFromKeyboard: boolean;
}

interface IMenubarProps extends BaseComponentProps {
  /** Names the bar for assistive technology. */
  label: string;
  menus: ReadonlyArray<IMenubarMenu>;
  /** Opens the first menu as the bar appears, for a bar shown by a click; one shown from the keyboard only marks it. */
  isFirstMenuOpen?: boolean;
  /** Puts the bar away: on Escape, Tab, a click elsewhere, the window losing focus, or a row chosen. */
  onCollapse: () => void;
}

/**
 * A desktop menubar that takes the keyboard as it appears and puts itself away through `onCollapse`.
 */
export function Menubar({
  "data-testid": dataTestId = "menubar",
  id,
  className,
  label,
  menus,
  isFirstMenuOpen = false,
  onCollapse,
}: IMenubarProps): ReactElement {
  const root = useRef<Nullable<HTMLDivElement>>(null);
  const buttons = useRef<Array<Nullable<HTMLElement>>>([]);
  const returnFocus = useRef<Nullable<Element>>(null);

  const [rawFocusIndex, setFocusIndex] = useState<number>(0);
  const [opening, setOpening] = useState<Nullable<IMenubarOpening>>(null);

  // Clamped to the menus there are now, since a tool can take its menu away while the bar is open, and a position past
  // the end would leave no button to hold the caret.
  const focusIndex: number = Math.min(rawFocusIndex, menus.length - 1);
  const openIndex: number = opening && opening.index < menus.length ? opening.index : -1;

  const openMenu: Nullable<IMenubarMenu> = openIndex < 0 ? null : menus[openIndex];
  const anchor: Nullable<HTMLElement> = openIndex < 0 ? null : (buttons.current[openIndex] ?? null);

  /**
   * Puts the bar away. Flushed so that whatever runs next - a chosen row's action, the focus handed back - finds the
   * bar gone, and with it the open menu every shortcut checks for.
   */
  const collapse = useCallback(
    (isFocusReturned: boolean) => {
      const target: Nullable<Element> = returnFocus.current;

      flushSync(onCollapse);

      if (isFocusReturned && target instanceof HTMLElement) {
        target.focus();
      }
    },
    [onCollapse]
  );

  const focusButton = useCallback(
    (index: number): number => {
      const wrapped: number = (index + menus.length) % menus.length;

      setFocusIndex(wrapped);
      buttons.current[wrapped]?.focus();

      return wrapped;
    },
    [menus.length]
  );

  const open = useCallback(
    (index: number, isFromKeyboard: boolean) => setOpening({ index: focusButton(index), isFromKeyboard }),
    [focusButton]
  );

  const close = useCallback(() => {
    if (openIndex >= 0) {
      focusButton(openIndex);
    }

    setOpening(null);
  }, [focusButton, openIndex]);

  const onChoose = useCallback(
    (item: IMenubarItem) => {
      // A switch keeps its menu open, so several can be turned in one visit, as a toolbar popover allows.
      if (item.isChecked === undefined) {
        collapse(false);
      }

      item.onSelect();
    },
    [collapse]
  );

  const onButtonClick = useCallback(
    (index: number) => (index === openIndex ? close() : open(index, false)),
    [close, open, openIndex]
  );

  // Pointing moves the caret, as a desktop menubar does, so the pointer and the keys never mark two menus.
  const onButtonPointerEnter = useCallback(
    (index: number) => {
      if (openIndex < 0) {
        focusButton(index);
      } else if (index !== openIndex) {
        open(index, false);
      }
    },
    [focusButton, open, openIndex]
  );

  const onButtonKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>, index: number) => {
      switch (event.key) {
        case "ArrowRight":
        case "ArrowLeft": {
          const next: number = index + (event.key === "ArrowRight" ? 1 : -1);

          if (openIndex >= 0) {
            open(next, true);
          } else {
            focusButton(next);
          }

          break;
        }
        case "ArrowDown":
          open(index, true);
          break;
        case "Home":
          focusButton(0);
          break;
        case "End":
          focusButton(menus.length - 1);
          break;
        case "Escape":
        case "Tab":
          collapse(true);
          break;
        default:
          return;
      }

      event.preventDefault();
      // The bar answers its own keys; nothing behind it should read them as shortcuts.
      event.stopPropagation();
    },
    [collapse, focusButton, menus.length, open, openIndex]
  );

  const onPopupKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (openIndex < 0) {
        return;
      }

      switch (event.key) {
        case "ArrowRight":
          open(openIndex + 1, true);
          break;
        case "ArrowLeft":
          open(openIndex - 1, true);
          break;
        case "Escape":
        case "Tab":
          collapse(true);
          break;
        default:
          // The list moves its own caret; a key it handled is still not one for the shortcuts behind it.
          event.stopPropagation();

          return;
      }

      event.preventDefault();
      event.stopPropagation();
    },
    [collapse, open, openIndex]
  );

  // Run once the buttons are mounted, since an open menu is placed under its button.
  useLayoutEffect(() => {
    returnFocus.current = document.activeElement;
    buttons.current[0]?.focus();

    if (isFirstMenuOpen) {
      setOpening({ index: 0, isFromKeyboard: false });
    }
  }, [isFirstMenuOpen]);

  // A menu leaving the bar takes its button and the caret with it; the caret comes back to the bar, or Escape would
  // reach nothing that can put it away.
  useLayoutEffect(() => {
    if (openIndex < 0 && !root.current?.contains(document.activeElement)) {
      buttons.current[focusIndex]?.focus();
    }
  }, [focusIndex, openIndex, menus.length]);

  // A native menu closes when its window loses focus; this one would otherwise wait open behind another application.
  useEffect(() => {
    function onBlur(): void {
      collapse(false);
    }

    window.addEventListener("blur", onBlur);

    return () => window.removeEventListener("blur", onBlur);
  }, [collapse]);

  return (
    <ClickAwayListener mouseEvent={"onPointerDown"} onClickAway={() => collapse(false)}>
      <div
        ref={root}
        data-testid={dataTestId}
        aria-label={label}
        id={id}
        className={cn("flex h-full shrink-0 items-center gap-0.5", className)}
        role={"menubar"}
      >
        {menus.map((menu: IMenubarMenu, index: number) => {
          const isOpen: boolean = openIndex === index;

          return (
            <Typography
              key={menu.id}
              ref={(element: Nullable<HTMLElement>) => {
                buttons.current[index] = element;
              }}
              aria-expanded={isOpen}
              aria-haspopup={"menu"}
              className={cn(
                "shrink-0 cursor-pointer appearance-none rounded-control border-0 bg-transparent px-2 py-0.5",
                // The caret is always placed by the bar itself, a click on the caption's menu button included, so the
                // current menu shows on focus and not only on a focus the browser deems keyboard-driven.
                isOpen
                  ? "bg-action-selected text-text-primary"
                  : "text-text-secondary hover:bg-action-hover focus:bg-action-hover focus:text-text-primary",
                "outline-none"
              )}
              component={"button"}
              noWrap={true}
              role={"menuitem"}
              tabIndex={index === focusIndex ? 0 : -1}
              type={"button"}
              variant={"body2"}
              onClick={() => onButtonClick(index)}
              onKeyDown={(event: KeyboardEvent<HTMLElement>) => onButtonKeyDown(event, index)}
              onPointerEnter={() => onButtonPointerEnter(index)}
            >
              {menu.label}
            </Typography>
          );
        })}

        {openMenu && anchor ? (
          <MenubarPopup
            key={openMenu.id}
            menu={openMenu}
            anchor={anchor}
            isFirstRowFocused={opening?.isFromKeyboard ?? false}
            onKeyDown={onPopupKeyDown}
            onChoose={onChoose}
          />
        ) : null}
      </div>
    </ClickAwayListener>
  );
}
