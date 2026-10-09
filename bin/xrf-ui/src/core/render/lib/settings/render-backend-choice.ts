import { Nullable } from "@xrf/types";

import { ERenderBackend, RenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";

/** The choice that names no backend: the renderer starts on the first that works. */
export const RENDER_BACKEND_AUTO = "auto";

/** Which graphics API the settings ask the renderer for: one by name, or whichever works first. */
export type TRenderBackendChoice = typeof RENDER_BACKEND_AUTO | ERenderBackend;

/** Each backend's name as the settings and the Renderer panel show it. */
const RENDER_BACKEND_NAMES: Readonly<Record<RenderBackend, string>> = {
  [ERenderBackend.D3D12]: "Direct3D 12",
  [ERenderBackend.VULKAN]: "Vulkan",
};

/** Each backend's short tag as a viewport's readout shows it. */
const RENDER_BACKEND_TAGS: Readonly<Record<RenderBackend, string>> = {
  [ERenderBackend.D3D12]: "dx",
  [ERenderBackend.VULKAN]: "vk",
};

/** The choices offered, in the renderer's own fallback order after the automatic one. */
export const RENDER_BACKEND_CHOICES: ReadonlyArray<TRenderBackendChoice> = [
  RENDER_BACKEND_AUTO,
  ERenderBackend.D3D12,
  ERenderBackend.VULKAN,
];

/**
 * Reads a stored choice, falling back to the automatic one rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns One of the offered choices.
 */
export function toRenderBackendChoice(stored: unknown): TRenderBackendChoice {
  return RENDER_BACKEND_CHOICES.find((choice: TRenderBackendChoice) => choice === stored) ?? RENDER_BACKEND_AUTO;
}

/**
 * @param choice - What the settings ask for.
 * @returns The backend the renderer is asked for, none for whichever works first.
 */
export function toRenderBackend(choice: TRenderBackendChoice): Nullable<ERenderBackend> {
  return choice === RENDER_BACKEND_AUTO ? null : choice;
}

/**
 * @param backend - A backend.
 * @param availability - What each backend can do on this machine, as the renderer probed it.
 * @returns Whether the renderer can draw with it here.
 */
export function isRenderBackendAvailable(
  backend: RenderBackend,
  availability: ReadonlyArray<RenderBackendAvailability>
): boolean {
  return availability.some((it: RenderBackendAvailability) => it.backend === backend && it.adapter !== null);
}

/**
 * The backend the renderer draws with for a choice, as it falls back: the one asked for where it is available, else
 * the first available in the renderer's order; none where none is.
 *
 * @param choice - What the settings ask for.
 * @param availability - What each backend can do on this machine, in the renderer's order.
 * @returns The backend drawn with.
 */
export function resolveRenderBackend(
  choice: TRenderBackendChoice,
  availability: ReadonlyArray<RenderBackendAvailability>
): Nullable<RenderBackend> {
  const asked: Nullable<ERenderBackend> = toRenderBackend(choice);

  if (asked !== null && isRenderBackendAvailable(asked, availability)) {
    return asked;
  }

  const first: RenderBackendAvailability | undefined = availability.find(
    (it: RenderBackendAvailability) => it.adapter !== null
  );

  return first ? first.backend : null;
}

/**
 * @param backend - A backend.
 * @returns Its name as the settings show it.
 */
export function describeRenderBackend(backend: RenderBackend): string {
  return RENDER_BACKEND_NAMES[backend];
}

/**
 * @param backend - A backend.
 * @returns Its short tag as a viewport's readout shows it.
 */
export function abbreviateRenderBackend(backend: RenderBackend): string {
  return RENDER_BACKEND_TAGS[backend];
}
