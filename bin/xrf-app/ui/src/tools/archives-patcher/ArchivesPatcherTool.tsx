import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ArchivesPatchRequest } from "@xrf/ipc/types/xrf-app-archives";
import { ArchivePatchConfig } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { SwitchFormRow } from "@xrf/ui/form/switch-row";
import { CompareArrowsIcon } from "@xrf/ui/icons/compare-arrows";
import { DifferenceIcon } from "@xrf/ui/icons/difference";
import { FolderIcon } from "@xrf/ui/icons/folder";
import { SettingsIcon } from "@xrf/ui/icons/settings";
import { TitleIcon } from "@xrf/ui/icons/title";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

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
import { DEFAULT_ENTRY_POINT } from "@/core/archive/lib/header";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { IPathField, usePathField } from "@/core/ui/form";
import { ArchivesPatcherService, EPatcherSection } from "@/tools/archives-patcher/services/archives-patcher.service";

import { ArchivesPatchResult } from "./components/ArchivesPatchResult";
import { PatcherConfirmSummary } from "./components/patching";
import { PatcherComparisonSection, PatcherSelectionSection } from "./components/sections";

/** The patcher's two runs: a comparison that writes nothing, and the patch it would publish. */
type TPatcherRun = "compare" | "patch";

const SECTIONS: ReadonlyArray<IArchiveConfigSection<EPatcherSection>> = [
  {
    id: EPatcherSection.COMPARISON,
    label: "Comparison",
    description: "What the patch is built from",
    icon: <CompareArrowsIcon />,
  },
  { id: EPatcherSection.OUTPUT, label: "Output", description: "Where the volumes land", icon: <FolderIcon /> },
  { id: EPatcherSection.SELECTION, label: "Selection", description: "What is compared", icon: <TuneIcon /> },
  { id: EPatcherSection.HEADER, label: "Header", description: "Where it mounts", icon: <TitleIcon /> },
  { id: EPatcherSection.OPTIONS, label: "Options", description: "How it is written", icon: <SettingsIcon /> },
];

/** Why the runs are unavailable, said by both. */
const RUN_DISABLED_REASON: string = "Choose a game and an output first";

