# Java→Rust porting pipeline (HaplotypeCaller defaults).
# Beginner guide: docs/guides/PORTING_WORKFLOW.md
# Override: make sources RUST_LOCAL=../my-port

JAVA_ALIAS ?= gatk-java-hc
JAVA_GIT   ?= https://github.com/broadinstitute/gatk.git
JAVA_SUBPATH ?= src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller
RUST_ALIAS ?= hc-rust
RUST_LOCAL ?= ../my-hc-port
GRAPH_FILTER ?= callable,calls,type,defines

.PHONY: sources graph-java graph-rust graph map diff \
        graph-export graph-export-java graph-export-rust graph-export-svg \
        open-report install-hooks clean-cache e2e-fixture

install-hooks:
	@bash scripts/install-hooks.sh

## Fixture e2e (no network): mini Java/Rust trees → diff report.
e2e-fixture:
	cargo test -p s4-cli --test e2e_mini_port -- --nocapture

sources:
	cargo run -p s4-cli -- source add $(JAVA_ALIAS) --git $(JAVA_GIT) --subpath $(JAVA_SUBPATH) --lang java
	cargo run -p s4-cli -- source add $(RUST_ALIAS) --local $(RUST_LOCAL) --lang rust

graph-java:
	cargo run -p s4-cli -- graph build --source $(JAVA_ALIAS)

graph-rust:
	cargo run -p s4-cli -- graph build --source $(RUST_ALIAS)

graph: graph-java graph-rust

graph-export-java: graph-java
	cargo run -p s4-cli -- graph export --source $(JAVA_ALIAS) --format dot --filter $(GRAPH_FILTER)

graph-export-rust: graph-rust
	cargo run -p s4-cli -- graph export --source $(RUST_ALIAS) --format dot --filter $(GRAPH_FILTER)

graph-export: graph-export-java graph-export-rust

graph-export-svg: graph-export-rust
	dot -Tsvg .s4/exports/$(RUST_ALIAS).dot -o .s4/exports/$(RUST_ALIAS).svg
	@echo "Open .s4/exports/$(RUST_ALIAS).svg"

map: graph
	cargo run -p s4-cli -- map suggest --java $(JAVA_ALIAS) --rust $(RUST_ALIAS)

diff: map
	cargo run -p s4-cli -- diff --java $(JAVA_ALIAS) --rust $(RUST_ALIAS)
	@echo "Report: .s4/reports/diff-report.md"

open-report:
	@echo .s4/reports/diff-report.md

clean-cache:
	rm -rf .s4/cache .s4/store .s4/graphs
