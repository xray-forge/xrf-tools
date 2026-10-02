/**
 * @returns A part of a batch's answer, framed as the server frames it.
 */
export function toFetchPart(index: number, status: number, type: string, bytes: ReadonlyArray<number>): Uint8Array {
  const media: Uint8Array = new TextEncoder().encode(type);
  const part: Uint8Array = new Uint8Array(12 + media.length + bytes.length);
  const view: DataView = new DataView(part.buffer);

  view.setUint32(0, index, true);
  view.setUint16(4, status, true);
  view.setUint16(6, media.length, true);
  view.setUint32(8, bytes.length, true);
  part.set(media, 12);
  part.set(bytes, 12 + media.length);

  return part;
}

/**
 * @returns A stream of the bytes as a fetch's body is one, cut into chunks of the size given.
 */
export function toStream(bytes: Uint8Array, size: number): ReadableStream<Uint8Array> {
  let at: number = 0;

  return new ReadableStream({
    pull(controller: ReadableByteStreamController): void {
      if (at >= bytes.length) {
        controller.close();
        controller.byobRequest?.respond(0);

        return;
      }

      controller.enqueue(bytes.slice(at, at + size));
      at += size;
    },
    type: "bytes",
  });
}
