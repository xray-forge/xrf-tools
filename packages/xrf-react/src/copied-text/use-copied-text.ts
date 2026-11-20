import { useCallback, useEffect, useState } from "react";

/** Milliseconds a copy is said to be done, counted from the latest copy. */
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
  // Counts successful copies within the current window, so each one restarts it; zero is no window.
  const [copies, setCopies] = useState<number>(0);

  const copy = useCallback(
    (text: string) => {
      navigator.clipboard
        ?.writeText(text)
        .then(() => setCopies((count) => count + 1))
        .catch(onFailure);
    },
    [onFailure]
  );

  useEffect(() => {
    if (copies === 0) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setCopies(0), COPIED_FOR);

    return () => clearTimeout(timeout);
  }, [copies]);

  return { copy, isCopied: copies > 0 };
}
