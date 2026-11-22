import { Nullable } from "@xrf/types";

import { LogLine } from "#/log-text/log-line";
import { ILogValuePlace } from "#/log-text/log-text-writer.types";

/**
 * Writes a logged line's values as one text, joined by a space as the console shows them.
 *
 * A string the line was logged with is kept as it is, and an `Error` is written as its stack. Any other value is
 * written as JSON to {@link LogTextWriter.MAX_DEPTH}, with a deeper object or array written as `[Object]` or `[Array]`
 * and a reference back to one of its own containers as `[Circular]`. Inside a value, an `Error` is written as its name
 * and message, a `Map` or `Set` as its size and items, and what JSON has no form for (`undefined`, a BigInt, a symbol, a
 * function, a DOM node, a binary buffer) as a readable marker. Nothing it is handed makes it throw: a value that cannot
 * be read, such as a property whose getter throws, is written as `[Unreadable]`. Walking stops once the line is cut, so
 * a huge value costs no more than the text it can show.
 */
export class LogTextWriter {
  /** How many characters a line's text keeps before it is cut and marked as cut. */
  public static readonly MAX_LENGTH: number = 8_192;

  /** How deep into nested objects and arrays a value is written before the rest is replaced by a marker. */
  public static readonly MAX_DEPTH: number = 5;

  private static readonly UNREADABLE: string = "[Unreadable]";

  /**
   * Writes one line's values.
   *
   * @param args - The values the line was logged with.
   * @param maxLength - How many characters the text keeps; past it the text is cut and ends with a marker saying so.
   * @returns The line's text.
   */
  public static write(args: ReadonlyArray<unknown>, maxLength: number = LogTextWriter.MAX_LENGTH): string {
    // A writer per line: a getter or `toJSON` that logs re-enters here, and must not share the line being written.
    const writer: LogTextWriter = new LogTextWriter(new LogLine(maxLength));

    writer.writeLine(args);

    return writer.line.toText();
  }

  /**
   * @param error - The error to write.
   * @returns Its stack, which carries its message; its name and message where it has no stack, or a stack that
   *   leaves the message out.
   */
  private static formatError(error: Error): string {
    const head: string = Error.prototype.toString.call(error);
    const stack: unknown = error.stack;

    if (typeof stack !== "string" || stack.length === 0) {
      return head;
    }

    return stack.includes(error.message) ? stack : `${head}\n${stack}`;
  }

  /**
   * @param value - An object inside the line.
   * @returns The marker it is written as without walking it, for an object whose fields say nothing or cost too
   *   much; null for a container to walk.
   */
  private static markerOf(value: object): Nullable<string> {
    if (value instanceof Error) {
      return JSON.stringify(Error.prototype.toString.call(value));
    } else if (value instanceof Date) {
      return Number.isNaN(value.getTime()) ? "Invalid Date" : JSON.stringify(value.toISOString());
    } else if (typeof Node === "function" && value instanceof Node) {
      return `[${value.nodeName}]`;
    } else if (typeof Window === "function" && value instanceof Window) {
      return "[Window]";
    } else if (ArrayBuffer.isView(value)) {
      return `${value.constructor.name}(${value.byteLength} bytes)`;
    } else if (value instanceof ArrayBuffer) {
      return `ArrayBuffer(${value.byteLength} bytes)`;
    }

    return null;
  }

  /**
   * @param value - An object inside the line.
   * @returns Whether it says how JSON writes it, as `JSON.stringify` asks.
   */
  private static hasToJson(value: object): value is { toJSON: () => unknown } {
    return typeof (value as { toJSON?: unknown }).toJSON === "function";
  }

  private readonly line: LogLine;

  private constructor(line: LogLine) {
    this.line = line;
  }

  /**
   * @param args - The values the line was logged with.
   */
  private writeLine(args: ReadonlyArray<unknown>): void {
    for (let index: number = 0; index < args.length && !this.line.isCut; index += 1) {
      if (index > 0) {
        this.line.emit(" ");
      }

      this.writeTopValue(args[index]);
    }
  }

  /**
   * Writes one of the line's values as the console would show it at the top of a line.
   *
   * @param value - The value.
   */
  private writeTopValue(value: unknown): void {
    this.line.emitOrFallBack(() => {
      if (typeof value === "string") {
        this.writeString(value, false);
      } else if (value instanceof Error) {
        this.line.emit(LogTextWriter.formatError(value));
      } else {
        this.writeValue(value, { depth: 0, ancestors: [] });
      }
    }, LogTextWriter.UNREADABLE);
  }

  /**
   * Writes a string, never copying more of it than the line has room for.
   *
   * @param text - The raw string.
   * @param isQuoted - Whether it is written as a JSON string, as it is inside a value.
   */
  private writeString(text: string, isQuoted: boolean): void {
    // One character past the room is enough for the cut to be seen: quoting and escaping only lengthen it.
    const kept: string = text.slice(0, this.line.room + 1);

    this.line.emit(isQuoted ? JSON.stringify(kept) : kept);
  }

