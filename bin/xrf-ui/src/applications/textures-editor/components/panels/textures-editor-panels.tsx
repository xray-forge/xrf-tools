import { default as EditNoteIcon } from "@mui/icons-material/EditNote";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as TuneIcon } from "@mui/icons-material/Tune";

import { TextureDescriptorPanel } from "@/applications/textures-editor/components/panels/TextureDescriptorPanel";
import { IEditorPanel } from "@/core/shell/panel/context";
import { TextureChannelsPanel } from "@/core/textures/components/panels/TextureChannelsPanel";
import { TextureFilesPanel } from "@/core/textures/components/panels/TextureFilesPanel";
import { TextureMaterialPanel } from "@/core/textures/components/panels/TextureMaterialPanel";

/**
 * The panels the workbench offers, all of them about the one texture that is open.
 *
 * @returns The panels to register.
 */
export function createTexturesEditorPanels(): Array<IEditorPanel> {
  return [
    {
      icon: <EditNoteIcon />,
      id: "descriptor",
      isOpenByDefault: true,
      label: "Descriptor",
      render: () => <TextureDescriptorPanel />,
    },
    {
      icon: <InfoIcon />,
      id: "material",
      isOpenByDefault: false,
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
    {
      icon: <TuneIcon />,
      id: "channels",
      isOpenByDefault: false,
      label: "Channels",
      render: () => <TextureChannelsPanel />,
    },
  ];
}
