import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
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

/** The query nothing has been typed into. */
export const EMPTY_FIND_QUERY: IConfigsFindQuery = { field: "", section: "", value: "" };

/**
 * Where the open project writes a section, a field or a value.
 *
 * The answer is authored statements, declarations and patches alike; which of them survives is the Resolved view's
 * question. Held across the panel closing, so a search reopened shows what it found rather than asking again.
 */
@Injectable()
export class ConfigsFindService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The query as the panel shows it, searched or not. */
  @Observable()
  public query: IConfigsFindQuery = EMPTY_FIND_QUERY;

  /** What the last search found. */
  @Observable()
  public results: AsyncState<ConfigsDeclarationSearch> = AsyncState.idle();

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Change part of the query without searching.
   *
   * @param query - The fields to replace.
   */
  public setQuery(query: Partial<IConfigsFindQuery>): void {
    runInAction(() => {
      this.query = { ...this.query, ...query };
    });
  }

  /**
   * Search the open project for the current query.
   */
  @LatestFlow("results")
  public *find(): TFlow {
    const sessionId: Nullable<string> = this.projectService.project.sessionId;
    const section: Nullable<string> = toAsked(this.query.section);
    const field: Nullable<string> = toAsked(this.query.field);
    const value: Nullable<string> = toAsked(this.query.value);

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
    this.setQuery({ ...EMPTY_FIND_QUERY, section });

    await this.find();
  }

  /**
   * Cancel the pending search and forget the query and its answer when the project goes.
   */
  public clear(): void {
    cancelFlow(this, "results");

    runInAction(() => {
      this.query = EMPTY_FIND_QUERY;
      this.results = this.results.asIdle(null);
    });
  }
}

/**
 * One field of the query as the backend takes it: as typed, or null when empty.
 *
 * Not trimmed: a header keeps its padding in the name it records, so `[ wpn_base ]` is a section of its own, and the
 * panel answers exactly what `ltx find` would.
 *
 * @param typed - The field's text.
 * @returns The text to ask for, or null for nothing of that kind.
 */
function toAsked(typed: string): Nullable<string> {
  return typed === "" ? null : typed;
}
