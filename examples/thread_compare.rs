use baklava::InsightFace;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = InsightFace::new("./Megatron")?;
    let handler_safe = Arc::new(Mutex::new(handler));

    let images = vec![
        ["./face1_test.png", "./face1_test.png"],
        ["./face1_test.png", "./face2_test.png"],
    ];

    let (tx, rx) = std::sync::mpsc::channel();
    let (txc, rxc) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let harc = handler_safe.clone();

        while let Ok(image) = rx.recv() {
            let mut inner = harc.lock().unwrap();

            let (cosine, percentage) = inner
                .prepare_images(&image)
                .expect("Failed to prepare images")
                .compare_images()
                .expect("Failed to compare images");

            txc.send((cosine, percentage))
                .expect("Failed to send results");

            println!("Cosine: {cosine}, Percentage: {percentage}");
        }
    });

    tx.send(images[0]).unwrap();
    tx.send(images[1]).unwrap();
    drop(tx); // Close the channel to signal the thread to finish

    while let Ok((cosine, percentage)) = rxc.recv() {
        println!("Received from thread: Cosine: {cosine}, Percentage: {percentage}");
    }

    Ok(())
}
