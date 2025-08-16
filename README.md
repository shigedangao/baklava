# Baklava 🍮

A small project that wrap the [InsightFace SDK](https://github.com/deepinsight/insightface) in order to perform face comparison. It uses a fork of InsightFace in order to implement some convenient method to be exposed for Rust that can be found [here](https://github.com/shigedangao/insightface). The implementation is based on the sample provided by the insightface team. [c example link](https://github.com/deepinsight/insightface/blob/master/cpp-package/inspireface/cpp/sample/api/sample_face_comparison.c)

## Usage from crates.io

> [!IMPORTANT]
> This guide required you to download the `insightface` library and to follow the instructions in orders.

1. To use the library you'll need to specify the model that will be used. These models are the same that are used by the InsightFace library. The models can be found [here](https://github.com/HyperInspire/InspireFace?tab=readme-ov-file#resource-package-list)

2. Clone the insightface's fork repository using this [link](https://github.com/shigedangao/insightface) and build the `insightface` library with the command

```sh
cd insightface/cpp-package/inspireface && git clone --recurse-submodules https://github.com/tunmx/inspireface-3rdparty.git 3rdparty
cd insightface/cpp-package/inspireface && bash command/build.sh
```

3. Specify the path to the `insightface` library in your `Cargo.toml` by adding this into your `.cargo/config.toml`:

```toml
[env]
INSIGHTFACE_PATH = "<path to>/insightface/cpp-package/inspireface/build/inspireface-<arch>/InspireFace"
```

Thereafter the library can be add as follows in your `Cargo.toml`:

```toml
[dependencies]
baklava = "0.1.0"
```

Below is a simple example of how to use the library:

```rs
// Should you have a list of images greater than 10. You can pass a chunk_size parameter in order to perform the image preparation concurrently.
let (cosine, percentage) = InsightFace::new("<model>", None)?
    .prepare_images(&[
        "./input1.png",
        "./input2.png",
    ])?
    .prepare_target_image("./target_image.png")?
    .compare_images(Methodology::Mean)?;
```

## Running example

The example can be run by executing the following command:

```sh
cargo run --example compare
```

## Local usage

1. Clone & Initialize submodules

```sh
git clone --recurse-submodules https://github.com/shigedangao/baklava.git

# or

git submodule update --init --recursive
```

2. Initialize insightface submodules dependencies

```sh
cd insightface/cpp-package/inspireface && git clone --recurse-submodules https://github.com/tunmx/inspireface-3rdparty.git 3rdparty
```

3. Replace the `build.rs` by the `build_local.rs`. This `build_local.rs` will build the project locally and copy the resulting `dylib` into the target directory.

4. Build the project

```sh
cargo build
```

5. Make sure that everything works fine by running the unit tests with

```sh
cargo test
```

## Recompile insightface

If you wish to recompile the InsightFace - `rm -rf ./insightface/cpp-package/inspireface/build`

## Compile issues

### Dynamic library not found

These includes error such as `inspireface.h` not found etc...

The linking between the InsightFace and Baklava is done through the `build.rs` script. It automatically build with the targeted architecture and copies the resulting `dylib` into the target directory. Should you encounter any issues try to pass the `DYLIB_INCLUDE_PATH` variable like below.

```sh
DYLD_LIBRARY_PATH=./insightface/cpp-package/inspireface/build/inspire-{arch}/InspireFace/lib cargo build
```

> [!TIP]
>
>  If the `inspireface.h` could still not be found. Then you may try to build the InsightFace library by yourself. You can run the following command within the InsightFace directory.
>
>  ```sh
>  cd insightface/cpp-package/inspireface
>  bash command/build/sh
>  ```
>
>  This will build a dynamic library your current architecture within the `build/inspire-{arch}/InspireFace/lib` directory.
