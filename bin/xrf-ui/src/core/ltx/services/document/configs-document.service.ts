import { inject, Injectable } from "@wirestate/core";
import { Computed, Observable, runInAction } from "@wirestate/mobx";

import { configsCommands } from "@/core/bindings/commands/configs";
import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { TConfigsReveal } from "@/core/ltx/lib/reveal";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** The two ways one config can be read. */
export const enum EConfigsDocumentMode {
  /** The file as written, which is what an editor will later change. */
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
    const error = this.document.value?.structure.parseError;

    return error ? `${error.line}:${error.column} ${error.message}` : null;
  }

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Show one config of the open project.
   *
   * A superseding selection cancels this one rather than racing it, so clicking down a tree leaves the last click on
   * screen instead of whichever read happened to finish last.
   *
   * @param path - Engine identity of the config to read.
   */
  @LatestFlow("document")
  public *select(path: string): TFlow {
    const sessionId: Nullable<string> = this.projectService.sessionId;

    if (!sessionId) {
      this.log.error("Cannot read a config with no project open:", path);

      return;
    }

    this.selected = path;
    this.document = this.document.asLoading();

    try {
      const document: ConfigsDocument = yield* call(configsCommands.readDocument({ path, sessionId }));

      this.document = this.document.asReady(document);
    } catch (error) {
      this.log.error("Failed to read the config:", path, error);

      // The path stays selected on a failure, so the row a person clicked keeps its highlight and a retry needs no
      // second search through the tree.
      this.document = this.document.asFailed(transformError(error));
    }
  }

  /**
   * Read the selected config again, for a retry after a failure.
   *
   * Not a flow of its own: `select` already owns the document lane, and a flow starting another flow in the same lane
   * cancels itself doing it.
   */
  public async retry(): Promise<void> {
    if (this.selected) {
      await this.select(this.selected);
    }
  }

  /**
   * Forget whatever is on screen, when the project behind it closes.
   */
  public clear(): void {
    runInAction(() => {
      this.selected = null;
      this.selectedSection = null;
      this.document = this.document.asIdle(null);
    });
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
   * The reveal is set after the read lands rather than before it: a request made while another config is still on
   * screen would be spent by the view drawing that one. Authored, because a line belongs to a file - a resolved
   * document is assembled out of sections and holds no line of any config.
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
