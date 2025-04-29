Build with 
```bash
cargo ndk --no-strip -t armeabi-v7a -t arm64-v8a -t x86_64 -o ../app/src/main/jniLibs build --profile=release-with-debug
```