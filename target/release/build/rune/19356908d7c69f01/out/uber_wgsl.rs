/// clip.wgsl and shader.wgsl.
pub(crate) const CLASSIC: &str = include_str!(concat!(env!("OUT_DIR"), "/uber_classic.wgsl"));
/// BLAKE3 of [`CLASSIC`], which only macOS ships a library for.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const CLASSIC_BLAKE3: &str = "e260b2888c49267504db720de7e1ad6c91ca9ff2e9c1aebc63c471555011d075";
/// Classic with outline.wgsl's glyph branch, then outline.wgsl.
pub(crate) const WINDFOIL: &str = include_str!(concat!(env!("OUT_DIR"), "/uber_windfoil.wgsl"));
/// BLAKE3 of [`WINDFOIL`], which Windows and macOS ship bytecode for.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) const WINDFOIL_BLAKE3: &str = "f4dd09f032914db84bf2a11afb8f375653a830409265cd9b2233e910419654a5";
