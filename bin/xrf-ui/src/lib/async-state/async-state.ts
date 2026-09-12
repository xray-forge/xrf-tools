import { Nullable } from "@/lib/types/general";

/** Lifecycle status, independent of whether a resource has a value. */
export const enum EAsyncStatus {
  IDLE = "idle",
  LOADING = "loading",
  READY = "ready",
  FAILED = "failed",
}

/** A failure is required only for the failed state. */
type TAsyncState<E> =
  | { readonly status: EAsyncStatus.IDLE | EAsyncStatus.LOADING | EAsyncStatus.READY }
  | { readonly status: EAsyncStatus.FAILED; readonly error: E };

/**
 * Immutable lifecycle and value for an asynchronous resource.
 *
 * Loading and failure can retain previous content. A ready value may be empty; an idle value may be a fallback.
 */
export class AsyncState<T, E = Error> {
  /**
   * Creates an idle resource with an optional fallback.
   *
   * @param value - Initial fallback, or null when no value is available.
   * @returns A new idle state.
   */
  public static idle<T, E = Error>(value: Nullable<T> = null): AsyncState<T, E> {
    return new AsyncState<T, E>(value, { status: EAsyncStatus.IDLE });
  }

  /**
   * Creates a resource for a successful request, including an empty result.
   *
   * @param value - Successful result.
   * @returns A new ready state.
   */
  public static ready<T, E = Error>(value: Nullable<T>): AsyncState<T, E> {
    return new AsyncState<T, E>(value, { status: EAsyncStatus.READY });
  }

  /**
   * Creates a resource for a request in progress.
   *
   * @param value - Optional content to expose while loading.
   * @returns A new loading state.
   */
  public static loading<T, E = Error>(value: Nullable<T> = null): AsyncState<T, E> {
    return new AsyncState<T, E>(value, { status: EAsyncStatus.LOADING });
  }

  /**
   * Creates a resource for a failed request.
   *
   * @param error - Failure to expose.
   * @param value - Optional content to expose after failure.
   * @returns A new failed state.
   */
  public static failed<T, E = Error>(error: E, value: Nullable<T> = null): AsyncState<T, E> {
    return new AsyncState<T, E>(value, { status: EAsyncStatus.FAILED, error });
  }

  private constructor(
    public readonly value: Nullable<T>,
    private readonly state: TAsyncState<E>
  ) {}

  /** @returns The resource lifecycle, independent of its value. */
  public get status(): EAsyncStatus {
    return this.state.status;
  }

  /** @returns Whether the resource has been initialized or reset without a completed request. */
  public get isIdle(): boolean {
    return this.status === EAsyncStatus.IDLE;
  }

  /** @returns Whether a request is in progress. */
  public get isLoading(): boolean {
    return this.status === EAsyncStatus.LOADING;
  }

  /** @returns Whether the request succeeded, including an empty result. */
  public get isReady(): boolean {
    return this.status === EAsyncStatus.READY;
  }

  /** @returns Whether the request failed. */
  public get isFailed(): boolean {
    return this.status === EAsyncStatus.FAILED;
  }

  /** @returns The failure, or null outside the failed state. */
  public get error(): Nullable<E> {
    return this.state.status === EAsyncStatus.FAILED ? this.state.error : null;
  }

  /**
   * Projects an available value while preserving its lifecycle and failure.
   *
   * @param project - Projection applied only when a value is present.
   * @returns The projected state.
   */
  public map<U>(project: (value: T) => U): AsyncState<U, E> {
    return new AsyncState(this.value === null ? null : project(this.value), this.state);
  }

  /**
   * Resets the resource and clears its failure and previous value.
   *
   * @param value - Optional fallback for the reset resource.
   * @returns A new idle state.
   */
  public asIdle(value: Nullable<T> = null): AsyncState<T, E> {
    return AsyncState.idle<T, E>(value);
  }

  /**
   * Marks the resource ready and clears any previous failure.
   *
   * @param value - Ready value, defaulting to the current value.
   * @returns A new ready state.
   */
  public asReady(value: Nullable<T> = this.value): AsyncState<T, E> {
    return AsyncState.ready<T, E>(value);
  }

  /**
   * Marks the resource loading and clears any previous failure.
   *
   * @param value - Value to retain while loading.
   * @returns A new loading state.
   */
  public asLoading(value: Nullable<T> = this.value): AsyncState<T, E> {
    return AsyncState.loading<T, E>(value);
  }

  /**
   * Marks the resource failed while optionally retaining the previous value.
   *
   * @param error - Failure to expose.
   * @param value - Value to retain after failure.
   * @returns A new failed state.
   */
  public asFailed(error: E, value: Nullable<T> = this.value): AsyncState<T, E> {
    return AsyncState.failed<T, E>(error, value);
  }
}
