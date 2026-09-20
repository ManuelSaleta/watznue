use watznue_core::detector::get_demo_packages;
use watznue_core::models::CategoryTier;
use watznue_core::normalizer::ChangelogNormalizer;

#[test]
fn test_demo_pipeline_normalization() {
    let packages = get_demo_packages();
    assert!(!packages.is_empty(), "Demo packages must not be empty");

    let mut normalized = Vec::new();
    for pkg in &packages {
        let item = ChangelogNormalizer::normalize(pkg, None, None);
        assert!(!item.bullets.is_empty(), "Item bullets must not be empty");
        normalized.push(item);
    }

    // Verify categories present in demo set
    let categories: Vec<CategoryTier> = normalized.iter().map(|item| item.category).collect();
    assert!(
        categories.contains(&CategoryTier::Core),
        "Demo should contain Core packages (kernel, mesa, pipewire)"
    );
    assert!(
        categories.contains(&CategoryTier::Desktop),
        "Demo should contain Desktop packages (mutter)"
    );
    assert!(
        categories.contains(&CategoryTier::Applications),
        "Demo should contain Application packages (firefox)"
    );

    // Verify JSON serialization of the full list
    let json_output = serde_json::to_string(&normalized).expect("Must serialize to JSON");
    assert!(json_output.contains("kernel"));
    assert!(json_output.contains("firefox"));
}
