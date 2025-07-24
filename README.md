# Baklava (toy project) - 🍮

A small project to do some face checking in Rust by using the [InsightFace SDK](https://github.com/deepinsight/insightface). It uses a fork of InsightFace in order to implement some convenient method to be exposed for Rust that can be found [here](https://github.com/shigedangao/insightface). The implementation is based on the sample provided by the insightface team. [c example link](https://github.com/deepinsight/insightface/blob/master/cpp-package/inspireface/cpp/sample/api/sample_face_comparison.c)

## Install

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

3. Build the project

```sh
cargo build
```

## Run

```sh
cargo run --example compare
```

If cargo run fail then you can try to specify the `DYLIB_INCLUDE_PATH` variable like below

```sh
DYLD_LIBRARY_PATH=./insightface/cpp-package/inspireface/build/inspire-{arch}/InspireFace/lib cargo run --example compare
```
