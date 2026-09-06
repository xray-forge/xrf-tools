import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TexturesExplorerOpenForm } from "@/applications/textures-explorer/components/TexturesExplorerOpenForm";
import { TexturesExplorerWorkspace } from "@/applications/textures-explorer/components/TexturesExplorerWorkspace";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ITexturesExplorerApplicationProps extends BaseComponentProps {}

/**
 * Browse a root set of textures, or inspect one texture on its own.
 */
export function TexturesExplorerApplication({
  "data-testid": dataTestId = "textures-explorer-application",
}: ITexturesExplorerApplicationProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const isOpen: boolean = catalogService.isBrowsing || selectionService.selected.value !== null;

  const [isPickerOpen, setPickerOpen] = useState<boolean>(false);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  if (!catalogService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || !isOpen) {
    return <TexturesExplorerOpenForm data-testid={dataTestId} onFinished={onFinished} />;
  }

  return <TexturesExplorerWorkspace data-testid={dataTestId} />;
}
