import { IArchiveEntry, TArchiveContent } from "@/core/archive/lib";
import { fetchBulk } from "@/core/ipc/bulk";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { archivesBulkRoutes } from "@/core/ipc/commands/archives-bulk";
import { SessionId } from "@/core/ipc/types/xrf-app";

/**
 * Reads one file of the open subject as whatever a preview can draw it with, every read addressed by the subject's
 * session, so the bytes always come from what the subject lists.
 */
export class ArchiveContentReader {
  /** The session every read of the open subject is addressed by, which only the service knows. */
  private readonly session: () => SessionId;

  public constructor(session: () => SessionId) {
    this.session = session;
  }

  /**
   * Reads one file as the representation asked for.
   *
   * @param entry - File to read.
   * @param kind - Representation to request.
   * @returns What a preview draws.
   */
  public read(entry: IArchiveEntry, kind: TArchiveContent["kind"]): Promise<TArchiveContent> {
    // Exhaustive by the union rather than by a default, so a new previewable kind fails to compile here first.
    switch (kind) {
      case "audio":
        return this.readAudio(entry);

      case "texture":
        return this.readTexture(entry);

      case "image":
        return this.readImage(entry);

      case "description":
        return this.readDescription(entry);

      case "text":
        return this.readText(entry);
    }
  }

  /** Reads a file as the Windows-1251 text an engine surface shows. */
  private async readText(entry: IArchiveEntry): Promise<TArchiveContent> {
    return { kind: "text", result: await archivesCommands.readFile(this.session(), entry.name) };
  }

  /**
   * Reads what the backend can say about a binary format in words, which may be it saying it has no describer for
   * this format yet.
   */
  private async readDescription(entry: IArchiveEntry): Promise<TArchiveContent> {
    return { description: await archivesCommands.describeFile(this.session(), entry.name), kind: "description" };
  }

  /** Reads a sound as the description the engine would read plus the bytes the webview plays. */
  private async readAudio(entry: IArchiveEntry): Promise<TArchiveContent> {
    const sessionId: SessionId = this.session();

    const [audio, bytes] = await Promise.all([
      archivesCommands.describeAudio(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readEntry(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: audio, kind: "audio" };
  }

  /** Reads a texture as its source shape plus the png the backend decoded it into. */
  private async readTexture(entry: IArchiveEntry): Promise<TArchiveContent> {
    const sessionId: SessionId = this.session();

    const [texture, bytes] = await Promise.all([
      archivesCommands.describeTexture(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readTexture(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: texture, kind: "texture" };
  }

  /** Reads a picture the webview draws itself: its shape plus the bytes exactly as stored. */
  private async readImage(entry: IArchiveEntry): Promise<TArchiveContent> {
    const sessionId: SessionId = this.session();

    const [image, bytes] = await Promise.all([
      archivesCommands.describeImage(sessionId, entry.name),
      fetchBulk(archivesBulkRoutes.readEntry(sessionId, entry.name)),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: image, kind: "image" };
  }
}
