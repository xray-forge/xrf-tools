import { default as CompareIcon } from "@mui/icons-material/Compare";
import { default as EditNoteIcon } from "@mui/icons-material/EditNote";
import { default as GradientIcon } from "@mui/icons-material/Gradient";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as TuneIcon } from "@mui/icons-material/Tune";

import { TextureBumpPanel } from "@/applications/textures-editor/components/panels/TextureBumpPanel";
import { TextureDescriptorPanel } from "@/applications/textures-editor/components/panels/TextureDescriptorPanel";
import { TextureFormatsPanel } from "@/applications/textures-editor/components/panels/TextureFormatsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";
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
      icon: <CompareIcon />,
      id: "formats",
      isOpenByDefault: false,
      label: "Formats",
      render: () => <TextureFormatsPanel />,
    },
    {
      icon: <GradientIcon />,
      id: "bump",
      isOpenByDefault: false,
      label: "Bump",
      render: () => <TextureBumpPanel />,
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
