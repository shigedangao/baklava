use baklava::InsightFace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cosine, percentage) = InsightFace::new("./Megatron")?
        .prepare_images(&["./img1.png", "./img1.png"])?
        .compare_images()?;

    println!("cosine: {cosine}");
    println!("percentage: {percentage}");

    Ok(())
}