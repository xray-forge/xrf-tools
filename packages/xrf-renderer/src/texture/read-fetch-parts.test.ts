import { describe, expect, it } from "@jest/globals";

import { toFetchPart, toStream } from "#/texture/fetch-batch-fixtures";
import { IFetchPart } from "#/texture/fetch-part";
import { readFetchParts } from "#/texture/read-fetch-parts";

describe("readFetchParts", () => {
  it("reads every part however the stream cuts it, each its own buffer", async () => {
    const answer: Uint8Array = new Uint8Array([
      ...toFetchPart(1, 200, "application/octet-stream", [1, 2, 3]),
      ...toFetchPart(0, 500, "application/json", [...new TextEncoder().encode('"refused"')]),
      ...toFetchPart(2, 200, "", []),
    ]);

    for (const size of [1, 5, 13, answer.length]) {
      const parts: Array<IFetchPart> = [];

      await readFetchParts(toStream(answer, size), (part: IFetchPart) => parts.push(part));

      expect(parts.map(({ index, status, type }: IFetchPart) => [index, status, type])).toEqual([
        [1, 200, "application/octet-stream"],
        [0, 500, "application/json"],
        [2, 200, ""],
      ]);
      expect([...new Uint8Array(parts[0].bytes)]).toEqual([1, 2, 3]);
      expect(parts[0].bytes.byteLength).toBe(3);
      expect(parts[2].bytes.byteLength).toBe(0);
    }
  });

  it("refuses an answer ending inside a part", async () => {
    const part: Uint8Array = toFetchPart(0, 200, "application/octet-stream", [1, 2, 3]);

    await expect(readFetchParts(toStream(part.subarray(0, part.length - 1), 4), () => {})).rejects.toThrow(
      "The batch's answer ended inside a part"
    );
  });
});
