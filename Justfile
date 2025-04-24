build_dir := 'out'

roc_bin :=  "$ROC_SRC_CODE_PATH/target/release/roc"
examples_dir := 'examples/'
dist_dir := "dist"
frontend_pkg := "frontend"
patch_script := "./scripts/patch_triple.sh"



alias b := build


default:
    @just --list

gen-glue:
    #!/usr/bin/env bash
    set -euxo pipefail
    roc glue "$ROC_SRC_CODE_PATH/crates/glue/src/ZigGlue.roc" platform/glue platform/main.roc

build:
    just build-platform
    just build-examples

build-wasm file_path:
    #!/usr/bin/env bash
    set -euxo pipefail

    mkdir -p {{ build_dir}}
    file_path=$(basename -s .roc {{ file_path }} )

    {{ roc_bin }} build \
        --target wasm32 {{ file_path }} \
        --output "{{build_dir}}/$file_path.wasm"

    wasm2wat "{{build_dir}}/$file_path.wasm" \
        -o "{{build_dir}}/$file_path.wat"

    
build-platform:
    roc build.roc

build-examples:
    #!/usr/bin/env bash
    set -euxo pipefail

    if [ ! -d {{ examples_dir }} ]; then
        echo "Error: Examples directory '{{ examples_dir }}' not found."
        exit 1
    fi

    mkdir -p "{{ build_dir }}/examples/"
    find {{ examples_dir }} -type f -name '*.roc' -print0 | while IFS= read -r -d $'\0' roc_source_file; do
        {{ roc_bin }} build $roc_source_file --output  "{{ build_dir }}/examples/"
    done

roc-clean:
    rm platform/{dynhost,libapp.so,linux-x64.rh,metadata_linux-x64.rm}



build-frontend:
    #!/bin/sh
    temp_dir=$(mktemp -d -t roc_frontend_build_XXXXXX)

    mkdir -p {{dist_dir}}
    
    cargo clean --package {{frontend_pkg}} --target wasm32-unknown-unknown --profile release
    RUSTFLAGS='--emit=llvm-bc' cargo build --package {{frontend_pkg}} --release --target wasm32-unknown-unknown
    llvm-link target/wasm32-unknown-unknown/release/deps/*.bc -o "$temp_dir/frontend_linked.bc"
    llvm-dis "$temp_dir/frontend_linked.bc" -o "$temp_dir/frontend_linked.ll"
    {{patch_script}} "$temp_dir/frontend_linked.ll" "$temp_dir/frontend_patched.ll"
    llvm-as "$temp_dir/frontend_patched.ll" -o {{dist_dir}}/frontend.o

    rm -rf "$temp_dir"


