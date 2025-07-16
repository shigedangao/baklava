use baklava::InsightFace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cosine, percentage) = InsightFace::new("./Megatron")?
        .prepare_images(&["./face1_test.png", "./face2_test.png"])?
        .compare_images()?;

    println!("cosine: {cosine}");
    println!("percentage: {percentage}");

    Ok(())
}
