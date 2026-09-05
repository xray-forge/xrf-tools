import { describeResolution, getLocatedAsset } from "@/core/assets/lib/resolution";
import {
  XrayBumpMode,
  XrayBumpOutcome,
  XrayDetailUsage,
  XrayMaterialBumpInput,
  XrayMaterialDeclaration,
  XrayMaterialDescriptor,
  XrayMaterialDetail,
} from "@/core/bindings/types/xrf-material";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

import { describeTextureState, IVisualTextureStateDescriptor } from "./VisualSubmeshTexture.utils";

/**
 * Wording and severity for what the renderer ends up drawing for a material.
 *
 * @param outcome - What the backend resolved the declaration to.
 * @returns How the chip reads, and how loudly.
 */
export function describeBumpOutcome(outcome: XrayBumpOutcome): IVisualTextureStateDescriptor {
  switch (outcome) {
    case "flat":
      return { color: "default", label: "Flat" };

    case "bumped":
      return { color: "success", label: "Bumped" };

    case "dummy":
      return { color: "warning", label: "Dummy bump" };

    case "missing":
      return { color: "error", label: "Bump missing" };
  }
}

/**
 * The bump mode in the engine's own words.
 *
 * @param mode - Mode the declaration selects.
 * @returns A short label.
 */
export function describeBumpMode(mode: XrayBumpMode): string {
  return mode === "parallax" ? "Use parallax" : "Use";
}

/**
 * States why a material is flat when it is flat for a reason other than having no descriptor, or what it declares.
 *
 * @param declaration - What the backend read.
 * @param descriptor - The descriptor file it read, when one was located.
 * @returns A line for the row, or null when there is nothing worth a row.
 */
export function describeBumpDeclaration(
  declaration: XrayMaterialDeclaration,
  descriptor: Nullable<XrayAsset>
): Nullable<string> {
  const source: string = descriptor ? descriptor.logicalPath : "descriptor";

  switch (declaration.kind) {
    case "noDescriptor":
      return null;

    case "unreadable":
      return `${source} could not be read: ${declaration.reason}`;

    case "typeDisqualified":
      return declaration.declaredBump
        ? `${source} declares '${declaration.declaredBump}', but its type '${declaration.label}' is skipped by the ` +
            "engine"
        : `${source} has type '${declaration.label}', which the engine skips`;

    case "noBumpChunk":
      return `${source} carries no bump chunk`;

    case "disabled":
      return `${source} sets bump mode to none`;

    case "emptyName":
      return `${source} asks for '${describeBumpMode(declaration.mode)}' with an empty bump name`;

    case "declared":
      return `${source}, mode '${describeBumpMode(declaration.mode)}'`;
  }
}

/**
 * One bound bump input: what the declaration asked for, and what the renderer puts on the surface.
 *
 * @param input - The input as the backend resolved it.
 * @returns A line for the row.
 */
export function describeBumpInput(input: XrayMaterialBumpInput): string {
  const located: Nullable<XrayAsset> = getLocatedAsset(input.resolution);
  const line: string = `${input.reference} · ${describeResolution(input.resolution)}`;

  return input.resolution.kind === "substituted" && located ? `${line} (${located.logicalPath})` : line;
}

/**
 * The authored virtual height, with what it does not do.
 *
 * @param virtualHeight - Value from the bump chunk, null when non-finite.
 * @returns A line for the row.
 */
export function describeVirtualHeight(virtualHeight: Nullable<number>): string {
  return `${formatNumber(virtualHeight, 3)} m · authoring only, not read by the renderer`;
}

/**
 * How a detail texture is applied, in the engine's terms.
 *
 * @param usage - The two detail flags, or null when neither is set.
 * @returns A short label.
 */
function describeDetailUsage(usage: Nullable<XrayDetailUsage>): string {
  switch (usage) {
    case "diffuse":
      return "diffuse";

    case "bump":
      return "bump";

    case "diffuseAndBump":
      return "diffuse and bump";

    case null:
      return "not applied, no usage flag is set";
  }
}

/**
 * The detail association in one line: what, how large, and whether the engine applies it.
 *
 * @param detail - The association the descriptor names.
 * @returns A line for the row.
 */
export function describeDetail(detail: XrayMaterialDetail): string {
  return `${detail.name} · ×${formatNumber(detail.scale, 1)} · ${describeDetailUsage(detail.usage)}`;
}

/**
 * Where the viewer's shading falls short of the game's for this material, stated rather than approximated silently.
 *
 * @param material - What the backend resolved.
 * @returns A line for the row, or null when the viewer draws exactly what it reports.
 */
export function describeBumpShading(material: XrayMaterialDescriptor): Nullable<string> {
  const gaps: Array<string> = [];

  if (material.bump?.mode === "parallax") {
    gaps.push("parallax is drawn as plain bump");
  }

  if (material.detail?.usage === "bump" || material.detail?.usage === "diffuseAndBump") {
    gaps.push("the detail bump is not drawn");
  }

  return gaps.length ? `In this viewer, ${gaps.join(" and ")}` : null;
}

/**
 * What became of the bump pair on the frontend, when anything short of both halves uploaded.
 *
 * @param status - Each half's outcome, or null for a material that bound no pair.
 * @returns A line for the row, or null when there is nothing to explain.
 */
export function describeBumpUpload(status: Nullable<IVisualBumpStatus>): Nullable<string> {
  if (!status || (status.bump === EVisualTextureState.APPLIED && status.companion === EVisualTextureState.APPLIED)) {
    return null;
  }

  const halves: string =
    `bump ${describeTextureState(status.bump).label.toLowerCase()}, ` +
    `bump# ${describeTextureState(status.companion).label.toLowerCase()}`;

  return status.reason ? `${halves}: ${status.reason}` : halves;
}
