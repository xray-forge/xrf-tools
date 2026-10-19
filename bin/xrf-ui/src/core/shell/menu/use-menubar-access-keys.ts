import { useEffect } from "react";

import { isModalOpen } from "@/lib/dom/modal";

/**
 * Opens the menubar from `F10`, or from `Alt` let go with no key, click or wheel turn while it was held. A surface that
 * reads `Alt` itself, such as a viewport, keeps it by stopping its `keydown`.
 *
 * @param onOpen - Opens the menubar with the keyboard in it.
 */
export function useMenubarAccessKeys(onOpen: () => void): void {
  useEffect(() => {
    let isAltAlone: boolean = false;

    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === "Alt") {
        isAltAlone = !event.repeat && !event.ctrlKey && !event.shiftKey && !event.metaKey;

        return;
      }

      isAltAlone = false;

      if (
        event.key === "F10" &&
        !event.defaultPrevented &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.shiftKey &&
        !event.metaKey &&
        !isModalOpen()
      ) {
        event.preventDefault();
        onOpen();
      }
    }

    function onKeyUp(event: KeyboardEvent): void {
      if (event.key !== "Alt" || !isAltAlone) {
        return;
      }

      isAltAlone = false;

      if (!isModalOpen()) {
        event.preventDefault();
        onOpen();
      }
    }

    function onInterrupt(): void {
      isAltAlone = false;
    }

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("pointerdown", onInterrupt);
    window.addEventListener("wheel", onInterrupt);
    window.addEventListener("blur", onInterrupt);

    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("pointerdown", onInterrupt);
      window.removeEventListener("wheel", onInterrupt);
      window.removeEventListener("blur", onInterrupt);
    };
  }, [onOpen]);
}
