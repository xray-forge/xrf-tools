import { useInjection } from "@wirestate/react";
import { ArchivePackConfig } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { SwitchFormRow } from "@xrf/ui/form/switch-row";
import { ArchiveIcon } from "@xrf/ui/icons/archive";
import { FolderIcon } from "@xrf/ui/icons/folder";
import { SettingsIcon } from "@xrf/ui/icons/settings";
import { TitleIcon } from "@xrf/ui/icons/title";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { ArchiveConfigEditorLayout } from "@/core/archive/components/config-editor/ArchiveConfigEditorLayout";
import { ArchiveConfigHeaderSection } from "@/core/archive/components/config-editor/ArchiveConfigHeaderSection";
import { ArchiveConfigOptionsSection } from "@/core/archive/components/config-editor/ArchiveConfigOptionsSection";
import { ArchiveConfigOutputSection } from "@/core/archive/components/config-editor/ArchiveConfigOutputSection";
import { IArchiveConfigSection } from "@/core/archive/components/config-editor/ArchiveConfigSectionsMenu";
import { ArchiveConfigToolbarActions } from "@/core/archive/components/config-editor/ArchiveConfigToolbarActions";
import { useArchiveConfigEditorShell } from "@/core/archive/components/config-editor/use-archive-config-editor-shell";
import {
  IArchiveRunConfirmation,
  useArchiveRunConfirmation,
} from "@/core/archive/components/config-editor/use-archive-run-confirmation";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { ArchivesPackerService, EPackerSection } from "@/tools/archives-packer/services/archives-packer.service";

import { ArchivesPackResult, PackerConfirmSummary } from "./components/packing";
import { PackerSelectionSection } from "./components/sections";

const SECTIONS: ReadonlyArray<IArchiveConfigSection<EPackerSection>> = [
  {
    id: EPackerSection.OUTPUT,
    label: "Source and output",
    description: "Where it comes from and lands",
    icon: <FolderIcon />,
  },
  { id: EPackerSection.SELECTION, label: "Selection", description: "What goes in", icon: <TuneIcon /> },
  { id: EPackerSection.HEADER, label: "Header", description: "Where it mounts", icon: <TitleIcon /> },
  { id: EPackerSection.OPTIONS, label: "Options", description: "How it is written", icon: <SettingsIcon /> },
];

