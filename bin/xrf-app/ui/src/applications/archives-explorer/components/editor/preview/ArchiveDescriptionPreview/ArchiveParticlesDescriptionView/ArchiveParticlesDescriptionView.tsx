import { formatCount } from "@xrf/format";
import {
  ArchiveDescribeScope,
  ArchiveParticlesDescription,
  ArchiveParticlesEffect,
  ArchiveParticlesGroup,
} from "@xrf/ipc/types/xrf-app-archives";
import { filterByName } from "@xrf/search";
import { EditorFilterInput } from "@xrf/ui/editor/filter-input";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveParticlesEffectRow } from "./ArchiveParticlesEffectRow";
import { ArchiveParticlesGroupRow } from "./ArchiveParticlesGroupRow";

interface IArchiveParticlesDescriptionViewProps extends BaseComponentProps {
  description: ArchiveParticlesDescription;
  scope: ArchiveDescribeScope;
}

/**
 * The particle library: the emitters it defines and the sequences that play them.
 */
export function ArchiveParticlesDescriptionView({
  "data-testid": dataTestId = "archive-particles-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveParticlesDescriptionViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { library, effects, groups } = description;

  const matchedEffects: Array<ArchiveParticlesEffect> = useMemo(
    () => filterByName(effects, filter, (effect: ArchiveParticlesEffect) => effect.name),
    [effects, filter]
  );

  const matchedGroups: Array<ArchiveParticlesGroup> = useMemo(
    () => filterByName(groups, filter, (group: ArchiveParticlesGroup) => group.name),
    [filter, groups]
  );

  const isFiltered: boolean = Boolean(filter.trim());

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Library"} isFirst>
        <ArchiveDescriptionRow
          label={"Effects"}
          value={formatCount(library.effects)}
          caption={`Built from ${formatCount(library.actions)} actions between them`}
        />

        <ArchiveDescriptionRow
          label={"Groups"}
          value={formatCount(library.groups)}
          caption={
            library.undefinedEffects
              ? `Naming ${formatCount(library.undefinedEffects)} ${library.undefinedEffects === 1 ? "effect" : "effects"} this library does not define`
              : "Every effect they name is defined here"
          }
        />

        <ArchiveDescriptionRow
          label={"Textures"}
          value={formatCount(library.textures)}
          caption={
            library.absentTextures
              ? `${formatCount(library.absentTextures)} not held by what is open`
              : "All held by what is open"
          }
        />

        <ArchiveDescriptionRow label={"Version"} value={`${library.version}`} />
      </EditorPanelSection>

      <div className={"px-4 pt-2"}>
        <EditorFilterInput
          query={filter}
          placeholder={"Filter effects and groups"}
          ariaLabel={"Filter effects and groups"}
          onQueryChange={setFilter}
        />
      </div>

      <EditorPanelSection
        title={
          isFiltered
            ? `Effects (${formatCount(matchedEffects.length)} of ${formatCount(effects.length)})`
            : `Effects (${formatCount(effects.length)})`
        }
        caption={"In the order the library declares them"}
      >
        {matchedEffects.length ? (
          matchedEffects.map((effect: ArchiveParticlesEffect, index: number) => (
            <ArchiveParticlesEffectRow key={`${index}-${effect.name}`} effect={effect} scope={scope} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No effect of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>

      <EditorPanelSection
        title={
          isFiltered
            ? `Groups (${formatCount(matchedGroups.length)} of ${formatCount(groups.length)})`
            : `Groups (${formatCount(groups.length)})`
        }
        caption={"Each slot names an effect, and the effects it starts alongside it"}
      >
        {matchedGroups.length ? (
          matchedGroups.map((group: ArchiveParticlesGroup, index: number) => (
            <ArchiveParticlesGroupRow key={`${index}-${group.name}`} group={group} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No group of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
