use java_spaghetti::{ByteArray, Null, PrimitiveArray};

#[allow(mismatched_lifetime_syntaxes)]
mod bindings;
use bindings::{dalvik::system::InMemoryDexClassLoader, java::nio::ByteBuffer};

pub fn load_bytecode(bytecode: &[u8]) {
    let vm = super::get_vm();
    vm.with_env(|env| {
        let java_bytecode = ByteArray::new(env, bytecode.len());
        let java_bytecode = ByteBuffer::wrap_byte_array(env, java_bytecode)
            .unwrap()
            .unwrap();

        let _dex_class_loader =
            InMemoryDexClassLoader::new_ByteBuffer_ClassLoader(env, java_bytecode, Null).unwrap();
    })
}
