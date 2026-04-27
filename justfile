set dotenv-load := false

program    := "program"
program_so := "target/deploy/project_incognitus.so"

export SBF_OUT_DIR := justfile_directory() / "target" / "deploy"

# list available recipes
default:
    @just --list

# check workspace (fast — no codegen)
check:
    cargo check --workspace

# build program for SBF target
build-sbf:
    cargo build-sbf --manifest-path {{program}}/Cargo.toml --features bpf-entrypoint

# build program (native, debug)
build:
    cargo build -p project-incognitus

# run all tests (unit + integration, builds SBF automatically)
test: build-sbf
    cargo test -p project-incognitus --tests

# cargo clippy on workspace
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# format fix
fmt:
    cargo fmt --all

# remove build artifacts
clean:
    cargo clean

# show program binary size
size: build-sbf
    @ls -lh {{program_so}} | awk '{print $5, $9}'
