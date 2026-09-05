import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { Nullable } from "@/lib/types/general";

import { describeTextureState } from "./VisualSubmeshTexture.utils";

/**
 * What became of the bump pair on the frontend, when anything short of both halves uploaded.
 *
 * The one part of a material row that is the viewer's rather than the material's: what a renderer managed to upload
 * says nothing about what the game binds, which is why the rest of these describers live in `core/materials`.
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
