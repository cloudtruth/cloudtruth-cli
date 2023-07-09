use color_eyre::Result;
fn main() -> Result<()> {
    color_eyre::install()?;
    cloudtruth_sdk_codegen::generate_sdk()
}