  /**
   * Writes a value inside the line as JSON, or as a marker where JSON has no form for it.
   *
   * @param value - The value, at any depth.
   * @param place - Where it sits.
   */
  private writeValue(value: unknown, place: ILogValuePlace): void {
    switch (typeof value) {
      case "string":
        return this.writeString(value, true);
      case "number":
      case "boolean":
        return this.line.emit(String(value));
      case "bigint":
        return this.line.emit(`${value}n`);
      case "undefined":
        return this.line.emit("undefined");
      case "symbol":
        return this.line.emit(value.toString());
      case "function":
        return this.line.emit(value.name ? `[Function ${value.name}]` : "[Function]");
      case "object":
        if (value === null) {
          return this.line.emit("null");
        }

        // A proxy can throw from any trap, so one unreadable object must not cost the rest of the line.
        return this.line.emitOrFallBack(() => this.writeObject(value, place), LogTextWriter.UNREADABLE);
    }
  }

  /**
   * Writes an object inside the line as a marker where it is circular, says nothing worth walking or sits too deep,
   * and otherwise walks it as a container.
   *
   * @param value - The object.
   * @param place - Where it sits.
   */
  private writeObject(value: object, place: ILogValuePlace): void {
    if (place.ancestors.includes(value)) {
      return this.line.emit("[Circular]");
    }

    const marker: Nullable<string> = LogTextWriter.markerOf(value);

    if (marker !== null) {
      return this.line.emit(marker);
    } else if (place.depth >= LogTextWriter.MAX_DEPTH) {
      return this.line.emit(Array.isArray(value) ? "[Array]" : "[Object]");
    }

    const inside: ILogValuePlace = { depth: place.depth + 1, ancestors: [...place.ancestors, value] };

    if (value instanceof Map) {
      this.line.emit(`Map(${value.size}) `);
      this.writeItems(value.entries(), inside);
    } else if (value instanceof Set) {
      this.line.emit(`Set(${value.size}) `);
      this.writeItems(value.values(), inside);
    } else if (Array.isArray(value)) {
      this.writeArray(value, inside);
    } else if (LogTextWriter.hasToJson(value)) {
      // One level deeper, so a `toJSON` answering a fresh object with its own `toJSON` still ends.
      this.writeValue(value.toJSON(), inside);
    } else {
      this.writeRecord(value as Record<string, unknown>, inside);
    }
  }

  /**
   * Writes what an iterator yields as a JSON array, taking no item once the line is cut, so a huge map or set costs
   * no more than the text it can show.
   *
   * @param items - A map's entries, each written as a key and value pair, or a set's values.
   * @param place - Where each item sits.
   */
  private writeItems(items: Iterator<unknown>, place: ILogValuePlace): void {
    this.line.emit("[");

    for (let index: number = 0; !this.line.isCut; index += 1) {
      const item: IteratorResult<unknown> = items.next();

      if (item.done) {
        break;
      } else if (index > 0) {
        this.line.emit(",");
      }

      this.writeValue(item.value, place);
    }

    this.line.emit("]");
  }

  /**
   * Writes an array as JSON.
   *
   * @param value - The array.
   * @param place - Where each element sits.
   */
  private writeArray(value: ReadonlyArray<unknown>, place: ILogValuePlace): void {
    const count: number = value.length;

    this.line.emit("[");

    for (let index: number = 0; index < count && !this.line.isCut; index += 1) {
      if (index > 0) {
        this.line.emit(",");
      }

      this.writeProperty(value as unknown as Record<string, unknown>, String(index), place);
    }

    this.line.emit("]");
  }

  /**
   * Writes a plain object's own enumerable properties as JSON.
   *
   * @param value - The object.
   * @param place - Where each property's value sits.
   */
  private writeRecord(value: Record<string, unknown>, place: ILogValuePlace): void {
    const keys: ReadonlyArray<string> = Object.keys(value);

    this.line.emit("{");

    for (let index: number = 0; index < keys.length && !this.line.isCut; index += 1) {
      if (index > 0) {
        this.line.emit(",");
      }

      this.line.emit(`${JSON.stringify(keys[index])}:`);
      this.writeProperty(value, keys[index], place);
    }

    this.line.emit("}");
  }

  /**
   * Writes a property's value as JSON, or the unreadable marker where reading it runs a getter that throws.
   *
   * @param owner - The object or array holding the property.
   * @param key - The property's name, or an array's index.
   * @param place - Where the property's value sits.
   */
  private writeProperty(owner: Record<string, unknown>, key: string, place: ILogValuePlace): void {
    let item: unknown;

    try {
      item = owner[key];
    } catch {
      return this.line.emit(LogTextWriter.UNREADABLE);
    }

    this.writeValue(item, place);
  }
}
