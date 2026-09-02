// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { AssetTextureDescriptor, AudioDescriptor } from "@/core/bindings/types/xrf-app";
import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveProject,
  ArchiveProjectReadPolicy,
  ProjectReadResult,
} from "@/core/bindings/types/xrf-archive";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import {
  ArchiveExtractDirectoryResult,
  ArchiveExtractResult,
  ArchivePackConfig,
  ArchivePackResult,
  ArchiveUnpackResult,
} from "@/core/bindings/types/xrf-pack";
import { XrayPathCollision, XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const archivesCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:archives|close_project"),
  /**
   * Hand back a packing configuration with nothing chosen yet.
   *
   * The editor starts from this rather than from its own literals, so defaults that belong to the format
   * - the volume ceiling, the skip list, the mode - have one definition, in the packer.
   */
  defaultPackConfig: () => __TAURI_INVOKE<ArchivePackConfig>("plugin:archives|default_pack_config"),
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
   * Write the selection rules of a configuration out as an xrCompress configuration file.
   *
   * Only what such a file can carry is written, so a round trip through import returns what was exported.
   * Paths, name, mode, and volume size belong to the run rather than to the file.
   */
  exportPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<null>("plugin:archives|export_pack_config", { path, config }),
  /**
   * Read an xrCompress configuration file over the configuration the caller holds.
   *
   * Layers rather than replaces, matching how the command line applies `--ltx`: a configuration file
   * carries selection rules and a header, never the source, destination, name, mode, or volume size, so
   * those stay as the caller had them.
   */
  importPackConfig: (path: string, config: ArchivePackConfig) =>
    __TAURI_INVOKE<ArchivePackConfig>("plugin:archives|import_pack_config", { path, config }),
  /** Write a single archived file to a path the user chose. */
  extractFile: (name: string, destination: string) =>
    __TAURI_INVOKE<ArchiveExtractResult>("plugin:archives|extract_file", { name, destination }),
  /**
   * Write every archived file under one directory into a destination root.
   *
   * An empty prefix means the whole archive, so this also covers extracting everything without needing
   * a separate command — which is why it is a job rather than a quick read.
   *
   * Holds the destination tree exclusively, sharing that lease with an unpack: both lay the archive's own layout into
   * the root, so two runs there overlap whatever each was asked for, even where their prefixes differ.
   */
  extractDirectory: (prefix: string, destination: string, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveExtractDirectoryResult>("plugin:archives|extract_directory", {
      prefix,
      destination,
      jobId,
      progress,
    }),
  getProject: () =>
    __TAURI_INVOKE<{
      /**
       * Volumes in merge order: a later one wins the name table, so a caller searching them as separate sources must
       * search them in reverse to resolve an entry to the bytes this project's table names.
       */
      archives: Array<ArchiveDescriptor>;
      /** Entries keyed by their authored name, which is the same allocation each descriptor carries as its `name`. */
      files: { [key in string]: ArchiveFileDescriptor };
      readPolicy: ArchiveProjectReadPolicy;
      /**
       * The tightest path holding exactly these volumes: the volume itself when one file was read, the volumes' common
       * parent when a directory was walked. Mounting it reaches this project's entries and no others, which is what a
       * caller reading an entry's bytes back out of the filesystem needs.
       */
      root: string;
      sizeReal: number;
    } | null>("plugin:archives|get_project"),
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
  listCollisions: () => __TAURI_INVOKE<Array<XrayPathCollision>>("plugin:archives|list_collisions"),
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
  openProject: (path: string) => __TAURI_INVOKE<ArchiveProject>("plugin:archives|open_project", { path }),
  /**
   * Pack a directory into archive volumes from a configuration held by the caller.
   *
   * Takes the whole configuration rather than a file path, so the editor packs exactly what is on screen
   * without having to save it first.
   *
   * Holds its destination exclusively for the whole run, so a second request for the same output set is refused rather
   * than allowed to truncate the volumes this one is writing.
   *
   * `is_forced` is the user answering for a destination that already holds this set; without it such a run is refused
   * before anything is written. It also decides what a stopped run leaves: an unforced one takes back the volumes it
   * made and the destination is untouched, while a forced one cannot tell its own output from what it replaced and
   * answers with a result naming every volume path it opened.
   */
  packDirectory: (config: ArchivePackConfig, isForced: boolean, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchivePackResult>("plugin:archives|pack_directory", { config, isForced, jobId, progress }),
  readFile: (path: string) => __TAURI_INVOKE<ProjectReadResult>("plugin:archives|read_file", { path }),
  /**
   * Unpack every archive of a directory into a destination tree, reporting progress and stopping on request.
   *
   * A cancelled run answers with a result rather than an error. It leaves the files it had already written where they
   * are — deleting them is not an option, because the destination may have held the user's own files and nothing here
   * can tell those apart from this run's — so the caller needs the counts to say what is now on disk.
   */
  unpackDirectory: (from: string, destination: string, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<ArchiveUnpackResult>("plugin:archives|unpack_directory", { from, destination, jobId, progress }),
};
