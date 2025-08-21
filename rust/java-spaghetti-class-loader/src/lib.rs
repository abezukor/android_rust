use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
    ops::DerefMut,
    sync::{LazyLock, Mutex},
};

use java_spaghetti::{
    ByteArray, Global, Local, PrimitiveArray, sys::JNINativeMethod as RawJNINativeMethod,
};
use java_spaghetti_context::get_application_context;
use log::trace;

#[allow(mismatched_lifetime_syntaxes)]
#[rustfmt::skip]
mod bindings;

use crate::bindings::{
    android::content::Context,
    dalvik::system::InMemoryDexClassLoader,
    java::{
        lang::{Class, ClassLoader, String as JString},
        nio::ByteBuffer,
    },
};

struct ClassNativeMethods {
    class: Global<Class>,
    methods: &'static [RawJNINativeMethod],
}

unsafe impl Send for ClassNativeMethods {}

struct ClassLoaderChain {
    loaded_packages: HashSet<&'static str>,
    class_loaders: VecDeque<Global<ClassLoader>>,
    native_methods: HashMap<&'static str, ClassNativeMethods>,
}

static CLASS_LOADERS: LazyLock<Mutex<ClassLoaderChain>> = LazyLock::new(|| {
    // Start with the application context class loader
    let app_context: Global<Context> = unsafe { get_application_context() };
    let app_context_class_loader = app_context.vm().with_env(|env| {
        let app_context = app_context.as_ref(env);
        let cl = app_context.getClassLoader().unwrap().unwrap();
        cl.as_global()
    });
    unsafe { java_spaghetti::Env::<'_>::set_class_loader(app_context_class_loader.as_raw()) }

    let class_loader_chain = ClassLoaderChain {
        loaded_packages: HashSet::from(["com.android"]),
        class_loaders: VecDeque::from([app_context_class_loader]),
        native_methods: HashMap::new(),
    };
    Mutex::new(class_loader_chain)
});

/// Loads a new package into the java_spaghetti class loader
pub fn load_bytecode(package: &'static str, bytecode: &'static [u8]) {
    let mut class_loader_chain = CLASS_LOADERS.lock().unwrap();
    if !class_loader_chain.loaded_packages.insert(package) {
        // We already loaded this package
        return;
    }
    trace!("Loading dex file for {}. ", package,);

    let last_loader = class_loader_chain.class_loaders.back().unwrap();

    let new_loader = last_loader.vm().with_env(|env| {
        let last_loader = last_loader.as_ref(env);

        let java_bytecode = ByteArray::new_from(
            env,
            &bytecode
                .iter()
                .map(|elem| elem.cast_signed())
                .collect::<Vec<i8>>(),
        );

        let java_bytecode = ByteBuffer::wrap_byte_array(env, java_bytecode)
            .unwrap()
            .unwrap();

        let new_loader =
            InMemoryDexClassLoader::new_ByteBuffer_ClassLoader(env, java_bytecode, last_loader)
                .unwrap();
        let new_loader: Local<ClassLoader> = new_loader.cast().unwrap();
        new_loader.as_global()
    });

    unsafe { java_spaghetti::Env::<'_>::set_class_loader(new_loader.as_raw()) }

    class_loader_chain
        .native_methods
        .iter()
        .for_each(|(_, clm)| register_native_methods(&new_loader, clm));

    class_loader_chain.class_loaders.push_back(new_loader);
}

pub fn declare_native_class_methods(class: &'static str, methods: &'static [RawJNINativeMethod]) {
    let mut class_loader_chain = CLASS_LOADERS.lock().unwrap();

    let ClassLoaderChain {
        loaded_packages: _,
        class_loaders,
        native_methods,
    } = class_loader_chain.deref_mut();
    let last_loader = class_loaders.back().unwrap();

    let native_methods_for_class = native_methods.entry(class);
    if matches!(native_methods_for_class, Entry::Occupied(_)) {
        // Already Registered methods for that class
        return;
    }

    let class_native_methods = ClassNativeMethods {
        class: {
            last_loader.vm().with_env(|env| {
                let last_loader = last_loader.as_ref(env);
                let class_name = JString::from_env_str(env, class);
                last_loader
                    .loadClass(class_name)
                    .unwrap()
                    .unwrap()
                    .as_global()
            })
        },
        methods,
    };
    register_native_methods(last_loader, &class_native_methods);
    native_methods_for_class.or_insert(class_native_methods);
}

fn register_native_methods(last_loader: &Global<ClassLoader>, methods: &ClassNativeMethods) {
    last_loader.vm().with_env(|env| unsafe {
        let class = methods.class.as_ref(env);

        let env = env.as_raw();

        ((**env).v1_2.RegisterNatives)(
            env,
            class.as_raw(),
            methods.methods.as_ptr(),
            methods.methods.len().try_into().unwrap(),
        )
    });
}
