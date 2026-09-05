import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as LayersIcon } from "@mui/icons-material/Layers";

import { TexturesMenu } from "@/applications/textures-explorer/components/editor/tree/TexturesMenu";
import { IEditorPanel } from "@/core/shell/panel/context";

import { TextureFilesPanel } from "./TextureFilesPanel";
import { TextureMaterialPanel } from "./TextureMaterialPanel";

/**
 * The panels the explorer offers, browsing on the left and inspection on the right.
 *
 * @param isBrowsing - Whether a root set is open.
 * @returns The panels to register.
 */
export function createTexturesExplorerPanels(isBrowsing: boolean): Array<IEditorPanel> {
  return [
    ...(isBrowsing
      ? [
          {
            icon: <ImageIcon />,
            id: "textures",
            isOpenByDefault: true,
            label: "Textures",
            render: () => <TexturesMenu />,
            side: "left" as const,
          },
        ]
      : []),
    {
      icon: <InfoIcon />,
      id: "material",
      isOpenByDefault: true,
      label: "Material",
      render: () => <TextureMaterialPanel />,
    },
    {
      icon: <LayersIcon />,
      id: "files",
      isOpenByDefault: false,
      label: "Files",
      render: () => <TextureFilesPanel />,
    },
  ];
}
