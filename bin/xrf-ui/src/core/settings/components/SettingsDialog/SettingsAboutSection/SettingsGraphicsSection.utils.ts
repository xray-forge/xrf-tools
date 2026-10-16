import { Nullable } from "@xrf/types";

import {
  RenderBackend,
  RenderBackendAvailability,
  RenderGpu,
  RenderGpuDescription,
  RenderLibraries,
} from "@/core/ipc/types/xrf-renderer";
import {
  describeRenderBackend,
  getForcedRenderBackend,
  RENDER_BACKEND_AUTO,
  resolveRenderBackend,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import { describeRenderGpu, resolveRenderGpu } from "@/core/render/lib/settings/render-gpu-choice";

import { IAboutRow } from "./about-row";

/** The attribution NVIDIA's DLSS licence requires in the About box. */
export const DLSS_ATTRIBUTION: string =
  "Upscaling and ray reconstruction by NVIDIA DLSS. NVIDIA and DLSS are trademarks and/or registered trademarks of " +
  "NVIDIA Corporation in the U.S. and other countries.";

/**
 * @param backend - One graphics API as the renderer probed it.
 * @returns Every GPU it can use with its driver, or why it is unavailable.
 */
function describeAvailability(backend: RenderBackendAvailability): string {
  if (backend.gpus.length === 0) {
    return `Not available: ${backend.problem ?? "no adapter"}`;
  }

  return backend.gpus.map((it: RenderGpuDescription) => describeRenderGpu(it)).join("; ");
}

/**
 * @param choice - The graphics API chosen in the settings.
 * @param gpu - The GPU chosen in the settings, or null for the automatic one.
 * @param availability - Each graphics API as the renderer probed it.
 * @returns The API and GPU in use, and why the API was picked.
 */
function describeDrawing(
  choice: TRenderBackendChoice,
  gpu: Nullable<RenderGpu>,
  availability: ReadonlyArray<RenderBackendAvailability>
): string {
  const drawn: Nullable<RenderBackend> = resolveRenderBackend(choice, availability);

  if (drawn === null) {
    return "No graphics API works on this machine";
  }

  const drawnGpu: Nullable<RenderGpuDescription> = resolveRenderGpu(
    gpu,
    availability.find((it: RenderBackendAvailability) => it.backend === drawn)
  );
  const drawing: string = `${describeRenderBackend(drawn)} on ${drawnGpu?.name ?? "the GPU"}`;

  if (getForcedRenderBackend(availability) === drawn) {
    return `${drawing}, set by XRF_RENDER_BACKEND`;
  }

  return choice === RENDER_BACKEND_AUTO ? `${drawing}, chosen automatically` : drawing;
}

/**
 * @param choice - The graphics API chosen in the settings.
 * @param gpu - The GPU chosen in the settings, or null for the automatic one.
 * @param availability - Each graphics API as the renderer probed it, or null until it answers.
 * @param libraries - The graphics libraries the renderer ships with, or null until it answers.
 * @returns The rows to show, in display order.
 */
export function describeGraphics(
  choice: TRenderBackendChoice,
  gpu: Nullable<RenderGpu>,
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>,
  libraries: Nullable<RenderLibraries>
): Array<IAboutRow> {
  const rows: Array<IAboutRow> = [];

  if (availability !== null) {
    rows.push(
      ...availability.map((it: RenderBackendAvailability) => ({
        label: describeRenderBackend(it.backend),
        value: describeAvailability(it),
      })),
      { label: "Drawing with", value: describeDrawing(choice, gpu, availability) }
    );
  }

  if (libraries !== null) {
    rows.push({ label: "DLSS", value: libraries.dlss ?? "Not in this build" });
  }

  return rows;
}
