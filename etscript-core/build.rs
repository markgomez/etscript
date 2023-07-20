use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    process::Command,
};

fn main() {
    let dotnet_proj_dir = "etscript-dotnet/Functions";
    let pkg_path = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let pkg_path_str = pkg_path.to_str().unwrap();
    let config = env::var_os("PROFILE").unwrap();
    let config_str = config.to_str().unwrap();
    let rid_os = match OS {
        "windows" => "win",
        "linux" => "linux",
        "macos" => "osx",
        _ => {
            panic!(
                "
Unsupported platform — .NET native AOT is restricted to Windows, Linux, and macOS:
https://learn.microsoft.com/dotnet/core/deploying/native-aot/#platformarchitecture-restrictions
"
            )
        }
    };
    let rid_arch = match ARCH {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "arm64",
        _ => {
            panic!(
                "
Unsupported architecture — .NET native AOT is restricted to x64 and Arm64:
https://learn.microsoft.com/dotnet/core/deploying/native-aot/#platformarchitecture-restrictions
"
            )
        }
    };
    let publish_path = &format!(
        "{}/../{dotnet_proj_dir}/bin/{}/net8.0/{rid_os}-{rid_arch}/native",
        pkg_path_str, config_str
    );

    Command::new("dotnet")
        .args([
            "publish",
            &format!("../{dotnet_proj_dir}"),
            "-p:NativeLib=shared",
        ])
        .args(["-r", &format!("{rid_os}-{rid_arch}")])
        .args(["-c", config_str])
        .status()
        .expect("`dotnet publish` command should have been executed.");

    let dl_prefix = "lib";
    let dl_name = "etscript_dotnet";
    let dl_suffix = match rid_os {
        "win" => ".dll",
        "osx" => ".dylib",
        _ => ".so",
    };
    let dl_file = format!("{dl_prefix}{dl_name}{dl_suffix}");

    // See: https://github.com/dotnet/runtime/issues/84500
    if rid_os == "osx" {
        Command::new("install_name_tool")
            .args([
                "-id",
                &format!("@loader_path/{dl_file}"),
                &format!("{publish_path}/{dl_file}"),
            ])
            .status()
            .expect("`install_name_tool` command should have been executed.");
    }

    let r_path = &format!("{}/../target/{}", pkg_path_str, config_str);

    std::fs::copy(
        format!("{publish_path}/{dl_file}"),
        format!("{r_path}/{dl_file}"),
    )
    .unwrap_or_else(|error| {
        panic!("{:?}", error);
    });

    if rid_os == "win" {
        println!("cargo:rustc-link-lib={dl_prefix}{dl_name}");
    } else {
        println!("cargo:rustc-link-lib={dl_name}");
    }
    println!("cargo:rustc-link-search=native={publish_path}");
}