export function ArchivesPackerTool(): ReactElement {
  const packerService: ArchivesPackerService = useInjection(ArchivesPackerService);

  const confirmation: IArchiveRunConfirmation<"pack"> = useArchiveRunConfirmation();

  const config: Nullable<ArchivePackConfig> = packerService.config;

  const isBusy: boolean = packerService.isBusy;
  const isPacking: boolean = Boolean(packerService.operation.job);

  const source: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PACKER,
    id: "source",
    title: "Select directory to pack",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isBusy,
    seed: () => resolveOutputPath(EToolId.ARCHIVES_PACKER),
  });

  /** The configuration as it would be packed, with the fields the editor owns folded back in. */
  const resolved: Nullable<ArchivePackConfig> = useMemo(() => {
    if (!config || !source.value || !destination.value) {
      return null;
    }

    return {
      ...config,
      source: source.value,
      destination: destination.value,
      maxVolumeSize: packerService.volumeSizeBytes,
    };
  }, [config, source.value, destination.value, packerService.volumeSizeBytes]);

  /** Opens the confirmation and asks what the destination already holds, so the summary can say so. */
  const onConfirmPack = useCallback(() => {
    if (!resolved) {
      return;
    }

    confirmation.open("pack");

    void packerService.checkDestination(resolved);
  }, [confirmation, packerService, resolved]);

  const onPack = useCallback(async () => {
    if (!resolved) {
      return;
    }

    confirmation.close();

    source.commit();
    destination.commit();

    await packerService.pack(resolved, confirmation.isForced);
  }, [confirmation, destination, packerService, resolved, source]);

  useEffect(() => {
    packerService.resetResult();
  }, [source.value, destination.value, packerService]);

  useArchiveConfigEditorShell({
    editor: packerService,
    panelId: "packer-sections",
    sections: SECTIONS,
    isBusy: isBusy || isPacking,
    resultStatus: packerService.operation.result ? [`${packerService.operation.result.volumes.length} volume(s)`] : [],
  });

  if (!config) {
    return <ToolLoader />;
  }

  const isPackDisabled: boolean = isBusy || isPacking || !resolved || Boolean(packerService.volumeSizeError);

  return (
    <ArchiveConfigEditorLayout
      editor={packerService}
      actions={
        <ArchiveConfigToolbarActions
          editor={packerService}
          noun={"packing configuration"}
          exportDescription={"Export these rules as a packing configuration"}
          isBusy={isBusy || isPacking}
        >
          <EditorTextAction
            label={"Pack"}
            description={isPackDisabled ? "Choose a source and an output first" : "Write the volumes"}
            icon={<ArchiveIcon />}
            isDisabled={isPackDisabled}
            variant={"contained"}
            onClick={onConfirmPack}
          />
        </ArchiveConfigToolbarActions>
      }
      result={packerService.operation.result ? <ArchivesPackResult result={packerService.operation.result} /> : null}
      confirmation={
        resolved ? (
          <ConfirmDialog
            isOpen={confirmation.run !== null}
            isDestructive={true}
            isConfirmDisabled={packerService.publishedVolumes.length > 0 && !confirmation.isForced}
            maxWidth={"sm"}
            title={"Pack archives?"}
            description={
              <PackerConfirmSummary
                config={resolved}
                publishedVolumes={packerService.publishedVolumes}
                isForced={confirmation.isForced}
                onForceChange={confirmation.setIsForced}
              />
            }
            confirmLabel={"Pack"}
            onConfirm={() => void onPack()}
            onClose={confirmation.close}
          />
        ) : null
      }
    >
      {packerService.section === EPackerSection.OUTPUT ? (
        <ArchiveConfigOutputSection
          editor={packerService}
          config={config}
          fieldId={"packer"}
          destination={destination}
          destinationDescription={"Directory the volumes are written into, overwriting volumes of the same name"}
          nameLabel={"Name"}
          nameDescription={"Base name of the volumes: one volume is <name>.db, several are <name>.db0 and up"}
          namePlaceholder={"gamedata"}
          isDisabled={isBusy}
        >
          <PathFormRow
            isDisabled={isBusy}
            label={"Source"}
            description={"Directory to pack. Every rule below is relative to it"}
            field={source}
          />
        </ArchiveConfigOutputSection>
      ) : null}

      {packerService.section === EPackerSection.SELECTION ? (
        <PackerSelectionSection config={config} isDisabled={isBusy} onChange={packerService.patchConfig} />
      ) : null}

      {packerService.section === EPackerSection.HEADER ? (
        <ArchiveConfigHeaderSection
          editor={packerService}
          config={config}
          fieldId={"packer"}
          entryPointDescription={"Where the engine mounts the contents. A packed gamedata tree wants the default"}
          missingEntryPointWarning={
            "Without an entry point the engine treats these volumes as encrypted Shadow of Chernobyl archives. Set " +
            "one, or switch the extension to xdb under Options."
          }
          isDisabled={isBusy}
        />
      ) : null}

      {packerService.section === EPackerSection.OPTIONS ? (
        <ArchiveConfigOptionsSection editor={packerService} config={config} fieldId={"packer"} isDisabled={isBusy}>
          <SwitchFormRow
            id={"packer-skip-list"}
            label={"Skip editor leftovers"}
            description={"Drops the sources and intermediates a game build never reads, as xrCompress does"}
            isChecked={config.isWithSkipList}
            isDisabled={isBusy}
            onChange={(isWithSkipList: boolean) => packerService.patchConfig({ isWithSkipList })}
          />
        </ArchiveConfigOptionsSection>
      ) : null}
    </ArchiveConfigEditorLayout>
  );
}
