import { inject, Injectable } from "@wirestate/core";
import { Computed, Observable, runInAction } from "@wirestate/mobx";

import { configsCommands } from "@/core/bindings/commands/configs";
import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

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

  /** The selected config's lines and structure. */
  @Observable()
  public document: Loadable<Nullable<ConfigsDocument>> = Loadable.idle(null);

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
   */
  @LatestFlow("document")
  public *retry(): TFlow {
    if (this.selected) {
      yield* this.select(this.selected);
    }
  }

  /**
   * Forget whatever is on screen, when the project behind it closes.
   */
  public clear(): void {
    runInAction(() => {
      this.selected = null;
      this.document = this.document.asIdle(null);
    });
  }
}
