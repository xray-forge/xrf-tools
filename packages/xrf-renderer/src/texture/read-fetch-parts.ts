import { Nullable } from "@xrf/types";

import { IFetchPart } from "#/texture/fetch-part";

/** Bytes of a part's header before its media type: index, status, media type's length, bytes' length. */
const PART_HEADER_BYTES: number = 12;

/** Bytes a part may say it holds, far past any texture: a header saying more is a broken answer, not one to allocate. */
const MAXIMUM_PART_BYTES: number = 1 << 30;

/**
 * Reads a batch's answer as it streams in, handing each part over as soon as its last byte comes, so a texture read
 * early is taken while the batch's slower ones are still read. Each part's bytes are read straight into a buffer of
 * their own, which the browser fills from the network as it would an answer's whole body: copied once, never by script.
 *
 * @param stream - The answer's body, a stream of bytes.
 * @param onPart - Takes each part.
 */
export async function readFetchParts(
  stream: ReadableStream<Uint8Array>,
  onPart: (part: IFetchPart) => void
): Promise<void> {
  const reader: ReadableStreamBYOBReader = stream.getReader({ mode: "byob" });
  const decoder: TextDecoder = new TextDecoder();

  try {
    for (;;) {
      const header: Nullable<ArrayBuffer> = await readFully(reader, new ArrayBuffer(PART_HEADER_BYTES), true);

      if (!header) {
        return;
      }

      const view: DataView = new DataView(header);
      const length: number = view.getUint32(8, true);

      if (length > MAXIMUM_PART_BYTES) {
        throw new Error(`The batch's answer says a part holds ${length} bytes, past what one may`);
      }

      const media: ArrayBuffer = await readFully(reader, new ArrayBuffer(view.getUint16(6, true)));
      const bytes: ArrayBuffer = await readFully(reader, new ArrayBuffer(length));

      onPart({ bytes, index: view.getUint32(0, true), status: view.getUint16(4, true), type: decoder.decode(media) });
    }
  } catch (error: unknown) {
    // Nothing more of a broken answer is read: what the server still sends goes nowhere.
    reader.cancel(error).catch(() => {});

    throw error;
  }
}

/**
 * @param reader - The answer's reader.
 * @param buffer - What to fill, which each read hands back as a buffer of its own.
 * @param isEndAllowed - Whether the answer may end before the buffer's first byte, as it does after its last part.
 * @returns The buffer filled, or null where the answer ended where that is allowed.
 */
async function readFully(
  reader: ReadableStreamBYOBReader,
  buffer: ArrayBuffer,
  isEndAllowed: true
): Promise<Nullable<ArrayBuffer>>;
async function readFully(reader: ReadableStreamBYOBReader, buffer: ArrayBuffer): Promise<ArrayBuffer>;
async function readFully(
  reader: ReadableStreamBYOBReader,
  buffer: ArrayBuffer,
  isEndAllowed: boolean = false
): Promise<Nullable<ArrayBuffer>> {
  let filled: ArrayBuffer = buffer;
  let at: number = 0;

  while (at < filled.byteLength) {
    const { done, value }: ReadableStreamReadResult<Uint8Array<ArrayBuffer>> = await reader.read(
      new Uint8Array(filled, at, filled.byteLength - at)
    );

    if (done || !value) {
      if (at === 0 && isEndAllowed) {
        return null;
      }

      throw new Error("The batch's answer ended inside a part");
    }

    filled = value.buffer;
    at += value.byteLength;
  }

  return filled;
}
