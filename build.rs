use std::io::Result;

fn main() -> Result<()> {
    // Prost
    let mut builder = prost_build::Config::new();
    builder.default_package_filename("api");
    builder.out_dir("./src/");
    builder.compile_protos(&["protobuf/api.proto"], &["protobuf/"])?;
    // builder.disable_comments(&["."]);
    // builder.enable_type_names();
    builder.protoc_arg("--include_source_info");

    Ok(())
}
