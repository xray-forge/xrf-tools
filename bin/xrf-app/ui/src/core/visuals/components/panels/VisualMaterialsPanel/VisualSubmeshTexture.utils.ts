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