export function ArchivesPatcherTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const patcherService: ArchivesPatcherService = useInjection(ArchivesPatcherService);

  const config: Nullable<ArchivePatchConfig> = patcherService.config;

  const isRunning: boolean = patcherService.operation.isRunning;
  const isBusy: boolean = isRunning || patcherService.isBusy;

  const [isVerifyingPayload, setIsVerifyingPayload] = useState<boolean>(false);
  const confirmation: IArchiveRunConfirmation<TPatcherRun> = useArchiveRunConfirmation();

  const input: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PATCHER,
    id: "input",
    title: "Select the game installation",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const target: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PATCHER,
    id: "target",
    title: "Select the tree the patch delivers",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PATCHER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isBusy,
    seed: () => resolveOutputPath(EToolId.ARCHIVES_PATCHER),
  });

  /** Whether the patch is built from a folder of its own, which is what a non-null target means. */
  const isDeliveringOwnTree: boolean = Boolean(config && config.target !== null);

  const request: Nullable<ArchivesPatchRequest> = useMemo(() => {
    if (!config || input.value === null || destination.value === null) {
      return null;
    }

    if (isDeliveringOwnTree && !target.value) {
      return null;
    }

    return {
      config: {
        ...config,
        input: input.value,
        target: isDeliveringOwnTree ? target.value : null,
        destination: destination.value,
        maxVolumeSize: patcherService.volumeSizeBytes,
      },
      isForced: confirmation.isForced,
      isVerifyingPayload,
    };
  }, [
    config,
    confirmation.isForced,
    destination.value,
    input.value,
    isDeliveringOwnTree,
    isVerifyingPayload,
    patcherService.volumeSizeBytes,
    target.value,
  ]);

  /** Opens the confirmation, asking what the output already holds so the summary can say so. */
  const onConfirm = useCallback(
    (run: TPatcherRun) => {
      if (!request) {
        return;
      }

      confirmation.open(run);

      if (run === "patch") {
        void patcherService.checkDestination(request.config);
      }
    },
    [confirmation, patcherService, request]
  );

  const onRun = useCallback(async () => {
    const run: Nullable<TPatcherRun> = confirmation.run;

    if (!request || !run) {
      return;
    }

    log.info("Running patcher:", run);

    confirmation.close();

    input.commit();
    destination.commit();

    if (isDeliveringOwnTree) {
      target.commit();
    }

    await (run === "compare" ? patcherService.compare(request) : patcherService.patch(request));
  }, [confirmation, destination, input, isDeliveringOwnTree, log, patcherService, request, target]);

  // Changing what is compared invalidates whatever the previous run reported.
  useEffect(() => {
    patcherService.resetResult();
  }, [destination.value, input.value, patcherService, target.value]);

  useArchiveConfigEditorShell({
    editor: patcherService,
    panelId: "patcher-sections",
    sections: SECTIONS,
    isBusy,
    resultStatus: patcherService.operation.result
      ? [
          `${formatCount(patcherService.operation.result.added.length + patcherService.operation.result.modified.length)} carried`,
        ]
      : [],
  });

  if (!config) {
    return <ToolLoader />;
  }

  const isRunDisabled: boolean = isBusy || !request || !config.name.trim() || Boolean(patcherService.volumeSizeError);

  return (
    <ArchiveConfigEditorLayout
      editor={patcherService}
      actions={
        <ArchiveConfigToolbarActions
          editor={patcherService}
          noun={"patching configuration"}
          exportDescription={"Export this scope and header as a patching configuration"}
          isBusy={isBusy}
        >
          <EditorTextAction
            label={"Compare"}
            description={isRunDisabled ? RUN_DISABLED_REASON : "Report what differs without writing"}
            icon={<CompareArrowsIcon />}
            isDisabled={isRunDisabled}
            onClick={() => onConfirm("compare")}
          />

          <EditorTextAction
            label={"Patch"}
            description={isRunDisabled ? RUN_DISABLED_REASON : "Write the patch volumes"}
            icon={<DifferenceIcon />}
            isDisabled={isRunDisabled}
            variant={"contained"}
            onClick={() => onConfirm("patch")}
          />
        </ArchiveConfigToolbarActions>
      }
      result={
        patcherService.operation.result ? (
          <ArchivesPatchResult result={patcherService.operation.result} outputPath={destination.value} />
        ) : null
      }
      confirmation={
        request ? (
          <ConfirmDialog
            isOpen={confirmation.run !== null}
            isDestructive={confirmation.run === "patch"}
            isConfirmDisabled={
              confirmation.run === "patch" && patcherService.publishedVolumes.length > 0 && !confirmation.isForced
            }
            maxWidth={"sm"}
            title={confirmation.run === "compare" ? "Compare archives?" : "Write patch?"}
            description={
              <PatcherConfirmSummary
                config={request.config}
                isPreviewOnly={confirmation.run === "compare"}
                publishedVolumes={patcherService.publishedVolumes}
                isForced={confirmation.isForced}
                onForceChange={confirmation.setIsForced}
              />
            }
            confirmLabel={confirmation.run === "compare" ? "Compare" : "Patch"}
            onConfirm={() => void onRun()}
            onClose={confirmation.close}
          />
        ) : null
      }
    >
      {patcherService.section === EPatcherSection.COMPARISON ? (
        <PatcherComparisonSection
          config={config}
          input={input}
          target={target}
          isDisabled={isBusy}
          onChange={patcherService.patchConfig}
        />
      ) : null}

      {patcherService.section === EPatcherSection.OUTPUT ? (
        <ArchiveConfigOutputSection
          editor={patcherService}
          config={config}
          fieldId={"patcher"}
          destination={destination}
          destinationDescription={"Directory the patch volumes are written into, outside the game"}
          nameLabel={"Volume name"}
          nameDescription={"Volumes are written as <name>.db0, <name>.db1 and so on"}
          isDisabled={isBusy}
        />
      ) : null}

      {patcherService.section === EPatcherSection.SELECTION ? (
        <PatcherSelectionSection config={config} isDisabled={isBusy} onChange={patcherService.patchConfig} />
      ) : null}

      {patcherService.section === EPatcherSection.HEADER ? (
        <ArchiveConfigHeaderSection
          editor={patcherService}
          config={config}
          fieldId={"patcher"}
          entryPointDescription={
            "Where the engine mounts the contents. A patch over a gamedata release wants the default"
          }
          missingEntryPointWarning={
            "The engine reads the entry point without checking whether it is there, so volumes without one stop the " +
            "game on load. Publishing refuses a header that omits it."
          }
          customEntryPointNotice={
            "A patch overrides entries by name, so it has to mount where the release it patches did. Every shipped " +
            `configuration uses ${DEFAULT_ENTRY_POINT}.`
          }
          isDisabled={isBusy}
        />
      ) : null}

      {patcherService.section === EPatcherSection.OPTIONS ? (
        <ArchiveConfigOptionsSection editor={patcherService} config={config} fieldId={"patcher"} isDisabled={isBusy}>
          <SwitchFormRow
            id={"patcher-verify-payload"}
            label={"Verify payloads"}
            description={"Confirm every checksum match by comparing the bytes, reading both sides in full"}
            isChecked={isVerifyingPayload}
            isDisabled={isBusy}
            onChange={setIsVerifyingPayload}
          />
        </ArchiveConfigOptionsSection>
      ) : null}
    </ArchiveConfigEditorLayout>
  );
}
