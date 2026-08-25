import { XrayResolution } from "@/core/bindings/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";

/** How a texture state reads in the panel, and how loudly. */
export interface IVisualTextureStateDescriptor {
  label: string;
  color: "default" | "success" | "warning" | "error";
}

/**
 * Wording and severity for one texture state.
 *
 * A submesh that never asked for a texture is not a problem and is coloured as such, while one that asked and did not
 * get it is. That distinction is the whole reason the panel shows this: an untextured mesh looks identical either way
 * in the viewport.
 */
export function describeTextureState(state: EVisualTextureState): IVisualTextureStateDescriptor {
  switch (state) {
    case EVisualTextureState.APPLIED:
      return { color: "success", label: "Applied" };

    case EVisualTextureState.DECODED:
      return { color: "success", label: "Decoded" };

    case EVisualTextureState.LOADING:
      return { color: "default", label: "Loading" };

    case EVisualTextureState.ABSENT:
      return { color: "default", label: "None declared" };

    case EVisualTextureState.UNSUPPORTED_FORMAT:
      return { color: "warning", label: "Format unsupported" };

    case EVisualTextureState.UNRESOLVED:
      return { color: "warning", label: "Not found" };

    case EVisualTextureState.FAILED:
      return { color: "error", label: "Failed" };
  }
}

/**
 * What the backend did with the reference, in the user's terms.
 *
 * Substitution is called out rather than presented as a plain resolution, because the file on screen is then not the
 * file the model asked for - which is what the game does too, and is worth knowing when a mesh looks wrong. A resolved
 * outcome names the step that answered, since with overlay trees which tree won is the thing that explains a surprise.
 */
export function describeResolution(resolution: XrayResolution): string {
  switch (resolution.kind) {
    case "resolved":
      return `Resolved in ${resolution.step}`;

    case "substituted":
      return `Missing, showing the engine placeholder from ${resolution.step}`;

    case "missing":
      return "Not present in any searched source";

    case "noScope":
      return "No source was searchable for this visual";

    case "rejected":
      return "Reference is not a usable asset path";
  }
}
