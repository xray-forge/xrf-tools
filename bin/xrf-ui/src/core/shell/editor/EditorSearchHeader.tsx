import { KeyboardEvent, ReactElement } from "react";

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
  return (
    <EditorPanelHeader data-testid={dataTestId} id={id} className={className} title={title} caption={count}>
      <EditorFilterInput
        query={query}
        placeholder={placeholder}
        ariaLabel={ariaLabel}
        onClear={onClear}
        onKeyDown={onKeyDown}
        onQueryChange={onQueryChange}
      />
    </EditorPanelHeader>
  );
}
