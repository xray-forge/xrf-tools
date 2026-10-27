import { inject, Injectable } from "@wirestate/core";
import { Computed, Observable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { configsCommands } from "@xrf/ipc/commands/configs";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { LtxStructureParseError } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { TConfigsReveal } from "@/tools/configs-explorer/lib/reveal";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

/** The two ways one config can be read. */
export enum EConfigsDocumentMode {
  /** The file as written, which is what its author edits. */
  AUTHORED = "authored",
  /** What its entry point resolves to, which is what the engine loads. */
  RESOLVED = "resolved",
}

/**
 * Which config is on screen, and what the backend says about it.
 *
 * One document at a time: the tree is the navigation and this is what it points at. A read is addressed by the open's
 * session id, so a document requested against a project since closed is refused rather than shown beside a tree it no
 * longer belongs to.
 */
@Injectable()
export class ConfigsDocumentService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Engine identity of the selected config, or null when none is selected. */
  @Observable()
  public selected: Nullable<string> = null;

  /**
   * Which of the two views the selected config is shown in.
   */
  @Observable()
  public mode: EConfigsDocumentMode = EConfigsDocumentMode.AUTHORED;

  /**
   * What a panel asked to be brought into view, if anything has.
   */
  @Observable()
  public revealed: Nullable<TConfigsReveal> = null;

  /**
   * The section the reader is looking at, which the Scheme panel explains.
   *
   * Set by a click in either view or in the Sections panel. It outlives a reveal on purpose: a reveal is spent by the
   * view that scrolls to it, while a selection stays until another section is picked.
   */
  @Observable()
  public selectedSection: Nullable<string> = null;

  /** The selected config's lines and structure. */
  @Observable()
  public document: AsyncState<ConfigsDocument> = AsyncState.idle();

  /**
   * @returns The entry point the selected config is judged against, or null when nothing reaches it.
   *
   * The first of them, which is the one the backend resolved the structure against. A config reached by two entry
   * points is read through the first in project order, and the structure names the others.
   */
  @Computed()
  public get entry(): Nullable<string> {
    return this.document.value?.structure.entryPoints[0] ?? null;
  }

  /**
   * @returns Whether the selected config could not be parsed, which is shown rather than refused.
   */
  @Computed()
  public get parseError(): Nullable<string> {
    const error: Nullable<LtxStructureParseError> = this.document.value?.structure.parseError ?? null;

    return error ? `${error.line}:${error.column} ${error.message}` : null;
  }

  public constructor(
    private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService),
    private readonly resolvedService: ConfigsResolvedService = inject(ConfigsResolvedService),
    private readonly findingsService: ConfigsFindingsService = inject(ConfigsFindingsService),
    private readonly schemeService: ConfigsSchemeService = inject(ConfigsSchemeService)
  ) {}

  /**
   * Show one config of the open project.
   *
   * @param path - Engine identity of the config to read.
   */
  @LatestFlow("document")
  public *select(path: string): TFlow {
    const sessionId: Nullable<string> = this.projectService.project.sessionId;

    if (!sessionId) {
      this.log.error("Cannot read a config with no project open:", path);

      return;
    }

    const timer: Timer = new Timer();

    this.selected = path;
    this.document = this.document.asLoading();

    try {
      const { value: document }: SessionSnapshot<ConfigsDocument> = yield* call(
        configsCommands.readDocument({ path, sessionId })
      );

      this.document = this.document.asReady(document);

      this.log.info("Config read:", path, document.findings.length, "findings, in", formatDuration(timer.elapsed()));
    } catch (error) {
      this.log.error("Failed to read the config:", path, "after", formatDuration(timer.elapsed()), error);

      // The path stays selected on a failure, so the row a person clicked keeps its highlight and a retry needs no
      // second search through the tree.
      this.document = this.document.asFailed(transformError(error));
    }
  }

  /**
   * Read the selected config again, for a retry after a failure.
   */
  public async retry(): Promise<void> {
    if (this.selected) {
      await this.select(this.selected);
    }
  }

  /**
   * Abandon pending reads and clear the document, resolution, findings, and scheme report together.
   * The project stays open, retaining its native caches for subsequent selections.
   */
  public clear(): void {
    cancelFlow(this, "document");

    runInAction(() => {
      this.selected = null;
      this.selectedSection = null;
      this.document = this.document.asIdle(null);
    });

    this.resolvedService.clear();
    this.findingsService.clear();
    this.schemeService.clear();
  }

  /**
   * Ask the open view to bring one section into sight, and make it the selection the Scheme panel explains.
   *
   * @param name - Section to reveal, as the view names it.
   */
  public revealSection(name: string): void {
    runInAction(() => {
      this.revealed = { kind: "section", section: name };
      this.selectedSection = name;
    });
  }

  /**
   * Name the section the reader is looking at, without moving the view.
   *
   * @param name - Section the click landed in, or null where it landed in none.
   */
  public selectSection(name: Nullable<string>): void {
    runInAction(() => {
      this.selectedSection = name;
    });
  }

  /**
   * Open one config at one of its lines, which is where a finding is anchored.
   *
   * @param path - Engine identity of the config to open.
   * @param line - One-based line to bring into view.
   */
  public async openAt(path: string, line: number): Promise<void> {
    this.setMode(EConfigsDocumentMode.AUTHORED);

    if (this.selected !== path || !this.document.isReady) {
      try {
        await this.select(path);
      } catch {
        // The read was superseded, which means something else is being opened and this jump is stale. A failure of the
        // read itself is not thrown: `select` keeps it in `document` for the view to show.
        return;
      }
    }

    if (this.selected === path && this.document.isReady) {
      runInAction(() => {
        this.revealed = { kind: "line", line };
      });
    }
  }

  /**
   * Forget a reveal the view has acted on, so the same place can be asked for again.
   */
  public clearRevealed(): void {
    runInAction(() => {
      this.revealed = null;
    });
  }

  /**
   * Show the selected config in one view or the other.
   *
   * @param mode - View to switch to.
   */
  public setMode(mode: EConfigsDocumentMode): void {
    runInAction(() => {
      this.mode = mode;
    });
  }
}
