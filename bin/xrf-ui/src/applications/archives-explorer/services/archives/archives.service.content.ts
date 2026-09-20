import { getSubjectRoots, IArchiveEntry, TArchiveContent } from "@/core/archive/lib";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { archivesRawCommands } from "@/core/ipc/commands/archives-raw";
import { assetsRawCommands } from "@/core/ipc/commands/assets-raw";
import { ArchiveSubject, SessionId } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/**
 * Reads one archived file as whatever a preview can draw it with.
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
   * @param subject - Open subject the file is read out of.
   * @returns What a preview draws.
   */
  public read(entry: IArchiveEntry, kind: TArchiveContent["kind"], subject: ArchiveSubject): Promise<TArchiveContent> {
    // Exhaustive by the union rather than by a default, so a new previewable kind fails to compile here first.
    switch (kind) {
      case "audio":
        return this.readAudio(entry, subject);

      case "texture":
        return this.readTexture(entry, subject);

      case "image":
        return this.readImage(entry, subject);

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
  private async readAudio(entry: IArchiveEntry, subject: ArchiveSubject): Promise<TArchiveContent> {
    const roots: XrayRoots = getSubjectRoots(subject);

    const [audio, bytes] = await Promise.all([
      archivesCommands.describeAudio(roots, entry.name),
      assetsRawCommands.readAsset(roots, entry.name),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: audio, kind: "audio" };
  }

  /** Reads a texture as its source shape plus the png the backend decoded it into. */
  private async readTexture(entry: IArchiveEntry, subject: ArchiveSubject): Promise<TArchiveContent> {
    const roots: XrayRoots = getSubjectRoots(subject);

    const [texture, bytes] = await Promise.all([
      archivesCommands.describeTexture(roots, entry.name),
      archivesRawCommands.readTexture(roots, entry.name),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: texture, kind: "texture" };
  }

  /** Reads a picture the webview draws itself: its shape plus the bytes exactly as stored. */
  private async readImage(entry: IArchiveEntry, subject: ArchiveSubject): Promise<TArchiveContent> {
    const roots: XrayRoots = getSubjectRoots(subject);

    const [image, bytes] = await Promise.all([
      archivesCommands.describeImage(roots, entry.name),
      assetsRawCommands.readAsset(roots, entry.name),
    ]);

    return { bytes: new Uint8Array(bytes), descriptor: image, kind: "image" };
  }
}
