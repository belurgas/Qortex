fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .out_dir("src/")
        .compile_protos(
            &[
                "proto/ai_service.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}