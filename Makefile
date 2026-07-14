# Java→Rust porting pipeline (HaplotypeCaller defaults).
# Beginner guide: docs/guides/PORTING_WORKFLOW.md
# Override: make sources RUST_LOCAL=../my-port

JAVA_ALIAS ?= gatk-java-hc
JAVA_GIT   ?= https://github.com/broadinstitute/gatk.git
JAVA_SUBPATH ?= src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller
RUST_ALIAS ?= hc-rust
RUST_LOCAL ?= ../my-hc-port

.PHONY: sources graph-java graph-rust graph map diff clean-cache

sources:
	cargo run -p s4-cli -- source add $(JAVA_ALIAS) --git $(JAVA_GIT) --subpath $(JAVA_SUBPATH) --lang java
	cargo run -p s4-cli -- source add $(RUST_ALIAS) --local $(RUST_LOCAL) --lang rust

graph-java:
	cargo run -p s4-cli -- graph --source $(JAVA_ALIAS)

graph-rust:
	cargo run -p s4-cli -- graph --source $(RUST_ALIAS)

graph: graph-java graph-rust

map: graph
	cargo run -p s4-cli -- map suggest --java $(JAVA_ALIAS) --rust $(RUST_ALIAS)

diff: map
	cargo run -p s4-cli -- diff --java $(JAVA_ALIAS) --rust $(RUST_ALIAS) --out diff-report.md
	@echo "Report: diff-report.md"

clean-cache:
	rm -rf .s4/cache .s4/store .s4/graphs
