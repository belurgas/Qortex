fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .out_dir("src/")
        .compile_protos(
            &[
                "proto/helloworld.proto",
                "proto/test.proto",
                "proto/prompt_service.proto",
                "proto/ai_service.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}