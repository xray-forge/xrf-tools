import { ArchiveFileDescription, AudioDescriptor, ImageDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { AssetTextureDescriptor } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
import { SessionId } from "@xrf/ipc/types/xrf-session";
import { EPathEntryKind } from "@xrf/path";

import { IArchiveEntry } from "./entry";

/**
 * What the explorer currently points at.
 */
export type TArchiveSelection =
  | { kind: "none" }
  | { kind: EPathEntryKind.FILE; entry: IArchiveEntry }
  | { kind: EPathEntryKind.DIRECTORY; path: string };

/**
 * Bytes of one archived asset, as the bulk routes deliver them.
 *
 * Pinned to `ArrayBuffer` rather than left as the default `ArrayBufferLike`, because these are never shared memory and
 * a `Blob` refuses anything that might be. The alternative is casting at every use.
 */
export type TArchiveBytes = Uint8Array<ArrayBuffer>;

/**
 * What was loaded for the current selection, whatever form it took.
 *
 * A picture carries its descriptor beside its bytes, as they arrive by two calls. A sound carries its descriptor and
 * where it is, as the application's audio plays it from the backend by the same session and name. Every read is
 * addressed by the open subject's session, so the parts always describe the same file.
 */
export type TArchiveContent =
  | { kind: "text"; result: ArchiveReadResult }
  | { kind: "texture"; descriptor: AssetTextureDescriptor; bytes: TArchiveBytes }
  | { kind: "image"; descriptor: ImageDescriptor; bytes: TArchiveBytes }
  | { kind: "audio"; descriptor: AudioDescriptor; sessionId: SessionId; name: string }
  | { kind: "description"; description: ArchiveFileDescription };
