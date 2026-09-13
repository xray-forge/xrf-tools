import { XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

import { IMaterialStateDescriptor } from "./material-description";

/** The range an alpha reference is stated in, so a row reads `200/255` rather than a bare number. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * Wording and severity for what the renderer ends up drawing for a surface.
 *
 * The chip says what is drawn when that is known, and why it is not when it is not: a surface drawn opaque because
 * its shader says so and one drawn opaque because no library was found look identical in the viewport and are
 * opposite fixes.
 *
 * @param descriptor - What the backend resolved for the shader name.
 * @returns How the chip reads, and how loudly.
 */
export function describeSurfaceOutcome(descriptor: XraySurfaceDescriptor): IMaterialStateDescriptor {
  switch (descriptor.declaration.kind) {
    case "noLibrary":
      return { color: "warning", label: "No shader library" };

    case "unreadable":
      return { color: "error", label: "Library unreadable" };

    case "undefined":
      return { color: "warning", label: "Shader undefined" };

    case "unmodelled":
      return { color: "warning", label: "Class not modelled" };

    case "described":
      return describeDrawState(descriptor.draw);
  }
}

/**
 * What the surface is drawn as, in the renderer's terms.
 *
 * @param draw - The pass the backend resolved.
 * @returns A line for the row, or null for a surface that reads no alpha and has nothing to add to its chip.
 */
export function describeSurfaceDraw(draw: XraySurfaceDraw): Nullable<string> {
  switch (draw.kind) {
    case "opaque":
      return null;

    case "alphaTested":
      return (
        `killed below ${draw.reference}/${ALPHA_REFERENCE_SCALE} · the deferred pixel shader's own constant, ` +
        "not the authored reference"
      );

    case "blended":
      return (
        `source alpha over the background, killed below ${draw.reference}/${ALPHA_REFERENCE_SCALE} · ` +
        "depth tested and not written"
      );
  }
}

/**
 * What the library says, or why it says nothing.
 *
 * @param declaration - What the backend read.
 * @param library - The `shaders.xr` it read, when one was located.
 * @returns A line for the row.
 */
export function describeSurfaceDeclaration(declaration: XraySurfaceDeclaration, library: Nullable<XrayAsset>): string {
  const source: string = library ? library.logicalPath : "shaders.xr";

  switch (declaration.kind) {
    case "noLibrary":
      return "no searched root holds shaders.xr, so nothing is known about this surface";

    case "unreadable":
      return `${source} could not be read: ${declaration.reason}`;

    case "undefined":
      return `${source} defines no shader of this name, so the engine falls back to its default shader`;

    case "unmodelled":
      return `${source}, class '${declaration.class}' · this viewer derives no pass from that class`;

    case "described":
      return [
        source,
        `class '${declaration.class}'`,
        ...describeAlphaKnobs(declaration.isAlphaUsed, declaration.alphaReference),
        ...(declaration.isStrictSorting ? ["strict sorting"] : []),
      ].join(" · ");
  }
}

/**
 * Where this viewer's drawing falls short of the game's for this surface, stated rather than approximated silently.
 *
 * @param descriptor - What the backend resolved.
 * @returns A line for the row, or null when the viewer draws what it reports.
 */
export function describeSurfaceShading(descriptor: XraySurfaceDescriptor): Nullable<string> {
  if (descriptor.draw.kind !== "blended") {
    return null;
  }

  return (
    "In this viewer, blended surfaces are sorted by distance from the camera rather than by the engine's own pass " +
    "order, so two of them crossing may composite in the other order"
  );
}

/** The chip for a surface whose blender was read: what it draws as. */
function describeDrawState(draw: XraySurfaceDraw): IMaterialStateDescriptor {
  switch (draw.kind) {
    case "opaque":
      return { color: "default", label: "Opaque" };

    case "alphaTested":
      return { color: "success", label: "Cut out" };

    case "blended":
      return { color: "success", label: "Blended" };
  }
}

/** The two authored knobs, each only when the class writes it. */
function describeAlphaKnobs(isAlphaUsed: Nullable<boolean>, reference: Nullable<number>): Array<string> {
  const knobs: Array<string> = [];

  if (isAlphaUsed !== null) {
    knobs.push(isAlphaUsed ? "alpha channel used" : "alpha channel unused");
  }

  if (reference !== null) {
    knobs.push(`alpha ref ${reference}`);
  }

  return knobs;
}
