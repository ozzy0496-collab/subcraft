//! Element 3D (Video Copilot, After Effects) export: one file per E3D texture slot.
//!
//! E3D has no metallic/roughness workflow and no Specular slot. Its slots are Diffuse,
//! Reflectivity, Glossiness, Illumination, Normal Bump, Occlusion and Environment (the last is an
//! HDRI, not a per-model map). See `docs/e3d.md` for what was verified in E3D itself.

use subcraft_core::ChannelKind;
use subcraft_image::FileFormat;

use crate::template::{OutputMap, Source, normal, rgb};
use crate::ExportTemplate;

/// Template name; also how project settings remember the choice.
pub const TEMPLATE_NAME: &str = "Element 3D (After Effects)";

/// E3D shrinks larger textures unless Preferences → Map Size Limit is raised (up to 8192).
pub const DEFAULT_MAP_SIZE_LIMIT: u32 = 2048;

impl ExportTemplate {
    /// Element 3D: Diffuse, Reflectivity (metallic), Glossiness (1 − roughness), Occlusion,
    /// Illumination (emission) and Normal Bump (OpenGL +Y by default; switch the normal
    /// convention in the export options if E3D shows the bumps inverted).
    pub fn element_3d() -> Self {
        use ChannelKind::*;
        let gray = |s: Source| vec![s; 3];
        Self {
            name: TEMPLATE_NAME.into(),
            maps: vec![
                OutputMap::new("Diffuse", FileFormat::Png8, rgb(BaseColor), true),
                OutputMap::new("Reflectivity", FileFormat::Png8, gray(Source::ch(Metallic, 0)), false),
                OutputMap::new("Glossiness", FileFormat::Png8, gray(Source::Channel { kind: Roughness, component: 0, invert: true }), false),
                OutputMap::new("Occlusion", FileFormat::Png8, gray(Source::ch(AmbientOcclusion, 0)), false),
                OutputMap::new("Illumination", FileFormat::Png8, rgb(Emissive), true),
                OutputMap::new("NormalBump", FileFormat::Png8, normal(false), false),
            ],
        }
    }
}

/// Whether `template` is the Element 3D one.
pub fn is_e3d(template: &ExportTemplate) -> bool {
    template.name == TEMPLATE_NAME
}

/// Warning for the export dialog when a texture set is larger than E3D's default map size limit.
pub fn size_warning(template: &ExportTemplate, resolutions: impl IntoIterator<Item = u32>) -> Option<String> {
    if !is_e3d(template) {
        return None;
    }
    let largest = resolutions.into_iter().max()?;
    (largest > DEFAULT_MAP_SIZE_LIMIT).then(|| {
        format!(
            "{largest}px maps: Element 3D shrinks textures above {DEFAULT_MAP_SIZE_LIMIT}px by default. \
             Raise Preferences → Map Size Limit in E3D (up to 8192)."
        )
    })
}
