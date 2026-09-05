import { listLocatedAssets } from "@/core/assets/lib/resolution";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { formatBytes } from "@/lib/memory/format";

/** What a model's textures weigh, how much of what it asked for that accounts for, and what the renderer bumps. */
export interface IVisualTextureSummary {
  /** Distinct files measured, which is what `bytes` is the weight of. */
  files: number;
  /** References the model declares, whether or not they resolved. */
  references: number;
  /** References that located a file. Exceeds `files` when submeshes share one. */
  located: number;
  bytes: number;
  /** Distinct references whose declaration binds a bump pair, whatever became of the pair. */
  bumped: number;
  /** Of `bumped`, the ones where the engine binds a dummy or the not-existing texture instead of the authored file. */
  degraded: number;
}

/**
 * Weighs a model's textures by file rather than by reference, and counts what the renderer bumps by reference.
 *
 * Materials are counted per reference because that is how the engine looks them up: two submeshes declaring one
 * texture are one material, and two declaring different textures that happen to share a file are two.
 *
 * @param textures - Descriptors the open reported, keyed by logical path.
 * @param dependencies - The model's texture references, resolved or not.
 * @param materials - What the renderer builds for each reference, keyed by the reference.
 * @returns Counts and total bytes of what was actually measured.
 */
export function summarizeVisualTextures(
  textures: Record<string, AssetTextureDescriptor>,
  dependencies: Array<VisualTextureDependency>,
  materials: Record<string, XrayMaterialDescriptor> = {}
): IVisualTextureSummary {
  const measured: Set<string> = new Set();
  const declared: Set<string> = new Set();

  let located: number = 0;
  let bumped: number = 0;
  let degraded: number = 0;

  for (const dependency of dependencies) {
    for (const asset of listLocatedAssets(dependency.resolution)) {
      located += 1;

      if (textures[asset.logicalPath]) {
        measured.add(asset.logicalPath);
      }
    }

    if (declared.has(dependency.reference)) {
      continue;
    }

    declared.add(dependency.reference);

    const outcome: string = materials[dependency.reference]?.outcome ?? "flat";

    if (outcome !== "flat") {
      bumped += 1;
    }

    if (outcome === "dummy" || outcome === "missing") {
      degraded += 1;
    }
  }

  let bytes: number = 0;

  for (const path of measured) {
    bytes += textures[path].size;
  }

  return { files: measured.size, references: dependencies.length, located, bytes, bumped, degraded };
}

/**
 * States a summary in the fewest words that stay true.
 *
 * The qualifications appear only when they say something: "of N" when a reference produced no file, the reference
 * count when submeshes share one, the bumped count when any material binds a bump, and the degraded count when any of
 * those binds a substitute - a line saying "8 bumped" over two flat dummies would repeat, at model level, the lie the
 * Materials panel exists to correct. On a model where none holds - most of them - it reads as a plain count and weight.
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
  const bumped: string = summary.bumped > 0 ? ` · ${summary.bumped} bumped` : "";
  const degraded: string = summary.degraded > 0 ? `, ${summary.degraded} degraded` : "";

  return `${counted} ${noun} · ${formatBytes(summary.bytes)}${shared}${bumped}${degraded}`;
}
