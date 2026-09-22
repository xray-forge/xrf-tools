import { useInjection } from "@wirestate/react";
import { Nullable, Optional } from "@xrf/types";
import { KeyboardEvent, ReactElement, useRef } from "react";

import { formatChord, parseChord } from "@/core/keybinds";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { useSearchFocusTarget } from "@/core/search/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { EditorFilterInput } from "./EditorFilterInput";
import { EditorPanelHeader } from "./EditorPanelHeader";

interface IEditorSearchHeaderProps extends BaseComponentProps {
  /** What the panel lists, as its heading. */
  title: string;
  count: number;
  query: string;
  placeholder: string;
  /** Names the field for a screen reader, which the placeholder alone does not. */
  ariaLabel: string;
  onClear: () => void;
  /** Lets the search field drive the result list without losing focus. */
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
}

/**
 * Heading and filter field for a side menu that lists more than fits on a screen.
 */
export function EditorSearchHeader({
  "data-testid": dataTestId,
  id,
  className,
  title,
  count,
  query,
  placeholder,
  ariaLabel,
  onClear,
  onKeyDown,
  onQueryChange,
}: IEditorSearchHeaderProps): ReactElement {
  const keymapService: KeymapService = useInjection(KeymapService);

  const inputRef = useRef<Nullable<HTMLInputElement>>(null);
  // Only where this screen declares the command: a hint in a tool that does not answer the chord would be a lie.
  const chord: Optional<string> = keymapService.getReachableChords(FOCUS_SEARCH_KEYBIND_COMMAND).at(0);

  useSearchFocusTarget(inputRef);

  return (
    <EditorPanelHeader data-testid={dataTestId} id={id} className={className} title={title} caption={count}>
      <EditorFilterInput
        ariaLabel={ariaLabel}
        inputRef={inputRef}
        chord={chord ? formatChord(parseChord(chord)) : undefined}
        query={query}
        placeholder={placeholder}
        onClear={onClear}
        onKeyDown={onKeyDown}
        onQueryChange={onQueryChange}
      />
    </EditorPanelHeader>
  );
}
