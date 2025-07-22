This repository contains various utilities for working with android system APIs from rust. The [Android NDK only supports a limited subset of APIs](https://developer.android.com/ndk/guides/stable_apis), so to implement more things you must use jni to interop with java/kotlin code. In some cases (e.g [`NsdManager.ResolveListener
`](https://developer.android.com/reference/android/net/nsd/NsdManager.ResolveListener)) uses must implement a java interface or create a java subclass. In these cases, to use this library, users must also depend on the various java dependencies.

# Repository Manifest
## [`nsd_rs`](./rust/nsd-rs/README.md)
## [`rust_android_utilities`](./rust/rust_android_utilities/README.md)
## [`bluedroid`] - A medium level crate for using bluetooth on android.