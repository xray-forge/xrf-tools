import { listLocatedAssets } from "@/core/assets/lib/resolution";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { formatBytes } from "@/lib/memory/format";

/** What a model's textures weigh, and how much of what it asked for that accounts for. */
export interface IVisualTextureSummary {
  /** Distinct files measured, which is what `bytes` is the weight of. */
  files: number;
  /** References the model declares, whether or not they resolved. */
  references: number;
  /** References that located a file. Exceeds `files` when submeshes share one. */
  located: number;
  bytes: number;
}

/**
 * Weighs a model's textures by file rather than by reference.
 *
 * @param textures - Descriptors the open reported, keyed by logical path.
 * @param dependencies - The model's texture references, resolved or not.
 * @returns Counts and total bytes of what was actually measured.
 */
export function summarizeVisualTextures(
  textures: Record<string, AssetTextureDescriptor>,
  dependencies: Array<VisualTextureDependency>
): IVisualTextureSummary {
  const measured: Set<string> = new Set();

  let located: number = 0;

  for (const dependency of dependencies) {
    for (const asset of listLocatedAssets(dependency.resolution)) {
      located += 1;

      if (textures[asset.logicalPath]) {
        measured.add(asset.logicalPath);
      }
    }
  }

  let bytes: number = 0;

  for (const path of measured) {
    bytes += textures[path].size;
  }

  return { files: measured.size, references: dependencies.length, located, bytes };
}

/**
 * States a summary in the fewest words that stay true.
 *
 * The qualifications appear only when they say something: "of N" when a reference produced no file, and the reference
 * count when submeshes share one. On a model where neither holds - most of them - it reads as a plain count and weight.
 *
 * @param summary - Counts and bytes to describe.
 * @returns A single line, or null when the model declares no textures at all.
 */
export function describeVisualTextureSummary(summary: IVisualTextureSummary): string {
  const counted: string =
    summary.files === summary.references ? `${summary.files}` : `${summary.files} of ${summary.references}`;
  // Only when a file is genuinely shared. With nothing measured there is nothing to share, and "of N" already said so.
  const isShared: boolean = summary.files > 0 && summary.located > summary.files;
  const shared: string = isShared ? ` (${summary.located} references)` : "";
  const noun: string = summary.references === 1 ? "texture" : "textures";

  return `${counted} ${noun} · ${formatBytes(summary.bytes)}${shared}`;
}
