// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { IIpcCommandSignature } from "@/core/ipc/signature";
import {
  ArchiveFileDescription,
  ArchiveOverrideReport,
  ArchiveResolution,
  ArchivesExtractRequest,
  ArchivesPackRequest,
  ArchivesPatchRequest,
  ArchiveSubject,
  ArchivesUnpackRequest,
  AudioDescriptor,
  ImageDescriptor,
} from "@/core/ipc/types/xrf-app-archives";
import { AssetTextureDescriptor, SessionId, SessionRestore, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
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

/** The wire name of every command of the `archives` plugin. */
export enum EArchivesCommand {
  CLOSE_SUBJECT = "plugin:archives|close_subject",
  DESCRIBE_RESOLUTION = "plugin:archives|describe_resolution",
  DESCRIBE_STATISTICS = "plugin:archives|describe_statistics",
  EXTRACT_DIRECTORY = "plugin:archives|extract_directory",
  EXTRACT_FILE = "plugin:archives|extract_file",
  LIST_OVERRIDES = "plugin:archives|list_overrides",
  LIST_SHARED_PAYLOADS = "plugin:archives|list_shared_payloads",
  OPEN_VOLUMES = "plugin:archives|open_volumes",
  OPEN_WORLD = "plugin:archives|open_world",
  READ_FILE = "plugin:archives|read_file",
  RESTORE_SUBJECT = "plugin:archives|restore_subject",
  DESCRIBE_FILE = "plugin:archives|describe_file",
  DESCRIBE_AUDIO = "plugin:archives|describe_audio",
  DESCRIBE_TEXTURE = "plugin:archives|describe_texture",
  DESCRIBE_IMAGE = "plugin:archives|describe_image",
  DEFAULT_PACK_CONFIG = "plugin:archives|default_pack_config",
  EXPORT_PACK_CONFIG = "plugin:archives|export_pack_config",
  IMPORT_PACK_CONFIG = "plugin:archives|import_pack_config",
  LIST_PACK_VOLUMES = "plugin:archives|list_pack_volumes",
  PACK_DIRECTORY = "plugin:archives|pack_directory",
  COMPARE_ARCHIVES = "plugin:archives|compare_archives",
  DEFAULT_PATCH_CONFIG = "plugin:archives|default_patch_config",
  EXPORT_PATCH_CONFIG = "plugin:archives|export_patch_config",
  IMPORT_PATCH_CONFIG = "plugin:archives|import_patch_config",
  LIST_PATCH_VOLUMES = "plugin:archives|list_patch_volumes",
  PATCH_ARCHIVES = "plugin:archives|patch_archives",
  UNPACK_DIRECTORY = "plugin:archives|unpack_directory",
}

/** Commands */
export const archivesCommands = {
  /** Close the subjects `session_ids` name, committed or opening. */
  closeSubject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EArchivesCommand.CLOSE_SUBJECT, { sessionIds }),
  /** Where the open subject looks for an engine path, in the order it looks. */
  describeResolution: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveResolution>(EArchivesCommand.DESCRIBE_RESOLUTION, { sessionId }),
  /** What the open subject holds, broken down by extension, folder, size, and where its files come from. */
  describeStatistics: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveStatistics>(EArchivesCommand.DESCRIBE_STATISTICS, { sessionId }),
  /** Write every file the open subject holds under one directory into a destination root. */
  extractDirectory: (request: ArchivesExtractRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveExtractDirectoryResult>(EArchivesCommand.EXTRACT_DIRECTORY, { request, jobId, progress }),
  /** Write one file of the open subject to a path the user chose. */
  extractFile: (sessionId: SessionId, name: string, destination: string) =>
    __TAURI_INVOKE<ArchiveExtractResult>(EArchivesCommand.EXTRACT_FILE, { sessionId, name, destination }),
  /** What the open subject answers with more than one copy, and what it cannot reach at all. */
  listOverrides: (sessionId: SessionId) =>
    __TAURI_INVOKE<ArchiveOverrideReport>(EArchivesCommand.LIST_OVERRIDES, { sessionId }),
  /** Payloads that several entries of the open volume set locate at once. */
  listSharedPayloads: (sessionId: SessionId) =>
    __TAURI_INVOKE<Array<ArchiveSharedPayload>>(EArchivesCommand.LIST_SHARED_PAYLOADS, { sessionId }),
  /** Open one archive volume, or every volume beneath a directory, as a single name table. */
  openVolumes: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<ArchiveSubject>>(EArchivesCommand.OPEN_VOLUMES, { sessionId, path }),
  /**
   * Open a game folder as the engine mounts it: its archives and the loose tree standing in front of them. The world
   * follows commits: one that moves what its listing read lists it again, at the revision a page then refreshes to.
   */
  openWorld: (sessionId: SessionId, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<ArchiveSubject>>(EArchivesCommand.OPEN_WORLD, { sessionId, roots }),
  /** Read one file of the open subject as text, subject to the viewer's read policy, whose ceiling keeps it a short read. */
  readFile: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ArchiveReadResult>(EArchivesCommand.READ_FILE, { sessionId, name }),
  /**
   * What `tool`'s explorer has open in the asking window, the newest it opened, so a reloaded frontend adopts it
   * instead of asking for it again.
   */
  restoreSubject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<ArchiveSubject>>(EArchivesCommand.RESTORE_SUBJECT, { tool }),
  /**
   * Describe one entry of the open subject in words, for a format the viewer cannot draw: on the pool, since a
   * describer may parse a whole level bundle or walk a large file by seeking.
   */
  describeFile: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ArchiveFileDescription>(EArchivesCommand.DESCRIBE_FILE, { sessionId, name }),
  /** Report whatever the engine would read out of a sound of the open subject, without handing over the sound. */
  describeAudio: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<AudioDescriptor>(EArchivesCommand.DESCRIBE_AUDIO, { sessionId, name }),
  /** Report the shape of a texture of the open subject, without decoding it into a picture. */
  describeTexture: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<AssetTextureDescriptor>(EArchivesCommand.DESCRIBE_TEXTURE, { sessionId, name }),
  /**
   * Report the shape of a picture of the open subject the webview draws as it stands.
   *
   * # Errors
   *
   * Returns an error when the subject holds no such file, its bytes cannot be read, or its extension is not one a
   * webview draws - which the caller decided before asking, so reaching it means the two disagree.
   */
  describeImage: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ImageDescriptor>(EArchivesCommand.DESCRIBE_IMAGE, { sessionId, name }),
  /** Hand back a packing configuration with nothing chosen yet. */
  defaultPackConfig: () => __TAURI_INVOKE<ArchivePackConfig>(EArchivesCommand.DEFAULT_PACK_CONFIG),
  /** Write the selection rules of a configuration out as a packing configuration file. */
  exportPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<null>(EArchivesCommand.EXPORT_PACK_CONFIG, { path, config }),
  /** Read a packing configuration file over the configuration the caller holds. */
  importPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<ArchivePackConfig>(EArchivesCommand.IMPORT_PACK_CONFIG, { path, config }),
  /** Volumes of this configuration's set the destination already holds. */
  listPackVolumes: (config: ArchivePackConfig) =>
    __TAURI_INVOKE<Array<string>>(EArchivesCommand.LIST_PACK_VOLUMES, { config }),
  /** Packs a directory using the supplied configuration. */
  packDirectory: (request: ArchivesPackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePackResult>(EArchivesCommand.PACK_DIRECTORY, { request, jobId, progress }),
  /** Compares two roots without writing files. */
  compareArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>(EArchivesCommand.COMPARE_ARCHIVES, { request, jobId, progress }),
  /** Returns the format defaults with empty paths and the volume name `patch`. */
  defaultPatchConfig: () => __TAURI_INVOKE<ArchivePatchConfig>(EArchivesCommand.DEFAULT_PATCH_CONFIG),
  /** Write the comparison scope and header of a configuration out as a patching configuration file. */
  exportPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<null>(EArchivesCommand.EXPORT_PATCH_CONFIG, { path, config }),
  /** Read a patching configuration file over the configuration the caller holds. */
  importPatchConfig: (path: string, config: ArchivePatchConfig) =>
    __TAURI_INVOKE<ArchivePatchConfig>(EArchivesCommand.IMPORT_PATCH_CONFIG, { path, config }),
  /** Volumes of this configuration's set the output already holds. */
  listPatchVolumes: (config: ArchivePatchConfig) =>
    __TAURI_INVOKE<Array<string>>(EArchivesCommand.LIST_PATCH_VOLUMES, { config }),
  /** Publishes added and modified entries as patch volumes. */
  patchArchives: (request: ArchivesPatchRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePatchResult>(EArchivesCommand.PATCH_ARCHIVES, { request, jobId, progress }),
  /** Unpack every archive of a directory into a destination tree, reporting progress and stopping on request. */
  unpackDirectory: (request: ArchivesUnpackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveUnpackResult>(EArchivesCommand.UNPACK_DIRECTORY, { request, jobId, progress }),
};

/** What each command of the `archives` plugin takes and resolves to, keyed by its wire name. */
export interface IArchivesCommandSignatures {
  [EArchivesCommand.CLOSE_SUBJECT]: IIpcCommandSignature<typeof archivesCommands.closeSubject, ["sessionIds"]>;
  [EArchivesCommand.DESCRIBE_RESOLUTION]: IIpcCommandSignature<
    typeof archivesCommands.describeResolution,
    ["sessionId"]
  >;
  [EArchivesCommand.DESCRIBE_STATISTICS]: IIpcCommandSignature<
    typeof archivesCommands.describeStatistics,
    ["sessionId"]
  >;
  [EArchivesCommand.EXTRACT_DIRECTORY]: IIpcCommandSignature<
    typeof archivesCommands.extractDirectory,
    ["request", "jobId", "progress"]
  >;
  [EArchivesCommand.EXTRACT_FILE]: IIpcCommandSignature<
    typeof archivesCommands.extractFile,
    ["sessionId", "name", "destination"]
  >;
  [EArchivesCommand.LIST_OVERRIDES]: IIpcCommandSignature<typeof archivesCommands.listOverrides, ["sessionId"]>;
  [EArchivesCommand.LIST_SHARED_PAYLOADS]: IIpcCommandSignature<
    typeof archivesCommands.listSharedPayloads,
    ["sessionId"]
  >;
  [EArchivesCommand.OPEN_VOLUMES]: IIpcCommandSignature<typeof archivesCommands.openVolumes, ["sessionId", "path"]>;
  [EArchivesCommand.OPEN_WORLD]: IIpcCommandSignature<typeof archivesCommands.openWorld, ["sessionId", "roots"]>;
  [EArchivesCommand.READ_FILE]: IIpcCommandSignature<typeof archivesCommands.readFile, ["sessionId", "name"]>;
  [EArchivesCommand.RESTORE_SUBJECT]: IIpcCommandSignature<typeof archivesCommands.restoreSubject, ["tool"]>;
  [EArchivesCommand.DESCRIBE_FILE]: IIpcCommandSignature<typeof archivesCommands.describeFile, ["sessionId", "name"]>;
  [EArchivesCommand.DESCRIBE_AUDIO]: IIpcCommandSignature<typeof archivesCommands.describeAudio, ["sessionId", "name"]>;
  [EArchivesCommand.DESCRIBE_TEXTURE]: IIpcCommandSignature<
    typeof archivesCommands.describeTexture,
    ["sessionId", "name"]
  >;
  [EArchivesCommand.DESCRIBE_IMAGE]: IIpcCommandSignature<typeof archivesCommands.describeImage, ["sessionId", "name"]>;
  [EArchivesCommand.DEFAULT_PACK_CONFIG]: IIpcCommandSignature<typeof archivesCommands.defaultPackConfig, []>;
  [EArchivesCommand.EXPORT_PACK_CONFIG]: IIpcCommandSignature<
    typeof archivesCommands.exportPackConfig,
    ["path", "config"]
  >;
  [EArchivesCommand.IMPORT_PACK_CONFIG]: IIpcCommandSignature<
    typeof archivesCommands.importPackConfig,
    ["path", "config"]
  >;
  [EArchivesCommand.LIST_PACK_VOLUMES]: IIpcCommandSignature<typeof archivesCommands.listPackVolumes, ["config"]>;
  [EArchivesCommand.PACK_DIRECTORY]: IIpcCommandSignature<
    typeof archivesCommands.packDirectory,
    ["request", "jobId", "progress"]
  >;
  [EArchivesCommand.COMPARE_ARCHIVES]: IIpcCommandSignature<
    typeof archivesCommands.compareArchives,
    ["request", "jobId", "progress"]
  >;
  [EArchivesCommand.DEFAULT_PATCH_CONFIG]: IIpcCommandSignature<typeof archivesCommands.defaultPatchConfig, []>;
  [EArchivesCommand.EXPORT_PATCH_CONFIG]: IIpcCommandSignature<
    typeof archivesCommands.exportPatchConfig,
    ["path", "config"]
  >;
  [EArchivesCommand.IMPORT_PATCH_CONFIG]: IIpcCommandSignature<
    typeof archivesCommands.importPatchConfig,
    ["path", "config"]
  >;
  [EArchivesCommand.LIST_PATCH_VOLUMES]: IIpcCommandSignature<typeof archivesCommands.listPatchVolumes, ["config"]>;
  [EArchivesCommand.PATCH_ARCHIVES]: IIpcCommandSignature<
    typeof archivesCommands.patchArchives,
    ["request", "jobId", "progress"]
  >;
  [EArchivesCommand.UNPACK_DIRECTORY]: IIpcCommandSignature<
    typeof archivesCommands.unpackDirectory,
    ["request", "jobId", "progress"]
  >;
}
