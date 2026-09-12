import { IArchiveEntry } from "@/core/archive/entry";
import { AssetTextureDescriptor, AudioDescriptor } from "@/core/bindings/types/xrf-app";
import { ArchiveReadResult } from "@/core/bindings/types/xrf-archive";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { EPathEntryKind } from "@/core/path/entry-kind";

/**
 * What the explorer currently points at.
 */
export type TArchiveSelection =
  | { kind: "none" }
  | { kind: EPathEntryKind.FILE; entry: IArchiveEntry }
  | { kind: EPathEntryKind.DIRECTORY; path: string };

/**
 * Bytes of one archived asset, as the raw commands deliver them.
 *
 * Pinned to `ArrayBuffer` rather than left as the default `ArrayBufferLike`, because these are never shared memory and
 * a `Blob` refuses anything that might be. The alternative is casting at every use.
 */
export type TArchiveBytes = Uint8Array<ArrayBuffer>;

/**
 * What was loaded for the current selection, whatever form it took.
 *
 * Media carries its description and its bytes side by side because they arrive as two calls: the bytes are what the
 * webview plays or paints, the descriptor is what the engine would read. Both are fetched against one roots, so they
 * always describe the same file.
 */
export type TArchiveContent =
  | { kind: "text"; result: ArchiveReadResult }
  | { kind: "image"; descriptor: AssetTextureDescriptor; bytes: TArchiveBytes }
  | { kind: "audio"; descriptor: AudioDescriptor; bytes: TArchiveBytes };

/** The last thing written to disk, so the surface that started it can report what happened. */
export type TArchiveOperation =
  { kind: "extract-file"; destination: string } | { kind: "extract-directory"; result: ArchiveExtractDirectoryResult };
