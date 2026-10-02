import { describe, expect, it, jest } from "@jest/globals";

import { IRendererFetchBatch } from "#/contract/scene/renderer-fetch-batch";
import { toFetchPart, toStream } from "#/texture/fetch-batch-fixtures";
import { RendererFetchBatches } from "#/texture/renderer-fetch-batches";
import { IRendererFetchedBytes } from "#/texture/renderer-fetched-bytes";

const URL: string = "http://127.0.0.1:1/batch";
const HEADERS: Record<string, string> = { Authorization: "Bearer token" };

/** A batch's post as the server got it, answered when the test says. */
interface IPost {
  url: string;
  init: RequestInit;
  answer: (response: Response) => void;
}

function toBatch(name: string, url: string = URL): IRendererFetchBatch {
  return { call: JSON.stringify({ args: { logicalPath: name }, route: "assets/read_asset" }), url };
}

function mockServer(): { posts: Array<IPost>; batches: RendererFetchBatches } {
  const posts: Array<IPost> = [];
  const post = jest.fn(
    (url: string, init: RequestInit): Promise<Response> =>
      new Promise((resolve, reject) => {
        posts.push({ answer: resolve, init, url });
        init.signal?.addEventListener("abort", () => reject(init.signal?.reason));
      })
  );

  return { batches: new RendererFetchBatches(post), posts };
}

