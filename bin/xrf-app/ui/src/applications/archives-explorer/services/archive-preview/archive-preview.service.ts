import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { ArchiveSubject, ArchiveWorldEntry, EArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveDescriptor, ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";
import { EXrayAssetContainer } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable, Optional } from "@xrf/types";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchivePreviewSupport,
  getArchivePreviewSupport,
  getArchiveVolumeOf,
  IArchiveEntry,
  TArchiveContent,
  TArchiveSelection,
} from "@/core/archive/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { EPathEntryKind } from "@/core/path/entry-kind";

import { ArchiveContentReader } from "./archive-preview.service.content";

/** What nothing selected is, which a subject opened or closed since the selection was made shows. */
const NOTHING_SELECTED: TArchiveSelection = { kind: "none" };

/** What the explorer points at, in the subject it was chosen in. */
interface IArchivePreviewView {
  /** The opening of the subject it was chosen in; another subject shows a view of its own. */
  sessionId: Nullable<SessionId>;
  selection: TArchiveSelection;
}

/**
 * Which file or directory of the open subject the explorer points at, and what was read of a file to preview it. Kept
 * with the subject it was chosen in, so a subject opened or closed since shows none of it.
 */
@Injectable()
export class ArchivePreviewService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The selected file as a preview draws it: text, or a decoded texture or sound with its description. */
  private readonly read: CommandOperation<TArchiveContent> = new CommandOperation(this.log);

  /** What each previewable kind has to be asked for, which is a table rather than state. */
  private readonly reader: ArchiveContentReader = new ArchiveContentReader();

  @Observable()
  private view: IArchivePreviewView = { selection: NOTHING_SELECTED, sessionId: null };

  /** The opening the read was asked of. */
  @Observable()
  private readFor: Nullable<SessionId> = null;

  public constructor(private readonly archivesService: ArchivesService = inject(ArchivesService)) {}

  /** What the explorer points at: one file, one directory, or nothing. */
  @Computed()
  public get selection(): TArchiveSelection {
    return this.held.selection;
  }

  /** What was read of the selected file for its preview. */
  @Computed()
  public get content(): AsyncState<TArchiveContent> {
    return this.readFor !== null && this.readFor === this.archivesService.subject.sessionId
      ? this.read.state
      : AsyncState.idle();
  }

  /**
   * @returns The selected file, or null.
   */
  @Computed()
  public get selectedEntry(): Nullable<IArchiveEntry> {
    return this.selection.kind === EPathEntryKind.FILE ? this.selection.entry : null;
  }

  /**
   * The name-table descriptor of the selected file, which only a volume set has.
   *
   * Looked up rather than carried on the selection: a selection is the one shape both subjects share, and widening it
   * to a union would make every surface that only shows a name narrow it first.
   *
   * @returns The descriptor, or null when a world is open or nothing is selected.
   */
  @Computed()
  public get selectedDescriptor(): Nullable<ArchiveFileDescriptor> {
    const subject: Nullable<ArchiveSubject> = this.archivesService.subject.value;
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (!entry || subject?.kind !== EArchiveSubject.VOLUMES) {
      return null;
    }

    return subject.project.files[entry.name] ?? null;
  }

  /**
   * The mounted-world entry selected, which carries where its bytes are and what it hides.
   *
   * @returns The entry, or null when a volume set is open or nothing is selected.
   */
  @Computed()
  public get selectedWorldEntry(): Nullable<ArchiveWorldEntry> {
    const subject: Nullable<ArchiveSubject> = this.archivesService.subject.value;
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (!entry || subject?.kind !== EArchiveSubject.WORLD) {
      return null;
    }

    return subject.world.files.find((candidate: ArchiveWorldEntry) => candidate.name === entry.name) ?? null;
  }

  /**
   * Where the selected file's bytes come from, and which copies that decision hides.
   *
   * @returns The origin of the selection, or null when nothing is selected.
   */
  @Computed()
  public get selectedOrigin(): Nullable<ArchiveWorldEntry> {
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;
    const subject: Nullable<ArchiveSubject> = this.archivesService.subject.value;

    if (!entry) {
      return null;
    }

    if (subject?.kind === EArchiveSubject.WORLD) {
      return this.selectedWorldEntry;
    }

    const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(subject?.project ?? null, this.selectedDescriptor);

    if (!volume) {
      return null;
    }

    return {
      container: { kind: EXrayAssetContainer.ARCHIVE, path: volume.path },
      name: entry.name,
      shadowed:
        this.archivesService.overridden.find((candidate: ArchiveWorldEntry) => candidate.name === entry.name)
          ?.shadowed ?? [],
      sizeReal: entry.sizeReal,
    };
  }

  /** The view of the subject held now, empty for one it was not chosen in. */
  @Computed()
  private get held(): IArchivePreviewView {
    const sessionId: Nullable<SessionId> = this.archivesService.subject.sessionId;

    return this.view.sessionId === sessionId ? this.view : { selection: NOTHING_SELECTED, sessionId };
  }

  /**
   * Selects a file and reads it for its preview; the latest file picked is the one read.
   *
   * @param entry - File to select.
   */
  @LatestFlow("content")
  public *selectFile(entry: IArchiveEntry): TFlow {
    this.log.info("Select archive file:", entry.name);

    this.view = { ...this.held, selection: { entry, kind: EPathEntryKind.FILE } };

    yield* this.readSelected(entry);
  }

  /**
   * Selects the file a description named, by the name the open subject lists it under.
   *
   * @param name - Entry name as the open subject lists it.
   */
  @BoundAction()
  public selectFileByName(name: string): void {
    const entry: Optional<IArchiveEntry> = this.archivesService.entries.find(
      (candidate: IArchiveEntry) => candidate.name === name
    );

    if (entry) {
      void this.selectFile(entry);
    } else {
      this.log.info("Referenced archive file is no longer listed:", name);
    }
  }

  /**
   * Selects a directory instead of a file, forgetting how the last writes ended, as they named what was selected.
   *
   * @param path - Engine directory path; an empty string selects the tree root.
   */
  @BoundAction()
  public selectDirectory(path: string): void {
    cancelFlow(this, "content");

    this.read.reset();
    this.view = { ...this.held, selection: { kind: EPathEntryKind.DIRECTORY, path } };
    this.archivesService.clearExtractions();
  }

  /** Reads the selected file again, after a read of it failed. */
  @LatestFlow("content")
  public *retry(): TFlow {
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (entry) {
      yield* this.readSelected(entry);
    }
  }

  /** Selects nothing, forgetting how the last writes ended, as they named what was selected. */
  @BoundAction()
  public clearSelection(): void {
    cancelFlow(this, "content");

    this.read.reset();
    this.view = { ...this.held, selection: NOTHING_SELECTED };
    this.archivesService.clearExtractions();
  }

  /**
   * Reads a selected file as its preview draws it; one no preview draws is not read.
   *
   * @param entry - Selected file to read.
   */
  private *readSelected(entry: IArchiveEntry): TFlow {
    const sessionId: Nullable<SessionId> = this.held.sessionId;
    const support: ArchivePreviewSupport = getArchivePreviewSupport(entry);

    this.read.reset();

    if (!sessionId) {
      return;
    }

    switch (support.kind) {
      case "audio":
      case "texture":
      case "image":
        return yield* this.readContent(sessionId, entry, support.kind);
      case "supported":
        return yield* this.readContent(sessionId, entry, "text");
      case "description":
        return yield* this.readContent(sessionId, entry, "description");
      case "model":
      case "too-large":
        return;
    }
  }

  /**
   * @param sessionId - The open subject's session.
   * @param entry - File to read.
   * @param kind - Preview representation to request from the backend.
   */
  private *readContent(sessionId: SessionId, entry: IArchiveEntry, kind: TArchiveContent["kind"]): TFlow {
    this.readFor = sessionId;

    yield* this.read.run({
      name: `Read archive ${kind}`,
      subject: entry.name,
      command: () => this.reader.read(sessionId, entry, kind),
      failure: `Could not read ${entry.name}`,
    });
  }
}
