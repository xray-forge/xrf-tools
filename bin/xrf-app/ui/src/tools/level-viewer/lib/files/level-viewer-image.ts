import { fetchBulk } from "@xrf/ipc";
import { levelsBulkRoutes } from "@xrf/ipc/commands/levels-bulk";
import { getPathName } from "@xrf/path";

/** A PNG of the screenshots folder as the viewer holds it: its file, its size, and its pixels for the page. */
export interface ILevelViewerImage {
  path: string;
  name: string;
  width: number;
  height: number;
  /** Bytes the PNG takes. */
  size: number;
  /** An object URL of the PNG, its holder's to let go once it is replaced or deleted. */
  url: string;
}

/** Where a PNG names its width and height: big-endian words after its signature and its header's length and type. */
const PNG_WIDTH_OFFSET: number = 16;

/**
 * @param bytes - A PNG file.
 * @returns Its width and height, from its header.
 */
export function readPngSize(bytes: Uint8Array): [number, number] {
  const view: DataView = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

  return [view.getUint32(PNG_WIDTH_OFFSET), view.getUint32(PNG_WIDTH_OFFSET + 4)];
}

/**
 * @param path - A PNG of the screenshots folder.
 * @returns It read back for the page.
 */
export async function readLevelViewerImage(path: string): Promise<ILevelViewerImage> {
  const bytes: Uint8Array = new Uint8Array(await fetchBulk(levelsBulkRoutes.readScreenshot(path)));
  const [width, height]: [number, number] = readPngSize(bytes);

  return {
    height,
    name: getPathName(path),
    path,
    size: bytes.byteLength,
    url: URL.createObjectURL(new Blob([bytes as BlobPart], { type: "image/png" })),
    width,
  };
}

/**
 * Puts an image's pixels on the clipboard.
 *
 * @param image - The image.
 */
export async function copyLevelViewerImage(image: ILevelViewerImage): Promise<void> {
  const blob: Blob = await fetch(image.url).then((response: Response) => response.blob());

  await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
}
