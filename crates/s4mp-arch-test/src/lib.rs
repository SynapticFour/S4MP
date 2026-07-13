//! Architecture enforcement tests for S4MP dependency tiers.

#[cfg(test)]
mod tier_tests {
    use cargo_metadata::{MetadataCommand, PackageId};
    use std::collections::{HashMap, HashSet};

    /// Tier assignments per crate name.
    fn crate_tiers() -> HashMap<&'static str, u8> {
        HashMap::from([
            ("s4mp-core", 0),
            ("s4mp-schema", 0),
            ("s4mp-store", 0),
            ("s4mp-ir", 1),
            ("s4mp-model", 1),
            ("s4mp-graph", 1),
            ("s4mp-query", 1),
            ("s4mp-plugin-api", 2),
            ("s4mp-plugin-sdk", 2),
            ("s4mp-plugin-host", 2),
            ("s4mp-plugin-registry", 2),
            ("s4mp-import", 3),
            ("s4mp-parse", 3),
            ("s4mp-link", 3),
            ("s4mp-analyze", 3),
            ("s4mp-reason", 3),
            ("s4mp-verify", 3),
            ("s4mp-pipeline", 4),
            ("s4mp-workspace", 4),
            ("s4mp-jobs", 4),
            ("s4mp-cli", 5),
            ("s4mp-api", 5),
            ("s4mp-client", 5),
        ])
    }

    fn s4mp_package_ids(metadata: &cargo_metadata::Metadata) -> HashMap<String, PackageId> {
        metadata
            .packages
            .iter()
            .filter(|p| p.name.starts_with("s4mp-") || p.name.starts_with("s4mp_"))
            .map(|p| (p.name.clone(), p.id.clone()))
            .collect()
    }

    #[test]
    fn dependency_direction_is_inward_only() {
        let metadata = MetadataCommand::new()
            .exec()
            .expect("cargo metadata");

        let tiers = crate_tiers();
        let packages = s4mp_package_ids(&metadata);

        for package in &metadata.packages {
            if !package.name.starts_with("s4mp-") {
                continue;
            }
            let Some(&from_tier) = tiers.get(package.name.as_str()) else {
                continue;
            };

            for dep in &package.dependencies {
                if !dep.name.starts_with("s4mp-") {
                    continue;
                }
                let Some(&to_tier) = tiers.get(dep.name.as_str()) else {
                    continue;
                };
                assert!(
                    from_tier >= to_tier,
                    "{} (tier {}) must not depend on {} (tier {})",
                    package.name,
                    from_tier,
                    dep.name,
                    to_tier
                );
            }
        }

        let _ = packages;
    }

    #[test]
    fn knowledge_tier_does_not_depend_on_reasoning() {
        let metadata = MetadataCommand::new()
            .exec()
            .expect("cargo metadata");

        let forbidden: HashSet<&str> = ["s4mp-reason"].into_iter().collect();

        for package in &metadata.packages {
            if !matches!(
                package.name.as_str(),
                "s4mp-ir" | "s4mp-model" | "s4mp-graph" | "s4mp-query"
            ) {
                continue;
            }
            for dep in &package.dependencies {
                assert!(
                    !forbidden.contains(dep.name.as_str()),
                    "{} must not depend on {}",
                    package.name,
                    dep.name
                );
            }
        }
    }
}
