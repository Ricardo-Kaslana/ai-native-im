//! Build script for ai-im-server
//! 
//! 优化：只在 proto 文件变化时重新编译

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 只在 proto 文件变化时重新运行
    println!("cargo:rerun-if-changed=../protos/chat.proto");
    println!("cargo:rerun-if-changed=../protos/");
    
    // 编译 protobuf
    prost_build::compile_protos(&["../protos/chat.proto"], &["../protos/"])?;
    
    Ok(())
}
