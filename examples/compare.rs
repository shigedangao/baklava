use baklava::InsightFace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = InsightFace::new("./Megatron")?
        .prepare_image(&["./img1.png", "./img2.png"])?
        .compare_images()?;

    println!("{res}");

    Ok(())
}