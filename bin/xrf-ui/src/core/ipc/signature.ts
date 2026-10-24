/** The parameters of a generated wrapper `F`, in order. */
type IpcParameters<F> = F extends (...parameters: infer P) => unknown ? P : never;

/**
 * The record a generated wrapper `F` sends its call with, its parameters keyed by the names `N` gives them in order.
 *
 * Read off the wrapper rather than written out beside it, so it is the wrapper's own types, retyped enums included.
 */
export type IpcArguments<F, N extends ReadonlyArray<string>> = {
  [K in keyof N as K extends `${number}` ? N[K] : never]: IpcParameters<F>[K & keyof IpcParameters<F>];
};

/** What one generated command takes, as the record it is invoked with, and what it resolves to. */
export interface IIpcCommandSignature<F, N extends ReadonlyArray<string>> {
  args: IpcArguments<F, N>;
  result: F extends (...parameters: never) => Promise<infer R> ? R : never;
}

/** What one generated bulk route takes, as the record it is fetched with; it answers bytes. */
export interface IIpcBulkRouteSignature<F, N extends ReadonlyArray<string>> {
  args: IpcArguments<F, N>;
}
