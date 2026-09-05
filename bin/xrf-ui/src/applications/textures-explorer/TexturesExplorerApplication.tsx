import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TexturesEditor } from "@/applications/textures-explorer/components/editor/TexturesEditor";
import { TexturesExplorerOpenForm } from "@/applications/textures-explorer/components/TexturesExplorerOpenForm";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ITexturesExplorerApplicationProps extends BaseComponentProps {}

/**
 * Browse a root set of textures, or inspect one texture on its own.
 */
export function TexturesExplorerApplication({
  "data-testid": dataTestId = "textures-explorer-application",
}: ITexturesExplorerApplicationProps = {}): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const isOpen: boolean = texturesService.isBrowsing || texturesService.selected.value !== null;

  const [isPickerOpen, setPickerOpen] = useState<boolean>(false);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  if (!texturesService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || !isOpen) {
    return <TexturesExplorerOpenForm data-testid={dataTestId} onFinished={onFinished} />;
  }

  return <TexturesEditor data-testid={dataTestId} />;
}
