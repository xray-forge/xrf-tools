import { Nullable } from "@xrf/types";

import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** The window's active pane: the viewport last pressed in, or the first attached while none was. */
let active: Nullable<NativeViewport> = null;

/** Whether the window's focus is followed yet. */
let isFollowingFocus: boolean = false;

/** Hears the active pane again as its window takes the focus, the app hearing only the focused window's. */
function onWindowFocus(): void {
  active?.listen();
}

/**
 * Hears a viewport's scene while it is the focused window's active pane: as it is pressed in, as its window takes the
 * focus, and as it attaches while no pane of the window is active.
 *
 * @param viewport - The viewport, just attached.
 * @param container - The element it draws under, which a press makes the active pane.
 * @returns What stops following it.
 */
export function followListening(viewport: NativeViewport, container: HTMLElement): () => void {
  function activate(): void {
    active = viewport;
    viewport.listen();
  }

  if (!isFollowingFocus) {
    isFollowingFocus = true;
    window.addEventListener("focus", onWindowFocus);
  }

  if (!active) {
    activate();
  }

  container.addEventListener("pointerdown", activate);

  return () => {
    container.removeEventListener("pointerdown", activate);

    if (active === viewport) {
      active = null;
    }
  };
}
