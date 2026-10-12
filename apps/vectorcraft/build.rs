//! Windows only: embed the app icon and version info (VERSIONINFO) into `vectorcraft.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `VECTORCRAFT_REQUIRE_WINRES=1` turns it into
//! an error (for release builds).

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/vectorcraft.ico");
    println!("cargo:rerun-if-env-changed=VECTORCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_OPENDRAW");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }
    let mut res = winresource::WindowsResource::new();
    if std::env::var_os("CARGO_FEATURE_OPENDRAW").is_some() {
        // Do not embed VectorCraft's trademarked icon in OpenDraw-branded preview builds.
        res.set("ProductName", "OpenDraw AI (Preview)")
            .set("FileDescription", "OpenDraw AI native vector editor preview")
            .set("CompanyName", "OpenDraw AI contributors")
            .set("LegalCopyright", "Derived from VectorCraft; MIT OR Apache-2.0.")
            .set("OriginalFilename", "OpenDrawAI.exe")
            .set("InternalName", "opendraw-ai-preview");
    } else {
        res.set_icon("../../assets/app-icon/vectorcraft.ico")
            .set("ProductName", "VectorCraft")
            .set("FileDescription", "VectorCraft vector illustration editor")
            .set("CompanyName", "Learning Machines LLC")
            .set("LegalCopyright", "Copyright (c) the VectorCraft authors. MIT OR Apache-2.0.")
            .set("OriginalFilename", "vectorcraft.exe")
            .set("InternalName", "vectorcraft");
    }
    if let Err(e) = res.compile() {
        if std::env::var_os("VECTORCRAFT_REQUIRE_WINRES").is_some() {
            return Err(format!("embedding Windows resources failed: {e}"));
        }
        println!("cargo:warning=vectorcraft.exe built without icon/version resources: {e}");
    }
    Ok(())
}
