import { Nullable } from "@xrf/types";

import { RenderBackendAvailability, RenderGpu, RenderGpuDescription } from "@/core/ipc/types/xrf-renderer";

/** Row value for the automatic GPU: the graphics API's high-performance GPU. */
export const RENDER_GPU_AUTO = "auto";

/** A GPU choice as the settings row names it: automatic, or a GPU by its `vendor:device` key. */
export type TRenderGpuKey = string;

/**
 * Reads a stored GPU choice, falling back to the automatic one rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, parsed from JSON.
 * @returns The GPU asked for, or null for the automatic one.
 */
export function toRenderGpuChoice(stored: unknown): Nullable<RenderGpu> {
  if (typeof stored !== "object" || stored === null) {
    return null;
  }

  const { vendor, device } = stored as Partial<RenderGpu>;

  return Number.isInteger(vendor) && Number.isInteger(device) ? { vendor: vendor!, device: device! } : null;
}

/**
 * @param gpu - A GPU, or null for the automatic one.
 * @returns Its key for the settings row.
 */
export function toRenderGpuKey(gpu: Nullable<RenderGpu>): TRenderGpuKey {
  return gpu === null ? RENDER_GPU_AUTO : `${gpu.vendor}:${gpu.device}`;
}

/**
 * @param key - A key of the settings row.
 * @param gpus - The GPUs the row offers.
 * @returns The GPU it names, or null for the automatic one.
 */
export function fromRenderGpuKey(key: TRenderGpuKey, gpus: ReadonlyArray<RenderGpuDescription>): Nullable<RenderGpu> {
  return gpus.find((it: RenderGpuDescription) => toRenderGpuKey(it.gpu) === key)?.gpu ?? null;
}

/**
 * The GPU the renderer uses for a choice: the chosen one if the backend has it, otherwise the automatic one (listed
 * first).
 *
 * @param choice - The GPU chosen in the settings, or null for the automatic one.
 * @param availability - The backend as the renderer probed it.
 * @returns The GPU drawn on, or null where the backend has none.
 */
export function resolveRenderGpu(
  choice: Nullable<RenderGpu>,
  availability: RenderBackendAvailability | undefined
): Nullable<RenderGpuDescription> {
  const gpus: ReadonlyArray<RenderGpuDescription> = availability?.gpus ?? [];

  return gpus.find((it: RenderGpuDescription) => toRenderGpuKey(it.gpu) === toRenderGpuKey(choice)) ?? gpus[0] ?? null;
}

/**
 * @param gpu - A GPU as the renderer probed it.
 * @returns Its name, and its driver where the API reports one.
 */
export function describeRenderGpu(gpu: RenderGpuDescription): string {
  return gpu.driver ? `${gpu.name}, driver ${gpu.driver}` : gpu.name;
}
