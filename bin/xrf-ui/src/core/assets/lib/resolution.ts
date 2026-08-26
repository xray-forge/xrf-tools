import { XrayAsset, XrayResolution } from "@/core/bindings/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/**
 * The asset a resolution located, or null when it located none.
 *
 * A substituted reference counts: the engine's dummy is a real file and rendering it is what the game does, which is
 * the point of substituting rather than leaving the submesh blank.
 *
 * @param resolution - What the backend reported for one reference.
 * @returns The located asset, or null when the outcome located nothing.
 */
export function getLocatedAsset(resolution: XrayResolution): Nullable<XrayAsset> {
  return listLocatedAssets(resolution)[0] ?? null;
}

/**
 * Every asset an outcome located, which for a masked reference is more than one.
 *
 * A texture reference answers with exactly one file, but a motion reference may be a mask - `wpn\wpn_ak74_*.omf` - so
 * naming only the first would misreport what was found.
 *
 * @param resolution - What the backend reported for one reference.
 * @returns The located assets, empty when the outcome located nothing.
 */
export function listLocatedAssets(resolution: XrayResolution): Array<XrayAsset> {
  return resolution.kind === "resolved" || resolution.kind === "substituted" ? resolution.assets : [];
}

/**
 * What the backend did with the reference, in the user's terms.
 *
 * Substitution is called out rather than presented as a plain resolution, because the file on screen is then not the
 * file the model asked for - which is what the game does too, and is worth knowing when a mesh looks wrong. A resolved
 * outcome names the step that answered, since with overlay trees which tree won is the thing that explains a surprise.
 *
 * @param resolution - What the backend reported for one reference.
 * @returns A single line naming the outcome.
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
