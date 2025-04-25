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
