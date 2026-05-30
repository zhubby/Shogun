use vergen_gitcl::{Emitter, Gitcl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let git = Gitcl::builder()
        .branch(true)
        .describe(false, true, None)
        .sha(true)
        .dirty(true)
        .build();

    Emitter::default().add_instructions(&git)?.emit()?;

    Ok(())
}
