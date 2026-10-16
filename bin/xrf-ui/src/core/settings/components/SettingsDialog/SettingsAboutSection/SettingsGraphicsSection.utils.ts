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

/** The attribution NVIDIA's DLSS licence requires in the About box. */
export const DLSS_ATTRIBUTION: string =
  "Upscaling and ray reconstruction by NVIDIA DLSS. NVIDIA and DLSS are trademarks and/or registered trademarks of " +
  "NVIDIA Corporation in the U.S. and other countries.";

/**
 * @param backend - One graphics API as the renderer probed it.
 * @returns The GPU and driver it would use, or why it is unavailable.
 */
function describeAvailability(backend: RenderBackendAvailability): string {
  if (backend.adapter === null) {
    return `Not available: ${backend.problem ?? "no adapter"}`;
  }

  return backend.driver ? `${backend.adapter}, driver ${backend.driver}` : backend.adapter;
}

/**
 * @param choice - The graphics API chosen in the settings.
 * @param availability - Each graphics API as the renderer probed it.
 * @returns The API in use, and why it was picked.
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
    return `${describeRenderBackend(drawn)}, set by XRF_RENDER_BACKEND`;
  }

  return choice === RENDER_BACKEND_AUTO
    ? `${describeRenderBackend(drawn)}, chosen automatically`
    : describeRenderBackend(drawn);
}

/**
 * @param choice - The graphics API chosen in the settings.
 * @param availability - Each graphics API as the renderer probed it, or null until it answers.
 * @param libraries - The graphics libraries the renderer ships with, or null until it answers.
 * @returns The rows to show, in display order.
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
