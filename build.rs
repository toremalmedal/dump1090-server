use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "server")]
    tonic_prost_build::configure()
        // To allow for optional fields
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(build_path.join("dump1090-server_binary.bin"))
        .compile_protos(&["proto/aircraft.proto"], &["proto"])?;

    #[cfg(feature = "wasm")]
    tonic_prost_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_transport(false)
        .file_descriptor_set_path(build_path.join("dump1090-server_binary.bin"))
        .compile_protos(&["proto/aircraft.proto"], &["proto"])?;
    Ok(())
}
