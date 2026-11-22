import { afterEach, describe, expect, it } from "@jest/globals";

import { LogTextWriter } from "#/log-text/log-text-writer";

describe("LogTextWriter", () => {
  afterEach(() => {
    Reflect.deleteProperty(globalThis, "Node");
  });

  it("joins values with a space, keeping strings as they are", () => {
    expect(LogTextWriter.write(["Opened", "C:\\game\\gamedata", 3, true, null, undefined])).toBe(
      "Opened C:\\game\\gamedata 3 true null undefined"
    );
  });

  it("writes an error as its stack, which carries its message", () => {
    const error: Error = new Error("could not read");

    expect(LogTextWriter.write(["Failed:", error])).toBe(`Failed: ${error.stack}`);
    expect(LogTextWriter.write([error])).toContain("could not read");
  });

  it("writes an error without a stack as its name and message", () => {
    const error: TypeError = new TypeError("bad input");

    error.stack = undefined;

    expect(LogTextWriter.write([error])).toBe("TypeError: bad input");
  });

  it("adds the message to a stack that leaves it out", () => {
    const error: Error = new Error("lost");

    error.stack = "    at somewhere (file.ts:1:1)";

    expect(LogTextWriter.write([error])).toBe("Error: lost\n    at somewhere (file.ts:1:1)");
  });

  it("writes other values as JSON", () => {
    expect(LogTextWriter.write([{ name: "level", sizes: [1, 2], nested: { isOn: false, label: "a" } }])).toBe(
      '{"name":"level","sizes":[1,2],"nested":{"isOn":false,"label":"a"}}'
    );
  });

  it("replaces values past the fifth level with a marker", () => {
    const deep: unknown = { a: { b: { c: { d: { e: { f: 1 } } } } }, list: [[[[[[1]]]]]] };

    expect(LogTextWriter.write([deep])).toBe('{"a":{"b":{"c":{"d":{"e":[Object]}}}},"list":[[[[[Array]]]]]}');
  });

  it("marks a reference back to a container, but writes a value shared by siblings in full", () => {
    const shared: { id: number } = { id: 7 };
    const looped: Record<string, unknown> = { first: shared, second: shared };

    looped.self = looped;

    expect(LogTextWriter.write([looped])).toBe('{"first":{"id":7},"second":{"id":7},"self":[Circular]}');
  });

  it("writes values JSON has no form for as readable markers", () => {
    expect(LogTextWriter.write([{ big: 12n, symbol: Symbol("tag"), named: parseInt }])).toBe(
      '{"big":12n,"symbol":Symbol(tag),"named":[Function parseInt]}'
    );
  });

  it("writes dates, maps, sets, buffers and nested errors briefly", () => {
    const value: unknown = {
      at: new Date(Date.UTC(2026, 10, 22)),
      map: new Map([["a", 1]]),
      set: new Set(["x"]),
      bytes: new Uint8Array(16),
      cause: new RangeError("too far"),
    };

    expect(LogTextWriter.write([value])).toBe(
      '{"at":"2026-11-22T00:00:00.000Z","map":Map(1) [["a",1]],"set":Set(1) ["x"],"bytes":Uint8Array(16 bytes),' +
        '"cause":"RangeError: too far"}'
    );
  });

  it("writes a DOM node as its name rather than walking it", () => {
    class FakeNode {
      public readonly nodeName: string = "DIV";
    }

    Object.assign(globalThis, { Node: FakeNode });

    expect(LogTextWriter.write(["Picked", new FakeNode()])).toBe("Picked [DIV]");
  });

  it("never throws, marking what cannot be read", () => {
    const hostile: object = {
      fine: 1,
      get broken(): never {
        throw new Error("getter");
      },
    };
    const trap: object = new Proxy(
      {},
      {
        ownKeys: () => {
          throw new Error("trap");
        },
      }
    );
    const lying: object = {
      toJSON: () => {
        throw new Error("toJSON");
      },
    };

    expect(LogTextWriter.write([hostile, trap, lying, "after"])).toBe(
      '{"fine":1,"broken":[Unreadable]} [Unreadable] [Unreadable] after'
    );
  });

  it("ends a toJSON that answers itself", () => {
    const selfish: Record<string, unknown> = {};

    selfish.toJSON = () => selfish;

    expect(LogTextWriter.write([selfish])).toBe("[Circular]");
  });

  it("cuts a long text with a marker", () => {
    const text: string = LogTextWriter.write(["x".repeat(100)], 10);

    expect(text).toBe("xxxxxxxxxx… [cut at 10 characters]");
  });

  it("stops walking a huge value once the cap is reached", () => {
    const huge: Array<number> = Array.from({ length: 100_000 }, (_: unknown, index: number) => index);
    const text: string = LogTextWriter.write([huge], 64);

    expect(text.startsWith("[0,1,2,3,")).toBe(true);
    expect(text.endsWith("… [cut at 64 characters]")).toBe(true);
    expect(text.length).toBeLessThan(100);
  });

  it("stops taking a huge set's values once the cap is reached", () => {
    class CountedSet extends Set<number> {
      public taken: number = 0;

      public override values(): SetIterator<number> {
        const values: SetIterator<number> = super.values();
        const next: SetIterator<number>["next"] = values.next.bind(values);

        values.next = (): ReturnType<typeof next> => {
          this.taken += 1;

          return next();
        };

        return values;
      }
    }

    const set: CountedSet = new CountedSet(Array.from({ length: 100_000 }, (_: unknown, index: number) => index));
    const text: string = LogTextWriter.write([set], 64);

    expect(text.startsWith("Set(100000) [0,1,2,")).toBe(true);
    expect(text.endsWith("… [cut at 64 characters]")).toBe(true);
    expect(set.taken).toBeLessThan(40);
  });

  it("stops taking a huge map's entries once the cap is reached", () => {
    class CountedMap extends Map<number, number> {
      public taken: number = 0;

      public override entries(): MapIterator<[number, number]> {
        const entries: MapIterator<[number, number]> = super.entries();
        const next: MapIterator<[number, number]>["next"] = entries.next.bind(entries);

        entries.next = (): ReturnType<typeof next> => {
          this.taken += 1;

          return next();
        };

        return entries;
      }
    }

    const map: CountedMap = new CountedMap(
      Array.from({ length: 100_000 }, (_: unknown, index: number): [number, number] => [index, 0])
    );
    const text: string = LogTextWriter.write([map], 64);

    expect(text.startsWith("Map(100000) [[0,0],[1,0],")).toBe(true);
    expect(text.endsWith("… [cut at 64 characters]")).toBe(true);
    expect(map.taken).toBeLessThan(20);
  });

  describe("the length cap", () => {
    const MARKER: string = `… [cut at ${LogTextWriter.MAX_LENGTH} characters]`;

    function nest(depth: number): Record<string, unknown> {
      const node: Record<string, unknown> = {};

      for (let index: number = 0; index < 6; index += 1) {
        node[`key${index}`] = depth > 0 ? nest(depth - 1) : "v".repeat(40);
      }

      return node;
    }

    it("fills the line from a large nested object and marks the cut", () => {
      const text: string = LogTextWriter.write(["state", nest(4)]);

      expect(text).toHaveLength(LogTextWriter.MAX_LENGTH + MARKER.length);
      expect(text.endsWith(MARKER)).toBe(true);
    });

    it("fills the line from a large flat array and marks the cut", () => {
      const huge: Array<number> = Array.from({ length: 100_000 }, (_: unknown, index: number) => index);
      const text: string = LogTextWriter.write([huge]);

      expect(text).toHaveLength(LogTextWriter.MAX_LENGTH + MARKER.length);
      expect(text.endsWith(MARKER)).toBe(true);
    });

    it("does not mark a value that fits exactly", () => {
      expect(LogTextWriter.write([["a", "b"]], 9)).toBe('["a","b"]');
      expect(LogTextWriter.write([["a", "b"]], 8)).toBe('["a","b"… [cut at 8 characters]');
    });

    it("shares one budget between the values of a line", () => {
      const text: string = LogTextWriter.write([{ first: "a".repeat(5_000) }, { second: "b".repeat(5_000) }]);

      expect(text).toHaveLength(LogTextWriter.MAX_LENGTH + MARKER.length);
      expect(text.startsWith('{"first":"aaa')).toBe(true);
      expect(text).toContain('} {"second":"bbb');
      expect(text.endsWith(`b${MARKER}`)).toBe(true);
    });
  });
});
