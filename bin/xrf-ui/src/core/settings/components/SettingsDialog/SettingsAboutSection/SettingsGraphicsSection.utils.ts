import { Nullable } from "@xrf/types";

import { RenderBackend, RenderBackendAvailability, RenderLibraries } from "@/core/ipc/types/xrf-renderer";
import {
  describeRenderBackend,
  getForcedRenderBackend,
  RENDER_BACKEND_AUTO,
  resolveRenderBackend,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";

import { IAboutRow } from "./about-row";

/** What NVIDIA's licence asks the application to say of the SDK it upscales with, beside its marks. */
export const DLSS_ATTRIBUTION: string =
  "Upscaling and ray reconstruction by NVIDIA DLSS. NVIDIA and DLSS are trademarks and/or registered trademarks of " +
  "NVIDIA Corporation in the U.S. and other countries.";

/**
 * @param backend - One graphics API as the renderer probed it.
 * @returns What it would draw on, with its driver, or why it cannot draw.
 */
function describeAvailability(backend: RenderBackendAvailability): string {
  if (backend.adapter === null) {
    return `Not available: ${backend.problem ?? "no adapter"}`;
  }

  return backend.driver ? `${backend.adapter}, driver ${backend.driver}` : backend.adapter;
}

/**
 * @param choice - The graphics API the settings ask for.
 * @param availability - Each graphics API as the renderer probed it.
 * @returns Which one draws, and why that one.
 */
function describeDrawing(
  choice: TRenderBackendChoice,
  availability: ReadonlyArray<RenderBackendAvailability>
): Nullable<string> {
  const drawn: Nullable<RenderBackend> = resolveRenderBackend(choice, availability);

  if (drawn === null) {
    return "No graphics API works on this machine";
  }

  if (getForcedRenderBackend(availability) === drawn) {
    return `${describeRenderBackend(drawn)}, as XRF_RENDER_BACKEND asks`;
  }

  return choice === RENDER_BACKEND_AUTO
    ? `${describeRenderBackend(drawn)}, the first that works`
    : describeRenderBackend(drawn);
}

/**
 * @param choice - The graphics API the settings ask for.
 * @param availability - Each graphics API as the renderer probed it, none until it answers.
 * @param libraries - The graphics libraries the renderer ships with, none until it answers.
 * @returns Every row worth stating about the GPU and the graphics libraries, in the order they read.
 */
export function describeGraphics(
  choice: TRenderBackendChoice,
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>,
  libraries: Nullable<RenderLibraries>
): Array<IAboutRow> {
  const rows: Array<IAboutRow> = [];

  if (availability !== null) {
    rows.push(
      ...availability.map((it: RenderBackendAvailability) => ({
        label: describeRenderBackend(it.backend),
        value: describeAvailability(it),
      }))
    );

    const drawing: Nullable<string> = describeDrawing(choice, availability);

    if (drawing) {
      rows.push({ label: "Drawing with", value: drawing });
    }
  }

  if (libraries !== null) {
    rows.push({ label: "DLSS", value: libraries.dlss ?? "Not in this build" });
  }

  return rows;
}
