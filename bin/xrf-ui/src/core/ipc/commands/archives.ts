// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  ArchiveFileDescription,
  ArchiveOverrideReport,
  ArchiveResolution,
  ArchivesExtractRequest,
  ArchivesPackRequest,
  ArchivesPatchRequest,
  ArchiveSubject,
  ArchivesUnpackRequest,
  AssetTextureDescriptor,
  AudioDescriptor,
  ImageDescriptor,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { ArchiveReadResult, ArchiveSharedPayload } from "@/core/ipc/types/xrf-archive";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import {
  ArchiveExtractDirectoryResult,
  ArchiveExtractResult,
  ArchivePackConfig,
  ArchivePackResult,
  ArchivePatchConfig,
  ArchivePatchResult,
  ArchiveUnpackResult,
} from "@/core/ipc/types/xrf-pack";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const archivesCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeSubject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:archives|close_subject", { sessionIds }),
  /** Where the open subject looks for an engine path, in the order it looks. */
  describeResolution: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveResolution>("plugin:archives|describe_resolution", { sessionId }),
  /** What the open subject holds, broken down by extension, folder, size, and where its files come from. */
  describeStatistics: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveStatistics>("plugin:archives|describe_statistics", { sessionId }),
  /** Write every file the open subject holds under one directory into a destination root. */
  extractDirectory: (request: ArchivesExtractRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveExtractDirectoryResult>("plugin:archives|extract_directory", { request, jobId, progress }),
  /** Write one file of the open subject to a path the user chose. */
  extractFile: (sessionId: SessionId, name: string, destination: string) =>
    __TAURI_INVOKE<ArchiveExtractResult>("plugin:archives|extract_file", { sessionId, name, destination }),
  /** What the explorer has open, so a reloaded frontend adopts it instead of asking for it again. */
  getSubject: () => __TAURI_INVOKE<SessionRestore<ArchiveSubject>>("plugin:archives|get_subject"),
  /** What the open subject answers with more than one copy, and what it cannot reach at all. */
  listOverrides: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveOverrideReport>("plugin:archives|list_overrides", { sessionId }),
  /** Payloads that several entries of the open volume set locate at once. */
  listSharedPayloads: (sessionId: SessionId) =>
    __TAURI_INVOKE<Array<ArchiveSharedPayload>>("plugin:archives|list_shared_payloads", { sessionId }),
  /** Open one archive volume, or every volume beneath a directory, as a single name table. */
  openVolumes: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<ArchiveSubject>>("plugin:archives|open_volumes", { sessionId, path }),
  /** Open a game folder as the engine mounts it: its archives and the loose tree standing in front of them. */
  openWorld: (sessionId: SessionId, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<ArchiveSubject>>("plugin:archives|open_world", { sessionId, roots }),
  /** Read one file of the open subject as text, subject to the viewer's read policy. */
  readFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<ArchiveReadResult>("plugin:archives|read_file", { sessionId, path }),
  /**
   * Describe one entry of the open subject in words, for a format the viewer cannot draw.
   *
   * Addressed by the session rather than by roots, because the answer is about an entry of what is open: the subject
   * owns the name table a reference is resolved against and the policy the read is bounded by. The picture and sound
   * commands take roots instead because they exist to serve bytes to the webview, which is a different question.
   */
  describeFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<ArchiveFileDescription>("plugin:archives|describe_file", { sessionId, path }),
  /**
   * Report whatever the engine would read out of a sound, without handing over the sound.
   *
   * Paired with `assets|read_asset`, which serves the bytes the webview plays. Both are addressed by the same roots and
   * logical path, so the numbers on screen describe the file that is playing rather than a second lookup's answer.
   */
  describeAudio: (roots: XrayRoots, logicalPath: string) =>
    __TAURI_INVOKE<AudioDescriptor>("plugin:archives|describe_audio", { roots, logicalPath }),
  /**
   * Report the shape of a texture, without decoding it into a picture.
   *
   * Paired with `archives|read_texture`, which serves the PNG the webview displays. Both are addressed by the same roots
   * and logical path, so the dimensions on screen belong to the picture beside them.
   *
   * Answers with the source DDS facts rather than the PNG's: format and mip count survive the description and would not
   * survive the transcode, and a viewer of X-Ray textures wants both.
   */
  describeTexture: (roots: XrayRoots, logicalPath: string) =>
    __TAURI_INVOKE<AssetTextureDescriptor>("plugin:archives|describe_texture", { roots, logicalPath }),
  /**
   * Report the shape of a picture the webview draws as it stands.
   *
   * # Errors
   *
   * Returns an error when the path resolves to nothing, its bytes cannot be read, or its extension is not one a
   * webview draws - which the caller decided before asking, so reaching it means the two disagree.
   */
  describeImage: (roots: XrayRoots, logicalPath: string) =>
    __TAURI_INVOKE<ImageDescriptor>("plugin:archives|describe_image", { roots, logicalPath }),
  /**
   * Hand back a packing configuration with nothing chosen yet.
   *
   * The editor starts from this rather than from its own literals, so defaults that belong to the format
   * - the volume ceiling, the skip list, the mode - have one definition, in the packer.
   */
  defaultPackConfig: () => __TAURI_INVOKE<ArchivePackConfig>("plugin:archives|default_pack_config"),
  /**
   * Write the selection rules of a configuration out as a packing configuration file.
   *
   * The writer is chosen from the destination's extension, so a surface offering both gets the format it asked for.
   * Only what such a file can carry is written, so a round trip through import returns what was exported.
   * Paths, name, mode, and volume size belong to the run rather than to the file.
   */
  exportPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<null>("plugin:archives|export_pack_config", { path, config }),
  /**
   * Read a packing configuration file over the configuration the caller holds.
   *
   * The codec is chosen from the path's extension, so one command reads an `ltx` and a `json` alike.
   *
   * Layers rather than replaces, matching how the command line applies `--config`: a configuration file
   * carries selection rules and a header, never the source, destination, name, mode, or volume size, so
   * those stay as the caller had them.
   */
  importPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<ArchivePackConfig>("plugin:archives|import_pack_config", { path, config }),
  /**
   * Volumes of this configuration's set the destination already holds.
   *
   * Asked before packing rather than after: the editor puts a pack behind a confirmation, and a run that would replace
   * an archive the user still has is exactly what that confirmation is for. Packing refuses the same destination on its
   * own, so this is what the user is shown, not what protects them.
   *
   * Cheap enough to answer on the async worker — one directory listing, no file is opened.
   */
  listPackVolumes: (config: ArchivePackConfig) =>
    __TAURI_INVOKE<Array<string>>("plugin:archives|list_pack_volumes", { config }),
  /**
   * Packs a directory using the supplied configuration.
   *
   * Holds an exclusive destination lease. Replacing an existing set requires `is_forced`. Unforced runs roll
   * back on failure or cancellation; forced runs cannot restore overwritten volumes.
   */
  packDirectory: (request: ArchivesPackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePackResult>("plugin:archives|pack_directory", { request, jobId, progress }),
  /**
   * Compares two roots without writing files.
   *
   * Ignores `is_forced` and takes no destination lease.
   */
  compareArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>("plugin:archives|compare_archives", { request, jobId, progress }),
  /** Returns the format defaults with empty paths and the volume name `patch`. */
  defaultPatchConfig: () => __TAURI_INVOKE<ArchivePatchConfig>("plugin:archives|default_patch_config"),
  /**
   * Write the comparison scope and header of a configuration out as a patching configuration file.
   *
   * Only what such a file can carry is written, so a round trip through import returns what was exported. What is
   * compared, where it is published and under what name belong to the run rather than to the file.
   */
  exportPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<null>("plugin:archives|export_patch_config", { path, config }),
  /** Read a patching configuration file over the configuration the caller holds. */
  importPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<ArchivePatchConfig>("plugin:archives|import_patch_config", { path, config }),
  /**
   * Volumes of this configuration's set the output already holds.
   *
   * The patcher's twin of `list_pack_volumes`, and asked for the same reason: the editor puts publishing behind a
   * confirmation, and a run that would replace volumes the user still has is exactly what that confirmation is for.
   * Publishing refuses the same output on its own, so this is what the user is shown, not what protects them.
   *
   * Cheap enough to answer on the async worker — one directory listing, no file is opened.
   */
  listPatchVolumes: (config: ArchivePatchConfig) =>
    __TAURI_INVOKE<Array<string>>("plugin:archives|list_patch_volumes", { config }),
  /**
   * Publishes added and modified entries as patch volumes.
   *
   * Holds an exclusive destination lease and shares the publishing group with archive packing. Replacing an
   * existing set requires `is_forced`.
   */
  patchArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>("plugin:archives|patch_archives", { request, jobId, progress }),
  /**
   * Unpack every archive of a directory into a destination tree, reporting progress and stopping on request.
   *
   * A cancelled run answers with a result rather than an error. It leaves the files it had already written where they
   * are — deleting them is not an option, because the destination may have held the user's own files and nothing here
   * can tell those apart from this run's — so the caller needs the counts to say what is now on disk.
   */
  unpackDirectory: (request: ArchivesUnpackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveUnpackResult>("plugin:archives|unpack_directory", { request, jobId, progress }),
};
