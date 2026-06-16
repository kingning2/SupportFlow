//! Skills version resolution tests.

use tauri_app_lib::services::agent::skills::{SkillLoader, SkillManager};

fn write_skill(dir: &std::path::Path, folder: &str, name: &str, version: &str) {
    let skill_dir = dir.join(folder);
    std::fs::create_dir_all(&skill_dir).expect("mkdir");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            "---\nname: {name}\nversion: {version}\ndescription: test skill {version}\n---\n\n# Body\n"
        ),
    )
    .expect("write skill");
}

#[test]
fn two_versions_coexist_and_resolve_by_spec() {
    let root = std::env::temp_dir().join(format!("sf-skills-ver-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let builtin = root.join("builtin");
    let custom = root.join("custom");
    write_skill(&builtin, "demo-skill", "demo-skill", "1.0.0");
    write_skill(&builtin, "demo-skill-v2", "demo-skill", "2.0.0");

    let loader = SkillLoader;
    let loaded = loader.load_all_skills(&builtin, &custom);
    assert!(loaded.skills.len() >= 2);

    let mgr = SkillManager::new(&root, Some(builtin));
    let v1 = mgr.resolve_skill("demo-skill@1.0.0").expect("v1");
    assert_eq!(v1.skill.version, "1.0.0");
    let v2 = mgr.resolve_skill("demo-skill@2.0.0").expect("v2");
    assert_eq!(v2.skill.version, "2.0.0");
    let latest = mgr.resolve_skill("demo-skill").expect("latest");
    assert_eq!(latest.skill.version, "2.0.0");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn invalid_parameters_skips_skill() {
    let root = std::env::temp_dir().join(format!("sf-skills-bad-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("bad");
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join("SKILL.md"),
        "---\nname: bad-skill\nversion: 1.0.0\nparameters: not-json\ndescription: x\n---\n",
    )
    .expect("write");

    let loader = SkillLoader;
    let loaded = loader.load_skills_from_dir(&dir, "builtin");
    assert!(loaded.skills.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}
