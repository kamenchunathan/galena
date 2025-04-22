build_dir := 'out'


default:
    @just --list

gen-glue:
    #!/usr/bin/env bash
    set -euxo pipefail
    roc glue "$ROC_SRC_CODE_PATH/crates/glue/src/ZigGlue.roc" platform/glue platform/main.roc

build-wasm file_path:
    #!/usr/bin/env bash
    set -euxo pipefail

    mkdir -p {{ build_dir}}
    file_path=$(basename -s .roc {{ file_path }} )

    "$ROC_SRC_CODE_PATH/target/release/roc" build \
        --target wasm32 {{ file_path }} \
        --output "{{build_dir}}/$file_path.wasm"

    wasm2wat "{{build_dir}}/$file_path.wasm" \
        -o "{{build_dir}}/$file_path.wat"


roc-clean:
    rm platform/{dynhost,libapp.so,linux-x64.rh,metadata_linux-x64.rm}
