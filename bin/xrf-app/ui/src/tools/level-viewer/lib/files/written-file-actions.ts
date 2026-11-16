import { TFlow } from "@xrf/flow";
import { systemCommands } from "@xrf/ipc/commands/system";
import { Nullable } from "@xrf/types";

import { CommandReporter } from "@/core/operations/lib/command-reporter";
import { copyLevelViewerImage, ILevelViewerImage } from "@/tools/level-viewer/lib/files/level-viewer-image";

/** What a written file is called: `Show screenshot report` in the log, `Could not show the screenshot's report`. */
export interface IWrittenFileNouns {
  /** The file as log lines name it: `screenshot report`. */
  name: string;
  /** The file as a failure notice names it, where that reads otherwise than `name`: `screenshot's report`. */
  notice?: string;
}

/** The file a {@link WrittenFileActions} acts on, and how it is called. */
export interface IWrittenFileActionsOptions<T> {
  nouns: IWrittenFileNouns;
  /** The file acted on now, or null for none, read as each action starts. */
  current: () => Nullable<T>;
  /** Where `file` is on disk. */
  toPath: (file: T) => string;
}

/**
 * What is done with a file the level viewer wrote, each reported as one command through its owner's reporter: deleting,
 * showing it in its folder, and copying its path or its image. Each acts on the file current as it starts and does
 * nothing where there is none.
 *
 * The owner runs them with `yield*` inside its own flows, which decide what supersedes what.
 */
export class WrittenFileActions<T> {
  /**
   * @param reporter - The owning service's reporter, so the lines and notices name the service and its tool.
   * @param options - The file acted on and how it is called.
   */
  public constructor(
    private readonly reporter: CommandReporter,
    private readonly options: IWrittenFileActionsOptions<T>
  ) {}

  /**
   * Deletes the file for good.
   *
   * @param command - Deletes `file`.
   * @returns Whether a file was deleted.
   */
  public *remove(command: (file: T) => Promise<unknown>): TFlow<boolean> {
    const file: Nullable<T> = this.options.current();

    if (!file) {
      return false;
    }

    const { error } = yield* this.reporter.run({
      name: `Delete ${this.name}`,
      subject: this.options.toPath(file),
      command: () => command(file),
      failure: `Could not delete the ${this.notice}`,
    });

    return !error;
  }

  /** Shows the file selected in its folder. */
  public *reveal(): TFlow {
    yield* this.attempt(`Show ${this.name}`, `Could not show the ${this.notice}`, (file: T) =>
      systemCommands.revealPath(this.options.toPath(file))
    );
  }

  /** Puts the file's path on the clipboard. */
  public *copyPath(): TFlow {
    yield* this.attempt(`Copy ${this.name} path`, `Could not copy the ${this.notice}'s path`, (file: T) =>
      navigator.clipboard.writeText(this.options.toPath(file))
    );
  }

  /**
   * Puts the file's image on the clipboard.
   *
   * @param toImage - The image `file` holds.
   */
  public *copyImage(toImage: (file: T) => ILevelViewerImage): TFlow {
    yield* this.attempt(`Copy ${this.name}`, `Could not copy the ${this.notice}`, (file: T) =>
      copyLevelViewerImage(toImage(file))
    );
  }

  /**
   * Does something else with the file, reported as one command.
   *
   * @param name - What it does, as its log lines name it.
   * @param failure - The failure notice's title.
   * @param action - Does it with the file.
   */
  public *attempt(name: string, failure: string, action: (file: T) => Promise<unknown>): TFlow {
    const file: Nullable<T> = this.options.current();

    if (file) {
      yield* this.reporter.run({ name, subject: this.options.toPath(file), command: () => action(file), failure });
    }
  }

  private get name(): string {
    return this.options.nouns.name;
  }

  private get notice(): string {
    return this.options.nouns.notice ?? this.options.nouns.name;
  }
}
