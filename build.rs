use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>
{
    // Set target information explicitly
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    // Parse target to get OS and arch
    let parts: Vec<&str> = target.split('-').collect();
    let arch = parts.get(0).unwrap_or(&"unknown");
    let os = parts.get(2).unwrap_or(parts.get(1).unwrap_or(&"unknown"));

    println!("cargo:rustc-env=BUILD_ARCH={}", arch);
    println!("cargo:rustc-env=BUILD_OS={}", os);

    // Generate build information using vergen
    let build = vergen_gix::BuildBuilder::all_build()?;
    let cargo = vergen_gix::CargoBuilder::all_cargo()?;
    let gitcl = vergen_gix::GixBuilder::all_git()?;
    let rustc = vergen_gix::RustcBuilder::all_rustc()?;
    let si = vergen_gix::SysinfoBuilder::all_sysinfo()?;

    vergen_gix::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    Ok(())
}
