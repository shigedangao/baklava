# Baklava (toy project) - 🍮

A small project to do some face checking in Rust by using the [InsightFace SDK](https://github.com/deepinsight/insightface). It uses a fork of InsightFace in order to implement some convenient method to be exposed for Rust that can be found [here](https://github.com/shigedangao/insightface). The implementation is based on the sample provided by the insightface team. [c example link](https://github.com/deepinsight/insightface/blob/master/cpp-package/inspireface/cpp/sample/api/sample_face_comparison.c)

## Config

Configure the environment two environment variables which target the include & dylib file with the `config.toml`

## Run

```sh
DYLD_LIBRARY_PATH=<dylib path> cargo run --example compare
```