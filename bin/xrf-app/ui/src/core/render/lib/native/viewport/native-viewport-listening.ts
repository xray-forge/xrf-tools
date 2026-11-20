import { Nullable } from "@xrf/types";

import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** The window's active pane: the viewport last pressed in, the first attached while none was, or the latest left. */
let active: Nullable<NativeViewport> = null;

/** The window's attached viewports, in the order they attached. */
const attached: Array<NativeViewport> = [];

/** Whether the window's focus is followed yet. */
let isFollowingFocus: boolean = false;

/** Hears the active pane again as its window takes the focus, the app hearing only the focused window's. */
function onWindowFocus(): void {
  active?.listen();
}

/** Makes a viewport the active pane and hears its scene. */
function activate(viewport: NativeViewport): void {
  active = viewport;
  viewport.listen();
}

/**
 * Hears a viewport's scene while it is the focused window's active pane. It becomes active when pressed in, when it
 * attaches while no pane is active, and when the active pane goes, the latest attached taking over.
 *
 * @param viewport - The viewport, just attached.
 * @param container - The element it draws under, which a press makes the active pane.
 * @returns What stops following it.
 */
export function followListening(viewport: NativeViewport, container: HTMLElement): () => void {
  function onPress(): void {
    activate(viewport);
  }

  if (!isFollowingFocus) {
    isFollowingFocus = true;
    window.addEventListener("focus", onWindowFocus);
  }

  attached.push(viewport);

  if (!active) {
    activate(viewport);
  }

  container.addEventListener("pointerdown", onPress);

  return () => {
    container.removeEventListener("pointerdown", onPress);
    attached.splice(attached.indexOf(viewport), 1);

    if (active === viewport) {
      const next: Nullable<NativeViewport> = attached.at(-1) ?? null;

      active = null;

      if (next) {
        activate(next);
      }
    }
  };
}
