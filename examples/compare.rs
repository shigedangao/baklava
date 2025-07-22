use baklava::{InsightFace, Methodology};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cosine, percentage) = InsightFace::new("./Megatron", 2)?
        .prepare_images(
            &["./face1_test.png", "./face2_test.png"],
            "./face2_test.png",
        )?
        .compare_images(Methodology::Median)?;

    println!("cosine: {cosine}");
    println!("percentage: {percentage}");

    Ok(())
}
