import { useCallback, useEffect, useState } from "react";

/** Milliseconds a copy is said to be done. */
export const COPIED_FOR: number = 1500;

/** Text copied to the clipboard, and whether it was a moment ago. */
export interface ICopiedText {
  /** Whether the last copy succeeded within the last moment, for a control to say so. */
  isCopied: boolean;
  /** Copies the text, failing to the given handler where the clipboard refuses it. */
  copy: (text: string) => void;
}

/**
 * @param onFailure - Told why the clipboard refused a copy.
 * @returns What copies text, and whether it just did.
 */
export function useCopiedText(onFailure: (error: unknown) => void): ICopiedText {
  const [isCopied, setIsCopied] = useState<boolean>(false);

  const copy = useCallback(
    (text: string) => {
      navigator.clipboard
        ?.writeText(text)
        .then(() => setIsCopied(true))
        .catch(onFailure);
    },
    [onFailure]
  );

  useEffect(() => {
    if (!isCopied) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setIsCopied(false), COPIED_FOR);

    return () => clearTimeout(timeout);
  }, [isCopied]);

  return { copy, isCopied };
}
