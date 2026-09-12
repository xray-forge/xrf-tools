// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import {
  ArchivesExtractRequest,
  ArchivesPackRequest,
  ArchivesPatchRequest,
  ArchivesUnpackRequest,
  AssetTextureDescriptor,
  AudioDescriptor,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/bindings/types/xrf-app";
import { ArchiveProject, ArchiveSharedPayload, ProjectReadResult } from "@/core/bindings/types/xrf-archive";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import {
  ArchiveExtractDirectoryResult,
  ArchiveExtractResult,
  ArchivePackConfig,
  ArchivePackResult,
  ArchivePatchConfig,
  ArchivePatchResult,
  ArchiveUnpackResult,
} from "@/core/bindings/types/xrf-pack";
import { XrayPathCollision, XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const archivesCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:archives|close_project", { sessionIds }),
  /**
   * Compares two roots without writing files.
   *
   * Ignores `is_forced` and takes no destination lease.
   */
  compareArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>("plugin:archives|compare_archives", { request, jobId, progress }),
  /**
   * Hand back a packing configuration with nothing chosen yet.
   *
   * The editor starts from this rather than from its own literals, so defaults that belong to the format
   * - the volume ceiling, the skip list, the mode - have one definition, in the packer.
   */
  defaultPackConfig: () => __TAURI_INVOKE<ArchivePackConfig>("plugin:archives|default_pack_config"),
  /** Returns the format defaults with empty paths and the volume name `patch`. */
  defaultPatchConfig: () => __TAURI_INVOKE<ArchivePatchConfig>("plugin:archives|default_patch_config"),
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
   * Paired with `archives|read_image`, which serves the PNG the webview displays. Both are addressed by the same roots
   * and logical path, so the dimensions on screen belong to the picture beside them.
   *
   * Answers with the source DDS facts rather than the PNG's: format and mip count survive the description and would not
   * survive the transcode, and a viewer of X-Ray textures wants both.
   */
  describeImage: (roots: XrayRoots, logicalPath: string) =>
    __TAURI_INVOKE<AssetTextureDescriptor>("plugin:archives|describe_image", { roots, logicalPath }),
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
   * Write the comparison scope and header of a configuration out as a patching configuration file.
   *
   * Only what such a file can carry is written, so a round trip through import returns what was exported. What is
   * compared, where it is published and under what name belong to the run rather than to the file.
   */
  exportPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<null>("plugin:archives|export_patch_config", { path, config }),
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
  /** Read a patching configuration file over the configuration the caller holds. */
  importPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<ArchivePatchConfig>("plugin:archives|import_patch_config", { path, config }),
  /** Write a single archived file to a path the user chose. */
  extractFile: (sessionId: SessionId, name: string, destination: string) =>
    __TAURI_INVOKE<ArchiveExtractResult>("plugin:archives|extract_file", { sessionId, name, destination }),
  /**
   * Write every archived file under one directory into a destination root.
   *
   * An empty prefix means the whole archive, so this also covers extracting everything without needing
   * a separate command — which is why it is a job rather than a quick read.
   *
   * Holds the destination tree exclusively, sharing that lease with an unpack: both lay the archive's own layout into
   * the root, so two runs there overlap whatever each was asked for, even where their prefixes differ.
   */
  extractDirectory: (request: ArchivesExtractRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveExtractDirectoryResult>("plugin:archives|extract_directory", { request, jobId, progress }),
  getProject: () => __TAURI_INVOKE<SessionRestore<ArchiveProject>>("plugin:archives|get_project"),
  hasProject: () => __TAURI_INVOKE<boolean>("plugin:archives|has_project"),
  /**
   * Entries the open volume set holds that no engine lookup can reach.
   *
   * Answered on demand out of the open project rather than stored beside it, so there is one source of truth and no
   * second slot to keep in step with an open or a close.
   *
   * Not part of [`ArchiveProject`]: the project keys entries by the name their volume's header authored, and folding
   * those onto engine identities is `xrf-vfs`'s to do. Asking the mount layer here is what keeps the explorer's answer
   * the same one `gamedata list` and `archive verify` give.
   */
  listCollisions: (sessionId: SessionId) =>
    __TAURI_INVOKE<Array<XrayPathCollision>>("plugin:archives|list_collisions", { sessionId }),
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
   * Payloads that several entries of the open volume set locate at once.
   *
   * Derived on demand out of the open project rather than stored beside it, the way `list_collisions` answers, so a
   * close cannot leave a stale answer behind. The derivation is `xrf-archive`'s: the format keeps no alias field, so
   * this is what a reader observes from equal descriptors and never what the packer recorded. See
   * [`ArchiveSharedPayload`].
   */
  listSharedPayloads: (sessionId: SessionId) =>
    __TAURI_INVOKE<Array<ArchiveSharedPayload>>("plugin:archives|list_shared_payloads", { sessionId }),
  openProject: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<ArchiveProject>>("plugin:archives|open_project", { sessionId, path }),
  /**
   * Packs a directory using the supplied configuration.
   *
   * Holds an exclusive destination lease. Replacing an existing set requires `is_forced`. Unforced runs roll
   * back on failure or cancellation; forced runs cannot restore overwritten volumes.
   */
  packDirectory: (request: ArchivesPackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePackResult>("plugin:archives|pack_directory", { request, jobId, progress }),
  /**
   * Publishes added and modified entries as patch volumes.
   *
   * Holds an exclusive destination lease and shares the publishing group with archive packing. Replacing an
   * existing set requires `is_forced`.
   */
  patchArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>("plugin:archives|patch_archives", { request, jobId, progress }),
  readFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<ProjectReadResult>("plugin:archives|read_file", { sessionId, path }),
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
