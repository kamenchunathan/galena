app [main!] {
    cli: platform "https://github.com/roc-lang/basic-cli/releases/download/0.19.0/Hj-J_zxz7V9YurCSTFcFdu6cQJie4guzsPMUi5kBYUk.tar.br",
    weaver: "https://github.com/smores56/weaver/releases/download/0.6.0/4GmRnyE7EFjzv6dDpebJoWWwXV285OMt4ntHIc6qvmY.tar.br",
}

import cli.Cmd
import cli.Stdout
import cli.Path
import cli.Env
import cli.Arg
import weaver.Opt
import weaver.Cli

## Builds the galena platform [platform](https://www.roc-lang.org/platforms).
##
## run with: roc ./build.roc
##
main! : _ => Result {} _
main! = |args|
    parsed_args =
        Result.on_err!(
            Cli.parse_or_display_message(cli_parser, args, Arg.to_os_raw),
            |message| Err(Exit(1, message)),
        )?

    run!(parsed_args)

cli_parser =
    Opt.maybe_str(
        {
            short: "p",
            long: "roc",
            help: "Path to the roc executable. Can be just `roc` or a full path.",
        },
    )
    |> Cli.finish(
        {
            name: "galena-builder",
            version: "",
            authors: ["Nathan Kamenchu <https://github.com/kamenchunathan>"],
            description: "",
        },
    )
    |> Cli.assert_valid

run! : Result Str err => Result {} _
run! = |maybe_roc|

    # roc_cmd may be a path or just roc
    roc_cmd = maybe_roc ?? "roc"

    roc_version!(roc_cmd)?

    os_and_arch = get_os_and_arch!({})?

    stub_lib_path = "platform/libapp.${stub_file_extension(os_and_arch)}"

    build_stub_app_lib!(roc_cmd, stub_lib_path)?

    cargo_build_backend_host!({})?

    rust_target_folder = get_rust_target_folder!({})?

    copy_host_lib!(os_and_arch, rust_target_folder)?

    preprocess_host!(roc_cmd, stub_lib_path, rust_target_folder)?

    build_frontend!(roc_cmd)?

    cargo_build_galena_cli!({})?

    info!("Successfully built platform files!")

roc_version! : Str => Result {} _
roc_version! = |roc_cmd|

    info!("Checking provided roc; executing `${roc_cmd} version`:")?

    Cmd.exec!(roc_cmd, ["version"])
    |> Result.map_err(RocVersionCheckFailed)

get_os_and_arch! : {} => Result OSAndArch _
get_os_and_arch! = |{}|
    info!("Getting the native operating system and architecture...")?
    convert_os_and_arch(Env.platform!({}))

build_frontend! : Str => Result {} _
build_frontend! = |roc_cmd|
    info!("Building frontend host & WebAssembly bindings ...")?

    cargo_build_frontend_host!({})?

    # build and copy the wasmbindgen output

    build_and_copy_wasmbindgen_js_to_frontend!(roc_cmd)?

    pnpm_build_frontend!({})?

    Ok {}

build_stub_app_lib! : Str, Str => Result {} _
build_stub_app_lib! = |roc_cmd, stub_lib_path|
    info!("Building stubbed app shared library ...")?
    Cmd.exec!(
        roc_cmd,
        [
            "build",
            "--lib",
            "platform/libapp.roc",
            "--output",
            stub_lib_path,
            "--optimize",
        ],
    )

get_rust_target_folder! : {} => Result Str _
get_rust_target_folder! = |{}|
    when Env.var!("CARGO_BUILD_TARGET") is
        Ok(target_env_var) ->
            info!("${target_env_var}")?
            if Str.is_empty(target_env_var) then
                Ok("target/release/")
            else
                Ok("target/${target_env_var}/release/")

        Err(e) ->
            info!("Failed to get env var CARGO_BUILD_TARGET with error ${Inspect.to_str(e)}. Assuming default CARGO_BUILD_TARGET (native)...")?

            Ok("target/release/")

cargo_build_galena_cli! : {} => Result {} _
cargo_build_galena_cli! = |{}|
    info!("Building Galena CLI ...")?

    exports = wasm_bindgen_exports!({})?

    temp_dir = Env.temp_dir!({})
    exports_file = "${Path.display temp_dir}/exports.txt"
    Path.write_bytes!(Str.to_utf8 exports, Path.from_str(exports_file))?

    cmd =
        Cmd.new("cargo")
        |> Cmd.args [
            "build",
            "--package",
            "galena_cli",
            "--package",
            "roc_backend_host_bin",
            "--release",
        ]
        |> Cmd.envs([("WASM_BINDGEN_EXPORTS", exports_file)])

    Cmd.status!(cmd) |> Result.map_ok (|_| {})

cargo_build_backend_host! : {} => Result {} _
cargo_build_backend_host! = |{}|
    info!("Building rust backend host ...")?

    Cmd.exec!(
        "cargo",
        [
            "build",
            "--package",
            "roc_backend_host_lib",
            "--package",
            "roc_backend_host_bin",
            "--release",
        ],
    )
    |> Result.map_err(ErrBuildingHostBinaries)

cargo_build_frontend_host! : {} => Result {} _
cargo_build_frontend_host! = |{}|
    info!("Building rust backend host ...")?

    Cmd.exec!(
        "cargo",
        [
            "build",
            "--package",
            "frontend_host",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ],
    )
    |> Result.map_err(ErrBuildingHostBinaries)

copy_host_lib! : OSAndArch, Str => Result {} _
copy_host_lib! = |os_and_arch, rust_target_folder|
    host_build_path = "${rust_target_folder}libhost.a"
    host_dest_path = "platform/${prebuilt_static_lib_file(os_and_arch)}"

    info!("Moving the prebuilt binary from ${host_build_path} to ${host_dest_path} ...")?

    Cmd.exec!("cp", [host_build_path, host_dest_path])
    |> Result.map_err(ErrMovingPrebuiltLegacyBinary)

