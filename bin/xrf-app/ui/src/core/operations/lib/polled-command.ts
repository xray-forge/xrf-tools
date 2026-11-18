import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

export interface IPolledCommandOptions<T> {
  /** Reads the current value; null clears the last reading, and a throw keeps it. */
  read: () => Promise<Nullable<T>>;
  /** How long to wait between reads. */
  interval: number;
  /**
   * Whether two readings are the same, so the later notifies no watcher; `Object.is` by default, which a native read
   * answering a new value every time never meets, so a structural one suits a reading that rarely changes.
   */
  equals?: (previous: T, next: T) => boolean;
}

/**
 * A native read asked again on an interval for as long as any view watches it, so every view showing it shares one
 * poll. The sibling of `CommandOperation` for a reading that changes on its own, such as memory or the jobs listing.
 *
 * A hidden document skips its ticks and reads once as soon as it is shown again, since nobody is looking in between.
 * The reading is forgotten once the last view stops watching, so the next one starts from nothing rather than from an
 * old reading. A failed read raises no notice: the owner's `read` logs it and says what it leaves.
 */
export class PolledCommand<T> {
  @Observable()
  private reading: Nullable<T> = null;

  private watchers: number = 0;

  private timer: Nullable<ReturnType<typeof setInterval>> = null;

  private isReading: boolean = false;

  /** Counts the polls started, so a read landing after its poll stopped publishes nothing. */
  private generation: number = 0;

  /** Reads once as the document is shown again, having skipped the ticks it was hidden for. */
  private readonly onVisibilityChange = (): void => void this.poll();

  /**
   * @param log - The owning service's logger.
   * @param options - What is read and how often.
   */
  public constructor(
    private readonly log: Logger,
    private readonly options: IPolledCommandOptions<T>
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** The most recent reading, or null before the first lands and while nothing watches. */
  @Computed()
  public get value(): Nullable<T> {
    return this.reading;
  }

  /**
   * Starts polling for one more watcher; the first starts the poll with a read at once.
   *
   * @returns Stops this watcher; the last to stop ends the poll. Calling it again does nothing.
   */
  public watch(): () => void {
    let isWatching: boolean = true;

    this.watchers += 1;

    if (this.watchers === 1) {
      this.start();
    }

    return () => {
      if (!isWatching) {
        return;
      }

      isWatching = false;
      this.watchers -= 1;

      if (this.watchers === 0) {
        this.stop();
      }
    };
  }

  private start(): void {
    this.generation += 1;
    this.timer = setInterval(() => void this.poll(), this.options.interval);

    document.addEventListener("visibilitychange", this.onVisibilityChange);

    void this.poll();
  }

  private stop(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }

    document.removeEventListener("visibilitychange", this.onVisibilityChange);

    this.generation += 1;
    this.isReading = false;
    this.accept(null);
  }

  private async poll(): Promise<void> {
    if (this.isReading || document.visibilityState === "hidden") {
      return;
    }

    const generation: number = this.generation;

    this.isReading = true;

    try {
      const next: Nullable<T> = await this.options.read();

      if (generation === this.generation) {
        this.accept(next);
      }
    } catch (error: unknown) {
      this.log.warn("Unexpected error during polling:", error);
    } finally {
      if (generation === this.generation) {
        this.isReading = false;
      }
    }
  }

  @BoundAction()
  private accept(next: Nullable<T>): void {
    // A reading equal to the last one notifies nothing.
    if (!this.isSame(this.reading, next)) {
      this.reading = next;
    }
  }

  private isSame(previous: Nullable<T>, next: Nullable<T>): boolean {
    if (previous === null || next === null || !this.options.equals) {
      return Object.is(previous, next);
    }

    return this.options.equals(previous, next);
  }
}
