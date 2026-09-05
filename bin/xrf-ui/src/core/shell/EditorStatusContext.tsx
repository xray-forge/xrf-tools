import { createContext, ReactElement, ReactNode, useContext, useEffect, useMemo, useRef, useState } from "react";

type StatusSetter = (segments: Array<string>) => void;

interface IEditorStatusContextValue {
  segments: Array<string>;
  setSegments: StatusSetter;
}

const EditorStatusContext = createContext<IEditorStatusContextValue>({
  segments: [],
  setSegments: () => {},
});

interface IEditorStatusProviderProps {
  children: ReactNode;
}

export function EditorStatusProvider({ children }: IEditorStatusProviderProps): ReactElement {
  const [segments, setSegments] = useState<Array<string>>([]);

  const value: IEditorStatusContextValue = useMemo(() => ({ segments, setSegments }), [segments]);

  return <EditorStatusContext.Provider value={value}>{children}</EditorStatusContext.Provider>;
}

export function useEditorStatusSegments(): Array<string> {
  return useContext(EditorStatusContext).segments;
}

/**
 * Publishes status segments while the calling editor is mounted.
 *
 * Segments are compared by serialized content rather than by array identity, so callers can pass a
 * literal without memoising it and without re-publishing on every render. The array itself is passed
 * through a ref so the published value never has to be reconstructed from the comparison key.
 *
 * @param segments - Status text segments to publish.
 */
export function useEditorStatus(segments: Array<string>): void {
  const { setSegments } = useContext(EditorStatusContext);

  const latest = useRef<Array<string>>(segments);
  const key: string = JSON.stringify(segments);

  latest.current = segments;

  useEffect(() => {
    setSegments(latest.current);

    return () => setSegments([]);
  }, [key, setSegments]);
}
