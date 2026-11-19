import { inject, Injectable, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, Observable, reaction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { configsCommands } from "@xrf/ipc/commands/configs";
import { ConfigsDeclarationSearch } from "@xrf/ipc/types/xrf-app-configs";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";

/** What a reader asks the project for, as typed: an empty field asks nothing of that kind. */
export interface IConfigsFindQuery {
  /** Section name without brackets or operation prefix. */
  section: string;
  /** Field name without operation prefix. */
  field: string;
  /** Text a written value must contain. */
  value: string;
}

/**
 * Where the open project writes a section, a field or a value.
 *
 * Finds authored statements, declarations and patches alike, not what survives resolution. Kept while the panel is
 * closed, so a reopened search shows its last answer.
 */
@Injectable()
export class ConfigsFindService {
  /** The query nothing has been typed into. */
  private static readonly EMPTY_QUERY: IConfigsFindQuery = { field: "", section: "", value: "" };

  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The query as the panel shows it, searched or not. */
  @Observable()
  public query: IConfigsFindQuery = ConfigsFindService.EMPTY_QUERY;

  /** What the last search found. */
  @Observable()
  public results: AsyncState<ConfigsDeclarationSearch> = AsyncState.idle();

  private stopFollowing: Nullable<() => void> = null;

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  @OnProvision()
  public onProvision(): void {
    this.stopFollowing?.();
    // Rows point into one session of the project; in another session their lines and files may have moved.
    this.stopFollowing = reaction(
      () => this.projectService.project.sessionId,
      () => this.clear()
    );
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.stopFollowing?.();
    this.stopFollowing = null;
  }

  /**
   * Change part of the query without searching.
   *
   * @param query - The fields to replace.
   */
  @BoundAction()
  public setQuery(query: Partial<IConfigsFindQuery>): void {
    this.query = { ...this.query, ...query };
  }

  /**
   * Search the open project for the current query.
   */
  @LatestFlow("results")
  public *find(): TFlow {
    const sessionId: Nullable<string> = this.projectService.project.sessionId;
    const section: Nullable<string> = ConfigsFindService.toAsked(this.query.section);
    const field: Nullable<string> = ConfigsFindService.toAsked(this.query.field);
    const value: Nullable<string> = ConfigsFindService.toAsked(this.query.value);

    if (!sessionId) {
      this.log.error("Cannot search configs with no project open");

      return;
    }

    // Nothing asked is not a search that found nothing, and the backend refuses it.
    if (section === null && field === null && value === null) {
      this.results = this.results.asIdle(null);

      return;
    }

    const timer: Timer = new Timer();

    this.results = this.results.asLoading();

    try {
      const { value: found }: SessionSnapshot<ConfigsDeclarationSearch> = yield* call(
        configsCommands.findDeclarations({ field, section, sessionId, value })
      );

      this.results = this.results.asReady(found);

      this.log.info("Declarations found:", found.total, "in", formatDuration(timer.elapsed()));
    } catch (error) {
      this.log.error("Failed to search the configs after", formatDuration(timer.elapsed()), error);

      this.results = this.results.asFailed(transformError(error));
    }
  }

  /**
   * Search for every header of one section, replacing whatever was asked before.
   *
   * @param section - Section name, as the document names it.
   */
  public async findSection(section: string): Promise<void> {
    this.setQuery({ ...ConfigsFindService.EMPTY_QUERY, section });

    await this.find();
  }

  /**
   * Cancel the pending search and forget the query and its answer.
   */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "results");

    this.query = ConfigsFindService.EMPTY_QUERY;
    this.results = this.results.asIdle(null);
  }

  /**
   * One field of the query as the backend takes it: as typed, or null when empty.
   *
   * Not trimmed: `[ wpn_base ]` is a section of its own, and the panel answers exactly what `ltx find` would.
   *
   * @param typed - The field's text.
   * @returns The text to ask for, or null for nothing of that kind.
   */
  private static toAsked(typed: string): Nullable<string> {
    return typed === "" ? null : typed;
  }
}
