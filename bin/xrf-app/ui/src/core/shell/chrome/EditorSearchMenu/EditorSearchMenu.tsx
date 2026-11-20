import {
  EditorSearchMenu as EditorSearchMenuBody,
  IEditorSearchMenuHeading,
  IEditorSearchMenuProps,
} from "@xrf/ui/editor/search-menu";
import { ReactElement } from "react";

import { EditorSearchHeader } from "@/core/shell/chrome/EditorSearchHeader";

function renderHeading(heading: IEditorSearchMenuHeading): ReactElement {
  return <EditorSearchHeader {...heading} />;
}

/**
 * A searchable side menu headed as a docked panel: the search chord, the field the focus command reaches, and the
 * panel's close action.
 */
export function EditorSearchMenu<T>(props: Omit<IEditorSearchMenuProps<T>, "renderHeading">): ReactElement {
  return <EditorSearchMenuBody<T> {...props} renderHeading={renderHeading} />;
}
