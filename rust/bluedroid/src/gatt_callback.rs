use java_spaghetti::sys::JNINativeMethod;

pub(super) const GATT_CALLBACK_CLASS_NAME: &str = "com.maticrobots.rust_bluedroid.GattCallback";
pub(super) const LE_SCAN_CALLBACK_NATIVE_METHODS: &[JNINativeMethod] = &[
    JNINativeMethod {
        name: c"rustOnConnectionChangeState".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;II)Z"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnConnectionChangeState as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnServiceChangedCallback".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;)Z"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServiceChangedCallback as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnReadRemoteRssiCallback".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;II)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::device::rustOnReadRemoteRssiCallback as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnCharacteristicWrite".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;I)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::characteristic::rust_on_characteristic_write as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnCharacteristicRead".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[BI)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::characteristic::rust_on_characteristic_read as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnServicesDiscoveredCallback".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[Lcom/maticrobots/rust_bluedroid/Service;I)Z"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServicesDiscoveredCallback as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnCharacteristicChanged".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[B)Z"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::characteristic::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnCharacteristicChanged as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnDescriptorRead".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[BI)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::descriptor::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorRead as *mut _,
    },
    JNINativeMethod {
        name: c"rustOnDescriptorWrite".as_ptr().cast_mut(),
        signature: c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;I)V"
            .as_ptr()
            .cast_mut(),
        fnPtr: crate::descriptor::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorWrite as *mut _,
    },
];
