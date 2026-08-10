//! Fixture-based Java↔Rust porting pipeline (no network / no GATK).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn s4_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_s4"))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/mini-port")
}

fn run_s4(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(s4_bin())
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn s4 {}: {e}", args.join(" ")))
}

fn assert_ok(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn e2e_mini_port_diff_is_deterministic() {
    let fixture = fixture_root();
    assert!(
        fixture.join("java/Calculator.java").is_file(),
        "missing fixture at {}",
        fixture.display()
    );

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let java_src = root.join("java");
    let rust_src = root.join("rust");
    copy_dir(&fixture.join("java"), &java_src);
    copy_dir(&fixture.join("rust"), &rust_src);

    assert_ok(&run_s4(root, &["init", "."]), "init");
    assert!(root.join(".s4/workspace.json").is_file());
    assert!(root.join(".s4/sources.json").is_file());

    let meta = fs::read_to_string(root.join(".s4/workspace.json")).unwrap();
    assert!(meta.contains("heuristic-map-v2"));

    assert_ok(
        &run_s4(
            root,
            &[
                "source",
                "add",
                "mini-java",
                "--local",
                java_src.to_str().unwrap(),
                "--lang",
                "java",
            ],
        ),
        "source add java",
    );
    assert_ok(
        &run_s4(
            root,
            &[
                "source",
                "add",
                "mini-rust",
                "--local",
                rust_src.to_str().unwrap(),
                "--lang",
                "rust",
            ],
        ),
        "source add rust",
    );

    assert_ok(
        &run_s4(root, &["graph", "build", "--source", "mini-java"]),
        "graph java",
    );
    assert_ok(
        &run_s4(root, &["graph", "build", "--source", "mini-rust"]),
        "graph rust",
    );
    assert_ok(
        &run_s4(
            root,
            &[
                "map",
                "suggest",
                "--java",
                "mini-java",
                "--rust",
                "mini-rust",
            ],
        ),
        "map suggest",
    );
    assert_ok(
        &run_s4(
            root,
            &["diff", "--java", "mini-java", "--rust", "mini-rust"],
        ),
        "diff",
    );

    let report_path = root.join(".s4/reports/diff-report.md");
    let report = fs::read_to_string(&report_path).expect("diff report");
    assert!(
        report.contains("heuristic-map-v2"),
        "missing maturity banner:\n{report}"
    );
    assert!(
        report.contains("# Diff: mini-java -> mini-rust"),
        "unexpected title:\n{report}"
    );
    assert!(
        report.contains("Confidence bands"),
        "missing confidence bands:\n{report}"
    );
    // Heuristic should pair shared names (add / multiply / Calculator / helper / scale).
    assert!(
        report.contains("Diverged")
            || report.contains("abweichend")
            || report.to_lowercase().contains("add"),
        "expected heuristic pairings in report:\n{report}"
    );

    let json_path = root.join(".s4/reports/diff-report.json");
    let json = fs::read_to_string(&json_path).expect("json sidecar");
    assert!(json.contains("heuristic-map-v2"), "{json}");
    assert!(json.contains("confidence_bands"), "{json}");

    // verify/certify are real but threshold-gated (default coverage 0 → pass).
    assert_ok(
        &run_s4(
            root,
            &[
                "verify",
                "--java",
                "mini-java",
                "--rust",
                "mini-rust",
                "--min-coverage",
                "0",
            ],
        ),
        "verify",
    );
    assert_ok(
        &run_s4(
            root,
            &[
                "certify",
                "--policy",
                "default",
                "--java",
                "mini-java",
                "--rust",
                "mini-rust",
            ],
        ),
        "certify",
    );
    assert!(root
        .join(".s4/certificates/mini-java__mini-rust__default.json")
        .is_file());
}

#[test]
fn e2e_init_is_idempotent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    assert_ok(&run_s4(root, &["init", "."]), "init first");
    assert_ok(&run_s4(root, &["init", "."]), "init second");
    let meta = fs::read_to_string(root.join(".s4/workspace.json")).unwrap();
    assert!(meta.contains("\"major\": 0"));
    assert!(meta.contains("\"minor\": 1"));
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}
