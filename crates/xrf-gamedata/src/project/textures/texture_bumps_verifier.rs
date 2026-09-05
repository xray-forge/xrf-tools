use rayon::prelude::*;
use xrf_material::{XrayMaterialBumpInput, XrayMaterialDeclaration, XrayMaterialDescriptor, XrayMaterialResolver};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType as AssetType;
use xrf_vfs::{XrayAsset, XrayProbe, XrayResolution};

use crate::GamedataFindingFactory;
use crate::project::textures::texture_bump_verification::{TextureBumpVerdict, TextureBumpVerification};
use crate::project::textures::texture_bumps_verification_result::GamedataTextureBumpsVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Every bump a texture descriptor asks for is the bump the engine will bind.
///
/// Walks descriptors the way `CTextureDescrMngr::LoadTHM` does — every `.thm`, whether or not a texture sits beside
/// it — and hands each to [`XrayMaterialResolver`], which applies the engine's own rules: the texture type that gates
/// the whole file, the bump name taken verbatim with no `_bump` convention behind it, the `#` companion derived from
/// it, and the dummy that `texture_load` substitutes for an absent name containing `_bump`. A name that resolves to
/// nothing does not turn bump mapping off, because `bump_exist` only tests that the name is non-empty: the renderer
/// still takes the bump shader path and the surface is flat anyway, with `! Fallback to default bump map` in the log.
///
/// Findings split by what fixes them: `textures.bump` wants the bump file, `textures.bump-companion` wants its `#`
/// file, and `textures.bump-declaration` wants the descriptor changed, since the engine never reads what it declares.
/// A missing companion is counted apart from a missing bump because it costs less: the engine binds
/// `ed\ed_dummy_bump#`, whose flat alpha removes parallax relief and whose constant error term leaves the DXT normal
/// uncorrected, while the surface still reads as bumped. Vanilla ships one such pair, so it fails only under strict.
pub(crate) struct TextureBumpsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> TextureBumpsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> GamedataTextureBumpsVerificationResult {
    let descriptors: Vec<XrayAsset> = self.project.entries_of_type(AssetType::Thm);
    let probe: XrayProbe = self
      .project
      .vfs()
      .probe()
      .with_step("project", self.project.scope.clone());

    // Descriptors are read in parallel; the sequence releases what each one says in path order.
    let sequence: OutputSequence = OutputSequence::new(&self.options.output, descriptors.len());

    let verifications: Vec<TextureBumpVerification> = descriptors
      .par_iter()
      .enumerate()
      .map(|(index, descriptor)| {
        let slot: OutputSlot = sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();
        let path: &str = descriptor.get_logical_path().as_str();

        Self::verify_descriptor(
          output,
          path,
          &XrayMaterialResolver::describe_descriptor(&probe, descriptor),
        )
      })
      .collect();

    let mut result: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      is_strict: self.options.is_strict,
      ..Default::default()
    };

    for verification in verifications {
      match verification.verdict {
        TextureBumpVerdict::Undeclared => {}
        TextureBumpVerdict::Bound {
          is_bump_missing,
          is_companion_missing,
        } => {
          result.checked_bumps_count += 1;
          result.unresolved_bumps_count += u32::from(is_bump_missing);
          result.missing_companions_count += u32::from(is_companion_missing);
        }
        TextureBumpVerdict::InvalidDeclaration => result.invalid_bump_declarations_count += 1,
      }

      result.findings.extend(verification.findings);
    }

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    result
  }

  /// What one descriptor's material comes to as findings, and which count it belongs in.
  fn verify_descriptor(
    output: &OutputOptions,
    path: &str,
    material: &XrayMaterialDescriptor,
  ) -> TextureBumpVerification {
    match &material.declaration {
      XrayMaterialDeclaration::NoDescriptor
      | XrayMaterialDeclaration::NoBumpChunk
      | XrayMaterialDeclaration::Disabled { .. } => TextureBumpVerification::undeclared(),
      XrayMaterialDeclaration::Unreadable { reason } => {
        // Reported by the descriptor rather than silently treated as declaring no bump.
        xrf_output::info!(output, "Texture descriptor is not readable: {path} - {reason}");

        TextureBumpVerification::undeclared().with_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesRead,
          path,
          format!("Texture descriptor is not readable: {reason}"),
        ))
      }
      XrayMaterialDeclaration::TypeDisqualified {
        label,
        declared_bump: Some(bump_name),
        ..
      } => {
        xrf_output::info!(
          output,
          "Texture descriptor declares bump the engine skips for its type: {path} -> {bump_name} ({label})"
        );

        TextureBumpVerification::invalid().with_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesBumpDeclaration,
          path,
          format!(
            "Texture descriptor declares bump '{bump_name}' but its texture type '{label}' is skipped by the engine"
          ),
        ))
      }
      XrayMaterialDeclaration::TypeDisqualified { .. } => TextureBumpVerification::undeclared(),
      XrayMaterialDeclaration::EmptyName { mode } => {
        xrf_output::info!(output, "Texture descriptor asks for a bump with an empty name: {path}");

        TextureBumpVerification::invalid().with_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesBumpDeclaration,
          path,
          format!("Texture descriptor asks for a bump in mode '{mode:?}' with an empty name"),
        ))
      }
      XrayMaterialDeclaration::Declared { name, .. } => {
        let Some(bump) = &material.bump else {
          return TextureBumpVerification::undeclared();
        };

        let mut verification: TextureBumpVerification =
          TextureBumpVerification::bound(!bump.bump.is_declared_file(), !bump.companion.is_declared_file());

        if let Some(bound) = Self::describe_substitution(&bump.bump) {
          xrf_output::info!(output, "Texture descriptor declares missing bump: {path} -> {name}");

          verification = verification.with_finding(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::TexturesBump,
            path,
            format!("Texture descriptor declares bump '{name}' that is not in gamedata, the engine binds {bound}"),
          ));
        }

        if let Some(bound) = Self::describe_substitution(&bump.companion) {
          xrf_output::warning!(
            output,
            "Texture descriptor declares bump without its companion: {path} -> {}",
            bump.companion.reference
          );

          verification = verification.with_finding(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::TexturesBumpCompanion,
            path,
            format!(
              "Texture descriptor declares bump '{name}' whose companion '{}' is not in gamedata, the engine binds {bound}",
              bump.companion.reference
            ),
          ));
        }

        verification
      }
    }
  }

  /// What the renderer puts on the surface for an input that did not resolve, or `None` for one that did.
  fn describe_substitution(input: &XrayMaterialBumpInput) -> Option<String> {
    match &input.resolution {
      XrayResolution::Resolved { .. } => None,
      XrayResolution::Substituted { fallback, .. } => Some(format!("'{fallback}'")),
      XrayResolution::Missing { .. } | XrayResolution::NoScope | XrayResolution::Rejected { .. } => {
        Some(String::from("nothing"))
      }
    }
  }
}
