// Auto-generated rust bindings. Do not edit it manually.

import { JobProgress } from "@/core/ipc/types/xrf-job";
import { XrayRevision } from "@/core/ipc/types/xrf-vfs";

/**
 * What one commit changed, or a derivation landing after it, as every page subscribed to changes is told: the project
 * revision it left, and what it did to each open session that moved or could not.
 */
export type AssetChangeEvent = {
  revision: XrayRevision;
  sessions: Array<SessionChange>;
};

/** Every `kind` the `AssetSource` union is told apart by, so a switch or a comparison names one. */
export enum EAssetSource {
  /** A file on disk, named by its filesystem path. */
  FILE = "file",
  /** An asset of the roots, named by its logical path such as `meshes\actors\stalker.ogf`. */
  ASSET = "asset",
}

/**
 * Where a document is read from: a file on disk, read by its path and mounted nowhere, or an asset of the roots, loose
 * or archived, read by its logical path.
 */
export type AssetSource =
  /** A file on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** An asset of the roots, named by its logical path such as `meshes\actors\stalker.ogf`. */
  | { kind: "asset"; logicalPath: string };

/** What a texture file is, once it has been located. */
export type AssetTextureDescriptor = {
  /** Bytes the file occupies, which is also what a renderer uploads for a block-compressed texture. */
  size: number;
  /** Header facts, absent when the bytes are not a readable DDS. */
  shape: AssetTextureShape | null;
};

/** Pixel layout a DDS header declares. */
export type AssetTextureShape = {
  width: number;
  height: number;
  /** Levels the file carries, one meaning no mip chain at all. */
  mipmapLevels: number;
  /** Format name from [`DdsMetadata::get_format_label`], so the viewer and the sweep agree on what a file is. */
  format: string;
};

/**
 * What was read at a path, so a later write cannot overwrite a change it never saw and a reload can tell the file
 * moved.
 */
export type FileStamp = {
  size: number;
  /** Milliseconds since the Unix epoch, as the platform reports the file's modification time. */
  modifiedMs: number;
};

/** How a job that is no longer running ended. */
export enum EJobConclusion {
  COMPLETED = "completed",
  CANCELLED = "cancelled",
  FAILED = "failed",
}

/** Every `EJobConclusion` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type JobConclusion = `${EJobConclusion}`;

/** One job as the listing describes it, running or recently finished. */
export type JobDescription = {
  id: string;
  /** What kind of work this is, as the command that started it named itself. */
  kind: JobKind;
  /** What this job holds exclusively, so a refused start can be explained by pointing at the job that refused it. */
  leaseKeys: Array<string>;
  /**
   * What the job was asked to do, as the command that started it described itself.
   * Absent for a job whose command described nothing.
   */
  request: unknown | null;
  /**
   * Whether stopping has been asked for. A job can carry this and still be running: cancellation lands at a boundary
   * the operation chooses, and the gap between asking and stopping is exactly what a reader wants to see.
   */
  isCancelRequested: boolean;
  /**
   * The job's own progress: live for a running job, as last seen for a finished one.
   * Absent for a job registered but not yet reporting — a run holding a lease while it validates its inputs, say.
   */
  progress: JobProgress | null;
  /** Absent while the job is running. */
  conclusion: JobConclusion | null;
  /** Why it failed, where it did. */
  error: string | null;
  /**
   * What the run answered, for a job that completed.
   * Absent while the job runs, and for a job that failed or was cancelled before it had an answer.
   */
  result: unknown | null;
  /** How long the job ran, measured by the registry rather than by the operation. */
  duration: number;
  /** When the registry took it on, as milliseconds since the epoch. */
  startedAt: number;
};

/**
 * The kind of an operation that can be registered, cancelled and rediscovered: its domain's plugin name, a dot and the
 * operation (`archives.pack`).
 *
 * Each domain declares its own kinds in its registry (`@jobs`), which builds them under its plugin name, so the
 * registry knows no domain and no domain can spell another's kind.
 */
export type JobKind = string;

/** What one committed change did to one open session, as a change event reports it. */
export type SessionChange = {
  sessionId: SessionId;
  /** The revision the session's snapshot is at now: the change's where it moved, its own where it could not. */
  revision: XrayRevision;
  root: SessionRootChange;
  /**
   * The parts the session let go, by name and sorted: built from something the change moved, or from the value it
   * replaced. A subscriber refetches what it showed of them.
   */
  staleParts: Array<string>;
  /**
   * Whether a view drawn from the session is built anew rather than updated: its value was derived again, or, the value
   * kept, a part its owner builds the view from went stale (`Session::with_view_parts`).
   */
  isViewRenewed: boolean;
};

/** A single opening, allocated by its caller before dispatch so it can also be closed while pending. */
export type SessionId = string;

/**
 * What a restore answers: the snapshot a tool holds, or none.
 *
 * A command answering `Option<SessionSnapshot<T>>` itself would be described with an unbound `T`: Tauri Specta
 * inlines an optional named result, and the snapshot's value is described through its parameter. A named answer is
 * referenced instead, with its parameter bound.
 */
export type SessionRestore<T> = SessionSnapshot<T> | null;

/** Every `kind` the `SessionRootChange` union is told apart by, so a switch or a comparison names one. */
export enum ESessionRootChange {
  /** Nothing it was read from moved: the next snapshot shares it. */
  KEPT = "kept",
  /** Something it was read from moved, and its plugin derived it again. */
  DERIVED = "derived",
  /**
   * Something it was read from moved, and its plugin derives it again off the commit: the session shows it as it was
   * until a later event reports it `Derived` or `Failed`.
   */
  DERIVING = "deriving",
  /** Something it was read from moved, and nothing derives it again: the session shows it as it was. */
  STALE = "stale",
  /** Deriving it again failed: the session stays at the snapshot it had. */
  FAILED = "failed",
}

/** What became of a session's value, the root every part derives from. */
export type SessionRootChange =
  /** Nothing it was read from moved: the next snapshot shares it. */
  | { kind: "kept" }
  /** Something it was read from moved, and its plugin derived it again. */
  | { kind: "derived" }
  /**
   * Something it was read from moved, and its plugin derives it again off the commit: the session shows it as it was
   * until a later event reports it `Derived` or `Failed`.
   */
  | { kind: "deriving" }
  /** Something it was read from moved, and nothing derives it again: the session shows it as it was. */
  | { kind: "stale" }
  /** Deriving it again failed: the session stays at the snapshot it had. */
  | { kind: "failed"; reason: string };

/** An immutable value addressed by the opening that produced it; a held snapshot survives close. */
export type SessionSnapshot<T> = {
  sessionId: SessionId;
  /** The project revision the snapshot is current at: what a change moved it to, or where an open left it. */
  revision: XrayRevision;
  /**
   * Shared with the snapshots a change moves the session to while what it was read from stays current, so a commit
   * that touched none of it copies nothing.
   */
  value: T;
};

/** Where the transport listens, and the token every request to it carries. */
export type TransportEndpoint = {
  /** The server's origin, such as `http://127.0.0.1:52011`, which a route's path is appended to. */
  origin: string;
  /** Sent as `Authorization: Bearer <token>`; new every launch. */
  token: string;
};
