alias b := build

default:
    @just build

gen-glue:
    #!/usr/bin/env bash
    set -euxo pipefail
    
    roc glue "$ROC_SRC_CODE_PATH/crates/glue/src/RustGlue.roc" crates platform/glue.roc
    rm -r crates/roc_std

    echo -e \
        "[package]\n" \
        "name = \"roc_app\"\n" \
        "description = \"This was generated by \`roc glue\`. It provides glue between a specific Roc platform and a Rust host.\"\n" \
        "version = \"1.0.0\"\n" \
        "edition = \"2021\"\n" \
        "\n" \
        "[dependencies]\n" \
        "roc_std.workspace = true" >  crates/roc_app/Cargo.toml

build:
    #!/usr/bin/env bash
    set -euxo pipefail

    roc check ./platform/libapp.roc

    roc ./scripts/build.roc

run FILE:
    target/release/cli watch {{ FILE }} --paths platform
    
clean:
    cargo clean
    rm -f platform/{dynhost,libapp.so,linux-x64.{a,rh},metadata_linux-x64.rm}
    rm -r .galena
