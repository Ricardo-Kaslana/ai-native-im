fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["../protos/chat.proto"], &["../protos/"])?;
    Ok(())
}
