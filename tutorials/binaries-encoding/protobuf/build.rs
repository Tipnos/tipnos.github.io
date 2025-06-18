use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["schemas/comment.proto"], &["schemas/"])?;
    Ok(())
}
