import { Nullable } from "@/lib/types/general";

/** Lifecycle status, independent of whether a resource has a value. */
export const enum ELoadableStatus {
  IDLE = "idle",
  LOADING = "loading",
  READY = "ready",
  FAILED = "failed",
}

/** A failure is required only for the failed state. */
type TLoadableState<E> =
  | { readonly status: ELoadableStatus.IDLE | ELoadableStatus.LOADING | ELoadableStatus.READY }
  | { readonly status: ELoadableStatus.FAILED; readonly error: E };

/**
 * Immutable lifecycle and value for an asynchronous resource.
 *
 * Loading and failure can retain previous content. A ready value may be empty; an idle value may be a fallback.
 */
export class Loadable<T, E = Error> {
  /**
   * Creates an idle resource with an optional fallback.
   *
   * @param value - Initial fallback, or null when no value is available.
   * @returns A new idle state.
   */
  public static idle<T, E = Error>(value: Nullable<T> = null): Loadable<T, E> {
    return new Loadable<T, E>(value, { status: ELoadableStatus.IDLE });
  }

  /**
   * Creates a resource for a successful request, including an empty result.
   *
   * @param value - Successful result.
   * @returns A new ready state.
   */
  public static ready<T, E = Error>(value: Nullable<T>): Loadable<T, E> {
    return new Loadable<T, E>(value, { status: ELoadableStatus.READY });
  }

  /**
   * Creates a resource for a request in progress.
   *
   * @param value - Optional content to expose while loading.
   * @returns A new loading state.
   */
  public static loading<T, E = Error>(value: Nullable<T> = null): Loadable<T, E> {
    return new Loadable<T, E>(value, { status: ELoadableStatus.LOADING });
  }

  /**
   * Creates a resource for a failed request.
   *
   * @param error - Failure to expose.
   * @param value - Optional content to expose after failure.
   * @returns A new failed state.
   */
  public static failed<T, E = Error>(error: E, value: Nullable<T> = null): Loadable<T, E> {
    return new Loadable<T, E>(value, { status: ELoadableStatus.FAILED, error });
  }

  private constructor(
    public readonly value: Nullable<T>,
    private readonly state: TLoadableState<E>
  ) {}

  /** @returns The resource lifecycle, independent of its value. */
  public get status(): ELoadableStatus {
    return this.state.status;
  }

  /** @returns Whether the resource has been initialized or reset without a completed request. */
  public get isIdle(): boolean {
    return this.status === ELoadableStatus.IDLE;
  }

  /** @returns Whether a request is in progress. */
  public get isLoading(): boolean {
    return this.status === ELoadableStatus.LOADING;
  }

  /** @returns Whether the request succeeded, including an empty result. */
  public get isReady(): boolean {
    return this.status === ELoadableStatus.READY;
  }

  /** @returns Whether the request failed. */
  public get isFailed(): boolean {
    return this.status === ELoadableStatus.FAILED;
  }

  /** @returns The failure, or null outside the failed state. */
  public get error(): Nullable<E> {
    return this.state.status === ELoadableStatus.FAILED ? this.state.error : null;
  }

  /**
   * Resets the resource and clears its failure and previous value.
   *
   * @param value - Optional fallback for the reset resource.
   * @returns A new idle state.
   */
  public asIdle(value: Nullable<T> = null): Loadable<T, E> {
    return Loadable.idle<T, E>(value);
  }

  /**
   * Marks the resource ready and clears any previous failure.
   *
   * @param value - Ready value, defaulting to the current value.
   * @returns A new ready state.
   */
  public asReady(value: Nullable<T> = this.value): Loadable<T, E> {
    return Loadable.ready<T, E>(value);
  }

  /**
   * Marks the resource loading and clears any previous failure.
   *
   * @param value - Value to retain while loading.
   * @returns A new loading state.
   */
  public asLoading(value: Nullable<T> = this.value): Loadable<T, E> {
    return Loadable.loading<T, E>(value);
  }

  /**
   * Marks the resource failed while optionally retaining the previous value.
   *
   * @param error - Failure to expose.
   * @param value - Value to retain after failure.
   * @returns A new failed state.
   */
  public asFailed(error: E, value: Nullable<T> = this.value): Loadable<T, E> {
    return Loadable.failed<T, E>(error, value);
  }
}
