import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { TextureCatalogService } from "@/applications/textures-explorer/services/texture-catalog.service";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";

import { TexturesExplorerOpenForm } from "./components/TexturesExplorerOpenForm";
import { TexturesExplorerWorkspace } from "./components/TexturesExplorerWorkspace";

/**
 * Browse a root set of textures, or inspect one texture on its own.
 */
export function TexturesExplorerApplication({
  "data-testid": dataTestId = "textures-explorer-application",
}: BaseComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const isOpen: boolean = catalogService.isBrowsing || selectionService.selected.value !== null;

  if (!catalogService.catalog.isRestored) {
    return <ApplicationLoader />;
  }

  if (!isOpen) {
    return <TexturesExplorerOpenForm />;
  }

  return <TexturesExplorerWorkspace data-testid={dataTestId} />;
}
