import { fetchBulk } from "@xrf/ipc";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { archivesBulkRoutes } from "@xrf/ipc/commands/archives-bulk";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";

import { IArchiveEntry, TArchiveContent } from "@/core/archive/lib";

/**
 * Reads one file of the open subject as whatever a preview can draw it with, every read addressed by the subject's
 * session, so the bytes always come from what the subject lists.
 */
export class ArchiveContentReader {
  /**
   * Reads one file as the representation asked for.
   *
   * @param sessionId - The open subject's session, which every read is addressed by.
   * @param entry - File to read.
   * @param kind - Representation to request.
   * @returns What a preview draws.
   */
  public read(sessionId: SessionId, entry: IArchiveEntry, kind: TArchiveContent["kind"]): Promise<TArchiveContent> {
    // Exhaustive by the union rather than by a default, so a new previewable kind fails to compile here first.
    switch (kind) {
      case "audio":
        return this.readAudio(sessionId, entry);

      case "texture":
        return this.readTexture(sessionId, entry);

      case "image":
        return this.readImage(sessionId, entry);

      case "description":
        return this.readDescription(sessionId, entry);

      case "text":
        return this.readText(sessionId, entry);
    }
  }

  /** Reads a file as the Windows-1251 text an engine surface shows. */
  private async readText(sessionId: SessionId, entry: IArchiveEntry): Promise<TArchiveContent> {
    return { kind: "text", result: (await archivesCommands.readFile(sessionId, entry.name)).value };
  }

  /**
   * Reads what the backend can say about a binary format in words, which may be it saying it has no describer for
   * this format yet.
   */
  private async readDescription(sessionId: SessionId, entry: IArchiveEntry): Promise<TArchiveContent> {
    return {
      description: (await archivesCommands.describeFile(sessionId, entry.name)).value,
      kind: "description",
    };
  }

  /** Reads a sound as the description the engine would read plus the bytes the webview plays. */
  private async readAudio(sessionId: SessionId, entry: IArchiveEntry): Promise<TArchiveContent> {
    const [audio, bytes] = await Promise.all([
      archivesCommands.describeAudio(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readEntry(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: audio.value, kind: "audio" };
  }

  /** Reads a texture as its source shape plus the png the backend decoded it into. */
  private async readTexture(sessionId: SessionId, entry: IArchiveEntry): Promise<TArchiveContent> {
    const [texture, bytes] = await Promise.all([
      archivesCommands.describeTexture(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readTexture(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: texture.value, kind: "texture" };
  }

  /** Reads a picture the webview draws itself: its shape plus the bytes exactly as stored. */
  private async readImage(sessionId: SessionId, entry: IArchiveEntry): Promise<TArchiveContent> {
    const [image, bytes] = await Promise.all([
      archivesCommands.describeImage(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readEntry(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: image.value, kind: "image" };
  }
}
