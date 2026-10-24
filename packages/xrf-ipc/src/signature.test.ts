import { describe, expect, it } from "@jest/globals";

import { IIpcBulkRouteSignature, IIpcCommandSignature, IpcArguments } from "#/signature";

/** Whether two types are the same both ways, so a test fails to compile where a signature reads a wrapper wrong. */
type IsSame<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

/** A wrapper of two parameters, as the generator writes one. */
type Open = (sessionId: string, index: number) => Promise<{ name: string }>;

/** A wrapper of none. */
type List = () => Promise<Array<string>>;

describe("IPC signatures", () => {
  it("keys a wrapper's parameters by their names", () => {
    const isSame: IsSame<IpcArguments<Open, ["sessionId", "index"]>, { sessionId: string; index: number }> = true;

    expect(isSame).toBe(true);
  });

  it("reads what a command resolves to", () => {
    const isSame: IsSame<IIpcCommandSignature<Open, ["sessionId", "index"]>["result"], { name: string }> = true;

    expect(isSame).toBe(true);
  });

  it("gives a command without parameters an empty record", () => {
    const isSame: IsSame<IIpcCommandSignature<List, []>["args"], object> = true;

    expect(isSame).toBe(true);
  });

  it("types a bulk route's arguments the same way", () => {
    const isSame: IsSame<
      IIpcBulkRouteSignature<Open, ["sessionId", "index"]>["args"],
      { sessionId: string; index: number }
    > = true;

    expect(isSame).toBe(true);
  });
});
