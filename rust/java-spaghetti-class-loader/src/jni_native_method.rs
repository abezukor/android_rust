use std::ffi::{CStr, c_void};

use java_spaghetti::sys::JNINativeMethod as RawJNINativeMethod;

/// A representation of a Java Native method binding
#[repr(transparent)]
pub struct JNINativeMethod(RawJNINativeMethod);

impl JNINativeMethod {
    /// Create a binding to a JNI Native method
    ///
    /// # Arguments
    /// - `name` -  Name of the Java Method
    /// - `signature` - Signature of the Java method see https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/types.html
    /// - `fn_ptr` - pointer to the rust function that represents the method.
    ///
    /// # Safety
    /// `fn_ptr` must be a pointer to a function whose signature is described by signature.
    ///
    /// # Examples
    /// For
    /// ```java
    /// import com.example.ExampleClass;
    ///
    /// class Foo {
    ///     public static native boolean exampleMethod(J exampleLong, ExampleClass[] exampleClassArray);
    /// }
    /// ```
    /// The corresponding rust definition would be
    /// ```rust
    /// use java_spaghetti::sys::{jobjectArray, jclass};
    ///
    /// use java_spaghetti_class_loader::JNINativeMethod;
    ///
    /// fn example_method(example_long: u64, example_class_array: jobjectArray) -> bool {
    ///     unimplemented!()
    /// }
    ///
    /// pub const NATIVE_METHOD: JNINativeMethod = unsafe {
    ///    JNINativeMethod::new(c"exampleMethod", c"(J[Lcom/example/ExampleClass;)Z", example_method as *mut _)
    /// };
    /// ```
    pub const unsafe fn new(
        name: &'static CStr,
        signature: &'static CStr,
        // I really wish I could take a function pointer here, but rust does not have a way to take function pointers
        // with N arguments.
        fn_ptr: *mut c_void,
    ) -> Self {
        Self(RawJNINativeMethod {
            name: name.as_ptr().cast_mut(),
            signature: signature.as_ptr().cast_mut(),
            fnPtr: fn_ptr,
        })
    }
}
