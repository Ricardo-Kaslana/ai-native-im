//! Build script for ai-im-server
//! 
//! 优化：只在 proto 文件变化时重新编译

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 只在 proto 文件变化时重新运行
    println!("cargo:rerun-if-changed=../protos/chat.proto");
    println!("cargo:rerun-if-changed=../protos/");
    println!("cargo:rerun-if-changed=../protos/include");

    // 如果设置了 PROTOC 环境变量，则使用它
    let protoc_path = std::env::var("PROTOC")
        .unwrap_or_else(|_| "./protoc.exe".to_string());

    prost_build::Config::new()
        .protoc_executable(&protoc_path)
        .compile_protos(
            &["../protos/chat.proto"],
            &["../protos/", "../protos/include"]
        )?;

    Ok(())
}
