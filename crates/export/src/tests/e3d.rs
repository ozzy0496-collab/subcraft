use super::*;
use crate::e3d;

#[test]
fn element_3d_writes_one_file_per_e3d_slot() {
    let (dir, names) = export_with(ExportTemplate::element_3d(), "e3d");
    assert_eq!(names, ["M_P_Diffuse.png", "M_P_Reflectivity.png", "M_P_Glossiness.png", "M_P_Occlusion.png", "M_P_Illumination.png", "M_P_NormalBump.png"]);

    let diffuse = px(&dir.join("M_P_Diffuse.png"), 9, 9);
    assert!(near(diffuse[0], linear_to_srgb(0.5)), "diffuse is sRGB base color: {diffuse:?}");
    let refl = px(&dir.join("M_P_Reflectivity.png"), 9, 9);
    assert!(near(refl[0], 0.8) && near(refl[1], 0.8) && near(refl[2], 0.8), "reflectivity is gray metallic: {refl:?}");
    let gloss = px(&dir.join("M_P_Glossiness.png"), 9, 9);
    assert!(near(gloss[0], 0.7) && near(gloss[2], 0.7), "glossiness is 1 - roughness: {gloss:?}");
    let ao = px(&dir.join("M_P_Occlusion.png"), 9, 9);
    assert!(near(ao[0], 0.6) && near(ao[1], 0.6), "occlusion is gray: {ao:?}");
    let illum = px(&dir.join("M_P_Illumination.png"), 9, 9);
    assert!(near(illum[2], linear_to_srgb(0.6)), "illumination is sRGB emission: {illum:?}");
    let n = px(&dir.join("M_P_NormalBump.png"), 9, 9);
    assert!(near(n[1], 0.8), "OpenGL (+Y) normals by default: {n:?}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn element_3d_normals_follow_the_directx_override() {
    let mesh = primitives::plane(2.0, 1);
    let ch = distinct_channels();
    let sets = [ExportTextureSet { name: "P", material_slot: 0, udim: None, channels: &ch, mesh_maps: None }];
    let dir = tmp("e3d-dx");
    let settings = ExportSettings { template: ExportTemplate::element_3d(), normal_directx: Some(true), ..ExportSettings::default() };
    export(&mesh, "M", &sets, &settings, &dir).unwrap();
    let n = px(&dir.join("M_P_NormalBump.png"), 9, 9);
    assert!(near(n[1], 1.0 - 0.8), "DirectX flips green: {n:?}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn size_warning_only_for_e3d_above_the_default_limit() {
    let e3d_t = ExportTemplate::element_3d();
    assert!(e3d::is_e3d(&e3d_t));
    assert_eq!(e3d::size_warning(&e3d_t, [1024, 2048]), None);
    let w = e3d::size_warning(&e3d_t, [2048, 4096]).unwrap();
    assert!(w.contains("4096") && w.contains("Map Size Limit"), "{w}");
    assert_eq!(e3d::size_warning(&ExportTemplate::unreal(), [8192]), None);
    assert_eq!(e3d::size_warning(&e3d_t, []), None);
}
