roc_bin :=  "$ROC_SRC_CODE_PATH/target/release/roc"


alias b := build

default:
    @just build

gen-glue:
    #!/usr/bin/env bash
    set -euxo pipefail
    roc glue "$ROC_SRC_CODE_PATH/crates/glue/src/RustGlue.roc" platform/glue platform/glue.roc

build:
    #!/usr/bin/env bash
    set -euxo pipefail

    roc check ./platform/libapp.roc

    roc ./scripts/build.roc
    
clean:
    cargo clean
    rm -f platform/{dynhost,libapp.so,linux-x64.{a,rh},metadata_linux-x64.rm}