/** Lets the microtask that sends what waits run. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

function toAnswer(...parts: Array<Uint8Array>): Response {
  const bytes: Uint8Array = new Uint8Array(parts.reduce((total, part) => total + part.length, 0));

  parts.reduce((at, part) => (bytes.set(part, at), at + part.length), 0);

  return new Response(toStream(bytes, 7), { status: 200 });
}

function toText(fetched: IRendererFetchedBytes): string {
  return new TextDecoder().decode(fetched.bytes);
}

describe("RendererFetchBatches", () => {
  it("posts requests made together as one batch, and answers each by its part in any order", async () => {
    const { batches, posts } = mockServer();
    const signal: AbortSignal = new AbortController().signal;
    const a: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("a"), HEADERS, signal);
    const b: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("b"), HEADERS, signal);

    await settle();

    expect(posts).toHaveLength(1);
    expect(posts[0].url).toBe(URL);
    expect(posts[0].init).toMatchObject({ headers: HEADERS, method: "POST" });
    expect(JSON.parse(posts[0].init.body as string)).toEqual([
      { args: { logicalPath: "a" }, route: "assets/read_asset" },
      { args: { logicalPath: "b" }, route: "assets/read_asset" },
    ]);

    posts[0].answer(
      toAnswer(
        toFetchPart(1, 200, "image/png", [...new TextEncoder().encode("B")]),
        toFetchPart(0, 200, "application/octet-stream", [...new TextEncoder().encode("A")])
      )
    );

    const [first, second] = await Promise.all([a, b]);

    expect(toText(first)).toBe("A");
    expect(first.type).toBe("application/octet-stream");
    expect(toText(second)).toBe("B");
    expect(second.type).toBe("image/png");
  });

  it("refuses only the call its part refuses, with the server's message", async () => {
    const { batches, posts } = mockServer();
    const signal: AbortSignal = new AbortController().signal;
    const a: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("a"), HEADERS, signal);
    const b: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("b"), HEADERS, signal);

    await settle();
    posts[0].answer(
      toAnswer(
        toFetchPart(0, 500, "application/json", [...new TextEncoder().encode("\"Failed to read asset 'a'\"")]),
        toFetchPart(1, 200, "application/octet-stream", [7])
      )
    );

    await expect(a).rejects.toThrow("Failed to read asset 'a'");
    expect(new Uint8Array((await b).bytes)).toEqual(new Uint8Array([7]));
  });

  it("refuses every call of a batch refused whole, or of one ending without its part", async () => {
    const { batches, posts } = mockServer();
    const signal: AbortSignal = new AbortController().signal;
    const refused: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("a"), HEADERS, signal);

    await settle();
    posts[0].answer(new Response(JSON.stringify("The transport token is missing"), { status: 401 }));
    await expect(refused).rejects.toThrow("The transport token is missing");

    const answered: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("b"), HEADERS, signal);
    const dropped: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("c"), HEADERS, signal);

    await settle();
    posts[1].answer(toAnswer(toFetchPart(0, 200, "application/octet-stream", [1])));

    await expect(answered).resolves.toBeDefined();
    await expect(dropped).rejects.toThrow("The batch ended without answering it");
  });

  it("keeps requests to other places, or with other headers, in batches of their own", async () => {
    const { batches, posts } = mockServer();
    const signal: AbortSignal = new AbortController().signal;

    void batches.fetch(toBatch("a"), HEADERS, signal);
    void batches.fetch(toBatch("b", "http://127.0.0.1:2/batch"), HEADERS, signal);
    void batches.fetch(toBatch("c"), { Authorization: "Bearer other" }, signal);
    void batches.fetch(toBatch("d"), HEADERS, signal);
    await settle();

    expect(posts.map(({ url, init }: IPost) => [url, JSON.parse(init.body as string).length])).toEqual([
      [URL, 2],
      ["http://127.0.0.1:2/batch", 1],
      [URL, 1],
    ]);
  });

  it("holds what is asked for while every batch allowed is out, and sends it together as one is read", async () => {
    const { batches, posts } = mockServer();
    const signal: AbortSignal = new AbortController().signal;
    const first: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("first"), HEADERS, signal);

    for (const name of ["x", "y", "z"]) {
      await settle();
      void batches.fetch(toBatch(name), HEADERS, signal);
    }

    await settle();
    expect(posts).toHaveLength(4);

    for (const name of ["p", "q", "r"]) {
      void batches.fetch(toBatch(name), HEADERS, signal);
    }

    await settle();
    expect(posts).toHaveLength(4);

    posts[0].answer(toAnswer(toFetchPart(0, 200, "application/octet-stream", [1])));
    await first;
    await settle();

    expect(posts).toHaveLength(5);
    expect(JSON.parse(posts[4].init.body as string)).toHaveLength(3);
  });

  it("sends no request aborted while it waits, and aborts a batch once all its calls are", async () => {
    const { batches, posts } = mockServer();
    const waiting: AbortController = new AbortController();
    const kept: AbortSignal = new AbortController().signal;
    const aborted: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("a"), HEADERS, waiting.signal);

    void batches.fetch(toBatch("b"), HEADERS, kept);
    waiting.abort();
    await expect(aborted).rejects.toBeDefined();
    await settle();

    expect(JSON.parse(posts[0].init.body as string)).toEqual([
      { args: { logicalPath: "b" }, route: "assets/read_asset" },
    ]);

    const one: AbortController = new AbortController();
    const other: AbortController = new AbortController();
    const first: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("c"), HEADERS, one.signal);
    const second: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("d"), HEADERS, other.signal);

    await settle();
    one.abort();
    await expect(first).rejects.toBeDefined();
    expect(posts[1].init.signal?.aborted).toBe(false);

    other.abort();
    await expect(second).rejects.toBeDefined();
    expect(posts[1].init.signal?.aborted).toBe(true);
  });

  it("refuses a request already aborted without sending it", async () => {
    const { batches, posts } = mockServer();
    const controller: AbortController = new AbortController();

    controller.abort();

    await expect(batches.fetch(toBatch("a"), HEADERS, controller.signal)).rejects.toBeDefined();
    await settle();
    expect(posts).toHaveLength(0);
  });

  it("aborts a batch whose answer breaks, so nothing more of it is read", async () => {
    const { batches, posts } = mockServer();
    const fetched: Promise<IRendererFetchedBytes> = batches.fetch(toBatch("a"), HEADERS, new AbortController().signal);

    await settle();
    posts[0].answer(toAnswer(toFetchPart(0, 200, "application/octet-stream", [1, 2, 3]).subarray(0, 14)));

    await expect(fetched).rejects.toThrow("ended inside a part");
    expect(posts[0].init.signal?.aborted).toBe(true);
  });
});
