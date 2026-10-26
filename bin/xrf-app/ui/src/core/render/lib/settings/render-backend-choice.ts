import {
  ERenderBackend,
  RenderBackend,
  RenderBackendAvailability,
  RenderGpu,
  RenderGpuDescription,
} from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { resolveRenderGpu } from "@/core/render/lib/settings/render-gpu-choice";

/** The choice that names no backend: the renderer starts on the first that works. */
export const RENDER_BACKEND_AUTO = "auto";

/** Which graphics API the settings ask the renderer for: one by name, or whichever works first. */
export type TRenderBackendChoice = typeof RENDER_BACKEND_AUTO | ERenderBackend;

/** Each backend's name as the settings and the Renderer panel show it. */
const RENDER_BACKEND_NAMES: Readonly<Record<RenderBackend, string>> = {
  [ERenderBackend.D3D12]: "Direct3D 12",
  [ERenderBackend.VULKAN]: "Vulkan",
  [ERenderBackend.METAL]: "Metal",
};

/** Each backend's short tag as a viewport's readout shows it. */
const RENDER_BACKEND_TAGS: Readonly<Record<RenderBackend, string>> = {
  [ERenderBackend.D3D12]: "dx",
  [ERenderBackend.VULKAN]: "vk",
  [ERenderBackend.METAL]: "mtl",
};

/** Every choice the settings can hold, whichever platform they were made on, the automatic one first. */
const RENDER_BACKEND_CHOICES: ReadonlyArray<TRenderBackendChoice> = [
  RENDER_BACKEND_AUTO,
  ERenderBackend.D3D12,
  ERenderBackend.VULKAN,
  ERenderBackend.METAL,
];

/**
 * The choices the settings offer: the automatic one, every backend the renderer probed on this platform, and the one
 * asked for even where it is not among them, so the row still shows it.
 *
 * @param availability - What each backend can do on this machine, as the renderer probed it; none until known.
 * @param value - What the settings ask for.
 * @returns The choices offered, in the renderer's order after the automatic one.
 */
export function listRenderBackendChoices(
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>,
  value: TRenderBackendChoice
): ReadonlyArray<TRenderBackendChoice> {
  return RENDER_BACKEND_CHOICES.filter(
    (choice: TRenderBackendChoice) =>
      choice === RENDER_BACKEND_AUTO ||
      choice === value ||
      Boolean(availability?.some((it: RenderBackendAvailability) => it.backend === choice))
  );
}

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
  return availability.some((it: RenderBackendAvailability) => it.backend === backend && it.gpus.length > 0);
}

/**
 * @param availability - What each backend can do on this machine, as the renderer probed it.
 * @returns The backend `XRF_RENDER_BACKEND` names, which the renderer starts on over the settings' choice; none where
 *   it names none.
 */
export function getForcedRenderBackend(
  availability: ReadonlyArray<RenderBackendAvailability>
): Nullable<RenderBackend> {
  return availability.find((it: RenderBackendAvailability) => it.isForced)?.backend ?? null;
}

/**
 * The backend the renderer draws with for a choice, as it falls back: the one the environment forces or else the one
 * asked for, where it is available; else the first available in the renderer's order; none where none is.
 *
 * @param choice - What the settings ask for.
 * @param availability - What each backend can do on this machine, in the renderer's order.
 * @returns The backend drawn with.
 */
export function resolveRenderBackend(
  choice: TRenderBackendChoice,
  availability: ReadonlyArray<RenderBackendAvailability>
): Nullable<RenderBackend> {
  const asked: Nullable<RenderBackend> = getForcedRenderBackend(availability) ?? toRenderBackend(choice);

  if (asked !== null && isRenderBackendAvailable(asked, availability)) {
    return asked;
  }

  const first: RenderBackendAvailability | undefined = availability.find(
    (it: RenderBackendAvailability) => it.gpus.length > 0
  );

  return first ? first.backend : null;
}

/** The graphics API the renderer draws with and the GPU it draws on. */
export interface IRenderDrawing {
  backend: RenderBackend;
  /** The GPU, or null where the API names none for the choice. */
  gpu: Nullable<RenderGpuDescription>;
}

/**
 * @param choice - The graphics API the settings ask for.
 * @param gpu - The GPU the settings ask for, or null for the automatic one.
 * @param availability - What each backend can do on this machine, in the renderer's order.
 * @returns What draws, as the renderer resolves the choice; null where no graphics API works here.
 */
export function resolveRenderDrawing(
  choice: TRenderBackendChoice,
  gpu: Nullable<RenderGpu>,
  availability: ReadonlyArray<RenderBackendAvailability>
): Nullable<IRenderDrawing> {
  const backend: Nullable<RenderBackend> = resolveRenderBackend(choice, availability);

  if (backend === null) {
    return null;
  }

  return {
    backend,
    gpu: resolveRenderGpu(
      gpu,
      availability.find((it: RenderBackendAvailability) => it.backend === backend)
    ),
  };
}

/**
 * @param drawing - What draws.
 * @returns The API and the GPU by name.
 */
export function describeRenderDrawing(drawing: IRenderDrawing): string {
  return `${describeRenderBackend(drawing.backend)} on ${drawing.gpu?.name ?? "the GPU"}`;
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
