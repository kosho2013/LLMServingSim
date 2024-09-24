use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "setup/proto/setup.proto"
        ],
        &["setup/proto/"],
    )?;
    Ok(())
}