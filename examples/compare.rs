use baklava::{InsightFace, Methodology};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cosine, percentage) = InsightFace::new("./Megatron", Some(3))?
        .prepare_images(&[
            "./face1_test.png",
            "./face1_test.png",
            "./face1_test.png",
            "./face2_test.png",
            "./face2_test.png",
            "./face2_test.png",
        ])?
        .prepare_target_image("./face1_test.png")?
        .compare_images(Methodology::Mean)?;

    println!("cosine: {cosine}");
    println!("percentage: {percentage}");

    Ok(())
}
