build_dir := 'out'

roc_bin :=  "$ROC_SRC_CODE_PATH/target/release/roc"
examples_dir := 'examples/'
frontend_dist_dir := "./frontend/dist"


alias b := build


default:
    @just --list

gen-glue:
    #!/usr/bin/env bash
    set -euxo pipefail
    roc glue "$ROC_SRC_CODE_PATH/crates/glue/src/ZigGlue.roc" platform/glue platform/main.roc

build:
    just build-frontend
    just build-platform
    just build-examples

build-frontend:
    #!/usr/bin/env bash
    set -euxo pipefail
    cd frontend && pnpm build

build-platform:
    #!/usr/bin/env bash
    set -euxo pipefail

    roc check ./platform/libapp.roc

    roc ./scripts/build.roc

build-examples:
    #!/usr/bin/env bash
    set -euxo pipefail

    if [ ! -d {{ examples_dir }} ]; then
        echo "Error: Examples directory '{{ examples_dir }}' not found."
        exit 1
    fi

    mkdir -p "{{ build_dir }}/examples/"
    find {{ examples_dir }} -type f -name '*.roc' -print0 | while IFS= read -r -d $'\0' roc_source_file; do
        roc build $roc_source_file --output  "{{ build_dir }}/examples/"
    done

clean:
    cargo clean
    rm -f platform/{dynhost,libapp.so,linux-x64.{a,rh},metadata_linux-x64.rm}
    
