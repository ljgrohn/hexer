use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=assets/hexer.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let version = env!("CARGO_PKG_VERSION");
    let mut version_parts = version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .take(4)
        .map(|part| part.parse::<u16>().expect("numeric package version"))
        .collect::<Vec<_>>();
    version_parts.resize(4, 0);
    let numeric_version = version_parts
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let icon = std::path::Path::new(&manifest_dir)
        .join("assets/hexer.ico")
        .display()
        .to_string()
        .replace('\\', "/");
    let out_dir = env::var("OUT_DIR").expect("Cargo build output directory");
    let resource_file = std::path::Path::new(&out_dir).join("hexer.rc");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let resource = format!(
        r#"#pragma code_page(65001)
1 ICON "{icon}"
1 VERSIONINFO
FILEOS 0x40004
FILEVERSION {numeric_version}
FILEFLAGS 0x0
FILETYPE 0x1
FILESUBTYPE 0x0
FILEFLAGSMASK 0x3f
PRODUCTVERSION {numeric_version}
{{
BLOCK "StringFileInfo"
{{
BLOCK "040904b0"
{{
VALUE "FileDescription", "{description}"
VALUE "InternalName", "hexer"
VALUE "FileVersion", "{version}"
VALUE "OriginalFilename", "hexer.exe"
VALUE "ProductName", "Hexer"
VALUE "ProductVersion", "{version}"
}}
}}
BLOCK "VarFileInfo"
{{
VALUE "Translation", 0x0409, 0x04b0
}}
}}
"#
    );
    fs::write(&resource_file, resource).expect("write generated Windows resources");

    embed_resource::compile(resource_file, embed_resource::NONE)
        .manifest_required()
        .expect("compile Windows app resources");
}
