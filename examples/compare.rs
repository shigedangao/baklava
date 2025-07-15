use baklava::InsightFace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = InsightFace::new("./Megatron")?
        .prepare_images(&["./face1_test.png", "./face2_test.png"])?
        .compare_images()?;

    println!("{res}");

    Ok(())
}