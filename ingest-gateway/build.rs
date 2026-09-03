fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // We only act as a server
        .compile(&["../protos/telemetry.proto"], &["../protos"])?;
    Ok(())
}
