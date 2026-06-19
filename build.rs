fn main() {
    // 添加 Swift 运行时库的 rpath，使 @rpath/libswift_Concurrency.dylib 等可被找到
    // screencapturekit 的 Swift FFI 部分会链接这些动态库
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    // macOS 26+ / Swift 6+ 将 libswift_Concurrency.dylib 移到了 Swift 5.5 兼容目录
    // 检查 Xcode toolchain 路径
    let xcrun_output = std::process::Command::new("xcrun").args(["--show-sdk-path"]).output().ok();
    if let Some(output) = xcrun_output {
        if output.status.success() {
            let swift55_path =
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx";

            if std::path::Path::new(swift55_path).join("libswift_Concurrency.dylib").exists() {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", swift55_path);
            }
        }
    }
}