OSAndArch : [
    MacosArm64,
    MacosX64,
    LinuxArm64,
    LinuxX64,
    WindowsArm64,
    WindowsX64,
]

convert_os_and_arch : _ -> Result OSAndArch _
convert_os_and_arch = |{ os, arch }|
    when (os, arch) is
        (MACOS, AARCH64) -> Ok(MacosArm64)
        (MACOS, X64) -> Ok(MacosX64)
        (LINUX, AARCH64) -> Ok(LinuxArm64)
        (LINUX, X64) -> Ok(LinuxX64)
        _ -> Err(UnsupportedNative(os, arch))

stub_file_extension : OSAndArch -> Str
stub_file_extension = |os_and_arch|
    when os_and_arch is
        MacosX64 | MacosArm64 -> "dylib"
        LinuxArm64 | LinuxX64 -> "so"
        WindowsX64 | WindowsArm64 -> "dll"

prebuilt_static_lib_file : OSAndArch -> Str
prebuilt_static_lib_file = |os_and_arch|
    when os_and_arch is
        MacosArm64 -> "macos-arm64.a"
        MacosX64 -> "macos-x64.a"
        LinuxArm64 -> "linux-arm64.a"
        LinuxX64 -> "linux-x64.a"
        WindowsArm64 -> "windows-arm64.lib"
        WindowsX64 -> "windows-x64.lib"

preprocess_host! : Str, Str, Str => Result {} _
preprocess_host! = |roc_cmd, stub_lib_path, rust_target_folder|

    info!("Preprocessing surgical host ...")?

    surgical_build_path = "${rust_target_folder}host"

    roc_cmd
    |> Cmd.exec!(["preprocess-host", surgical_build_path, "platform/main.roc", stub_lib_path])
    |> Result.map_err(ErrPreprocessingSurgicalBinary)

info! : Str => Result {} _
info! = |msg|
    Stdout.line!("\u(001b)[34mINFO:\u(001b)[0m ${msg}")

error! : Str => Result {} _
error! = |msg|
    Stdout.line!("\u(001b)[31mERROR:\u(001b)[0m ${msg}")

# warn! : Str => Result {} _
# warn! = |msg|
#     Stdout.line!("\u(001b)[33mWARN:\u(001b)[0m ${msg}")

wasm_bindgen_exports! : {} => Result Str _
wasm_bindgen_exports! = |{}|
    args = [
        "-c",
        Str.join_with(
            [
                "wasm2wat target/wasm32-unknown-unknown/release/frontend_host.wasm",
                "| grep -E '^\\s*\\(export'",
                "| grep '(func' ",
                "| sed -E 's/^\\s*\\(export \\\"([^\\\"]+)\\\" \\(func.*/\\1/'",
            ],
            " ",
        ),
    ]
    output =
        Cmd.new("sh")
        |> Cmd.args(args)
        |> Cmd.output!

    when output.status is
        Ok 0 ->
            Ok
                (
                    Str.join_with(
                        [
                            # IMPORTANT: Include the exported functions of the host,
                            #  even theough they are public and marked with wasm_bindgen, they
                            # will not be in the wasm build unless in this list
                            "run",
                            Str.from_utf8_lossy(output.stdout),
                        ],
                        "\n",
                    )
                )

        _ ->
            error!(Str.from_utf8_lossy output.stderr)?
            Err WasmBindGenSExportsFail

build_and_copy_wasmbindgen_js_to_frontend! : Str => Result {} _
build_and_copy_wasmbindgen_js_to_frontend! = |roc_cmd|
    info!("Building WASM stub ...")?

    temp_dir = Env.temp_dir!({})
    wasm_obj_path = "${Path.display(temp_dir)}/libapp-498da92kdowk.o"
    wasm_stub_stem = "libapp-32k392k3172l4"
    wasm_stub_path = "${Path.display(temp_dir)}/${wasm_stub_stem}.wasm"
    wasm_bindgen_dir = "${Path.display(temp_dir)}/libapp-kdl293948/"

    exports = wasm_bindgen_exports!({})?
    Cmd.exec!(
        roc_cmd,
        [
            "build",
            "platform/libapp.roc",
            "--target",
            "wasm32",
            "--no-link",
            "--output",
            wasm_obj_path,
        ],
    )?

    Cmd.exec!(
        "wasm-ld",
        Str.split_on(exports, "\n")
        |> List.keep_if(|s| !Str.is_empty(s))
        |> List.map(|fn| "--export=${fn}")
        |> List.concat [
            "--no-entry",
            "target/wasm32-unknown-unknown/release/libfrontend_host.a",
            wasm_obj_path,
            "-o",
            wasm_stub_path,
        ],
    )?

    Cmd.exec!(
        "wasm-bindgen",
        [
            "--typescript",
            "--target",
            "web",
            wasm_stub_path,
            "--out-dir",
            wasm_bindgen_dir,
        ],
    )?

    Cmd.exec!(
        "cp",
        [
            "${Path.display(temp_dir)}/libapp-kdl293948/${wasm_stub_stem}.js",
            "frontend/src/rocApp.js",
        ],
    )?

    Cmd.exec!(
        "cp",
        [
            "${Path.display(temp_dir)}/libapp-kdl293948/${wasm_stub_stem}.d.ts",
            "frontend/src/rocApp.d.ts",
        ],
    )?

    Ok {}

pnpm_build_frontend! : {} => Result {} _
pnpm_build_frontend! = |_|
    Cmd.exec!(
        "bash",
        [
            "-c",
            "cd frontend && pnpm build",
        ],
    )?

    Ok {}
