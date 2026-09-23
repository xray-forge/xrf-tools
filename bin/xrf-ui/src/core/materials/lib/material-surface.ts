import { assertExhaustive, Nullable } from "@xrf/types";

import {
  EXraySurfaceDeclaration,
  EXraySurfaceDraw,
  XraySurfaceDeclaration,
  XraySurfaceDescriptor,
  XraySurfaceDraw,
} from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";

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
    case EXraySurfaceDeclaration.UNDECLARED:
      return { color: "default", label: "No shader" };

    case EXraySurfaceDeclaration.NO_LIBRARY:
      return { color: "warning", label: "No shader library" };

    case EXraySurfaceDeclaration.UNREADABLE:
      return { color: "error", label: "Library unreadable" };

    case EXraySurfaceDeclaration.UNDEFINED:
      return { color: "warning", label: "Shader undefined" };

    case EXraySurfaceDeclaration.UNMODELLED:
      return { color: "warning", label: "Class not modelled" };

    case EXraySurfaceDeclaration.SCRIPTED:
    case EXraySurfaceDeclaration.DESCRIBED:
      return describeDrawState(descriptor.draw);

    default:
      return assertExhaustive(descriptor.declaration);
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
    case EXraySurfaceDraw.OPAQUE:
      return null;

    case EXraySurfaceDraw.ALPHA_TESTED:
      return (
        `killed below ${draw.reference}/${ALPHA_REFERENCE_SCALE} · the deferred pixel shader's own constant, ` +
        "not the authored reference"
      );

    case EXraySurfaceDraw.BLENDED:
      return (
        `source alpha over the background, killed below ${draw.reference}/${ALPHA_REFERENCE_SCALE} · ` +
        "depth tested and not written"
      );

    case EXraySurfaceDraw.ADDED:
      return (
        `${draw.isWeighted ? "added to the background by its alpha" : "added whole to the background"}, ` +
        `killed below ${draw.reference}/${ALPHA_REFERENCE_SCALE} · depth tested and not written`
      );

    case EXraySurfaceDraw.MULTIPLIED:
      return draw.isDoubled
        ? "multiplied both ways into the background, doubling it · depth tested and not written"
        : "multiplied into the background · depth tested and not written";

    case EXraySurfaceDraw.INVISIBLE:
      return "the background times one and the surface times zero · submitted, drawn, and contributing nothing";

    default:
      return assertExhaustive(draw);
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
    case EXraySurfaceDeclaration.UNDECLARED:
      return "nothing is dressed by this entry, so no shader was looked up for it";

    case EXraySurfaceDeclaration.NO_LIBRARY:
      return "no searched root holds shaders.xr, so nothing is known about this surface";

    case EXraySurfaceDeclaration.UNREADABLE:
      return `${source} could not be read: ${declaration.reason}`;

    case EXraySurfaceDeclaration.UNDEFINED:
      return `${source} defines no shader of this name, so the engine falls back to its default shader`;

    case EXraySurfaceDeclaration.UNMODELLED:
      return `${source}, class '${declaration.class}' · this viewer derives no pass from that class`;

    case EXraySurfaceDeclaration.SCRIPTED:
      return [
        `${declaration.script}, function '${declaration.function}'`,
        // The whole point of saying so: a shader with a script is that script, and whatever class shaders.xr gives
        // the same name never reaches the screen.
        "read instead of the blender library, as the engine reads it",
        declaration.isBlended ? "composited" : "written",
        ...(declaration.isAlphaTested ? ["alpha tested"] : []),
        declaration.isDepthWritten ? "depth written" : "depth not written",
        ...(declaration.isWallmark ? ["wall mark"] : []),
      ].join(" · ");

    case EXraySurfaceDeclaration.DESCRIBED:
      return [
        source,
        `class '${declaration.class}'`,
        ...describeAlphaKnobs(declaration.isAlphaUsed, declaration.alphaReference),
        ...(declaration.isStrictSorting ? ["strict sorting"] : []),
      ].join(" · ");

    default:
      return assertExhaustive(declaration);
  }
}

/**
 * Where this viewer's drawing falls short of the game's for this surface, stated rather than approximated silently.
 *
 * @param descriptor - What the backend resolved.
 * @returns A line for the row, or null when the viewer draws what it reports.
 */
export function describeSurfaceShading(descriptor: XraySurfaceDescriptor): Nullable<string> {
  if (descriptor.draw.kind !== EXraySurfaceDraw.BLENDED) {
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
    case EXraySurfaceDraw.OPAQUE:
      return { color: "default", label: "Opaque" };

    case EXraySurfaceDraw.ALPHA_TESTED:
      return { color: "success", label: "Cut out" };

    case EXraySurfaceDraw.BLENDED:
      return { color: "success", label: "Blended" };

    case EXraySurfaceDraw.ADDED:
      return { color: "success", label: "Added" };

    case EXraySurfaceDraw.MULTIPLIED:
      return { color: "success", label: draw.isDoubled ? "Multiplied 2x" : "Multiplied" };

    case EXraySurfaceDraw.INVISIBLE:
      return { color: "default", label: "Draws nothing" };

    default:
      return assertExhaustive(draw);
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
