/**
 * One logged line's text as it is written, never longer than its maximum length: what does not fit is cut, and the
 * finished text then ends with a marker saying so.
 */
export class LogLine {
  private readonly maxLength: number;

  /** The text written so far, in order; together never longer than the maximum length. */
  private readonly chunks: Array<string> = [];

  /** How many characters the chunks hold. */
  private written: number = 0;

  /** Whether text was lost to the maximum length; once it was, nothing more is kept. */
  private cut: boolean = false;

  /**
   * @param maxLength - How many characters the line keeps.
   */
  public constructor(maxLength: number) {
    this.maxLength = maxLength;
  }

  /** Whether text was lost to the maximum length, after which writing more only costs time. */
  public get isCut(): boolean {
    return this.cut;
  }

  /** How many more characters the line keeps. */
  public get room(): number {
    return Math.max(this.maxLength - this.written, 0);
  }

  /**
   * Adds text to the line, keeping what fits and noting a cut where it does not all fit.
   *
   * @param text - The next piece of the line.
   */
  public emit(text: string): void {
    const room: number = this.room;

    if (text.length > room) {
      this.chunks.push(text.slice(0, room));
      this.written = this.maxLength;
      this.cut = true;
    } else {
      this.chunks.push(text);
      this.written += text.length;
    }
  }

  /**
   * Runs a write that may throw midway, putting a fallback in place of whatever it had written when it does.
   *
   * @param write - Writes one value into this line.
   * @param fallback - What the value is written as when its write throws.
   */
  public emitOrFallBack(write: () => void, fallback: string): void {
    const chunkCount: number = this.chunks.length;
    const written: number = this.written;
    const isCut: boolean = this.cut;

    try {
      write();
    } catch {
      this.chunks.length = chunkCount;
      this.written = written;
      this.cut = isCut;
      this.emit(fallback);
    }
  }

  /**
   * @returns The line's text, ending with a marker when anything was cut.
   */
  public toText(): string {
    const text: string = this.chunks.join("");

    return this.cut ? `${text}… [cut at ${this.maxLength} characters]` : text;
  }
}
