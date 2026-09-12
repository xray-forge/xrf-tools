import { inject, Injectable } from "@wirestate/core";
import { Computed, Observable, runInAction } from "@wirestate/mobx";

import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxResolvedIndex, LtxResolvedIndexEntry, LtxResolvedSection } from "@/core/bindings/types/xrf-ltx-inspect";
import { transformError } from "@/core/error/lib";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * What one entry point resolves to, indexed once and paged on demand.
 *
 * The index is the whole document's shape - every section, named and counted - and it is what lets the view know its
 * own height before a single body arrives. Bodies come a page at a time as they scroll into view, because a vanilla
 * `system.ltx` resolves to 23,500 sections holding 293,000 fields and no screen shows more than forty lines of it.
 */
@Injectable()
export class ConfigsResolvedService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The entry point being read, or null when nothing is. */
  @Observable()
  public entry: Nullable<string> = null;

  /** Every section the entry point resolves to, named and counted. */
  @Observable()
  public index: AsyncState<LtxResolvedIndex> = AsyncState.idle();

  /**
   * How many pages have landed, which is what a consumer re-reads `sections` on.
   *
   * A counter beside a mutable map rather than a new map per page: a full scroll of a resolved `system.ltx` lands
   * hundreds of pages, and copying a map that grows to 23,500 entries for each of them costs more than every fetch put
   * together. The map is only ever added to, so a revision says everything a consumer needs to know about it.
   */
  @Observable()
  public revision: number = 0;

  /** Bodies that have arrived, by section name. Read it alongside `revision`. */
  public readonly sections: Map<string, LtxResolvedSection> = new Map();

  /**
   * The config the view is narrowed to, or null for the whole root.
   *
   * An included config resolves through its entry point, so its sections live in a document of tens of thousands. What
   * a person opening `items\w_ak74.ltx` wants is what *that* file's sections came to, not where they sit in
   * `system.ltx` - so the view narrows to the sections that config declared, and says which entry point resolved them.
   */
  @Observable()
  public narrowedTo: Nullable<string> = null;

  /** Sections asked for and not yet answered, so a second viewport does not ask again. */
  private readonly pending: Set<string> = new Set();

  /**
   * @returns Whether an index is on screen, which is what makes the resolved view showable.
   */
  @Computed()
  public get isReady(): boolean {
    return this.index.value !== null;
  }

  /**
   * @returns The sections the view shows: the whole root, or only those the narrowed config declared.
   */
  @Computed()
  public get visibleSections(): Array<LtxResolvedIndexEntry> {
    const sections: Array<LtxResolvedIndexEntry> = this.index.value?.sections ?? [];

    return this.narrowedTo ? sections.filter((section) => section.origin === this.narrowedTo) : sections;
  }

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Narrow the view to one config's own sections, or widen it back to the whole root.
   *
   * @param path - Engine identity of the config to narrow to, or null for the whole root.
   */
  public narrowTo(path: Nullable<string>): void {
    runInAction(() => {
      this.narrowedTo = path;
    });
  }

  /**
   * Read one entry point's resolution, replacing whatever was open.
   *
   * Superseding is right here: a different entry point is a different document, and pages already fetched describe the
   * one being replaced.
   *
   * @param entry - Engine identity of the entry point to resolve.
   */
  @LatestFlow("index")
  public *open(entry: string): TFlow {
    const sessionId: Nullable<string> = this.projectService.sessionId;

    if (!sessionId) {
      this.log.error("Cannot resolve an entry point with no project open:", entry);

      return;
    }

    if (this.entry === entry && this.index.isReady) {
      return;
    }

    this.reset(entry);
    this.index = this.index.asLoading();

    try {
      const index: LtxResolvedIndex = yield* call(configsCommands.listResolvedSections({ entry, sessionId }));

      this.log.info("Resolved", entry, "to", index.sections.length, "sections");

      this.index = this.index.asReady(index);
    } catch (error) {
      this.log.error("Failed to resolve the entry point:", entry, error);

      this.index = this.index.asFailed(transformError(error));
    }
  }

  /**
   * Fetch the bodies of sections that have scrolled into view.
   *
   * Not a flow, because neither lane policy fits: superseding would cancel a page for a different part of the document,
   * and ignoring would drop one the viewport is waiting on. Pages are additive and addressed by name, so the guard is
   * what has already been asked for rather than what is already running.
   *
   * @param names - Sections the viewport needs, in any order. Ones already held or in flight are dropped.
   */
  public async request(names: ReadonlyArray<string>): Promise<void> {
    const sessionId: Nullable<string> = this.projectService.sessionId;
    const entry: Nullable<string> = this.entry;

    if (!sessionId || !entry) {
      return;
    }

    const wanted: Array<string> = names.filter((name: string) => !this.sections.has(name) && !this.pending.has(name));

    if (!wanted.length) {
      return;
    }

    for (const name of wanted) {
      this.pending.add(name);
    }

    try {
      const sections: Array<LtxResolvedSection> = await configsCommands.readResolvedSections({
        entry,
        names: wanted,
        sessionId,
      });

      runInAction(() => {
        // Checked on the way back, not only on the way out: a reopen between the ask and the answer would otherwise
        // fill this document with another one's sections.
        if (this.entry !== entry) {
          return;
        }

        for (const section of sections) {
          this.sections.set(section.name, section);
        }

        this.revision += 1;
      });
    } catch (error) {
      this.log.error("Failed to read resolved sections of:", entry, error);
    } finally {
      for (const name of wanted) {
        this.pending.delete(name);
      }
    }
  }

  /**
   * Forget the resolution, when the project behind it closes.
   */
  public clear(): void {
    runInAction(() => this.reset(null));
  }

  /**
   * Drop everything held for the previous entry point.
   *
   * Narrowing survives, because it is a decision about the config someone selected rather than about the root that
   * resolves it: clearing it here made opening a root undo the narrowing the selection had just asked for.
   *
   * @param entry - Entry point taking its place, or null when none is.
   */
  private reset(entry: Nullable<string>): void {
    this.entry = entry;
    this.index = this.index.asIdle(null);
    this.sections.clear();
    this.pending.clear();
    this.revision += 1;
  }
}
