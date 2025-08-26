use java_spaghetti_class_loader::JNINativeMethod;

pub(super) const GATT_CALLBACK_CLASS_NAME: &str = "com.maticrobots.rust_bluedroid.GattCallback";

pub(super) const LE_SCAN_CALLBACK_NATIVE_METHODS: &[JNINativeMethod] = unsafe {
    &[
    JNINativeMethod::new(
        c"rustOnConnectionChangeState",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;II)Z",
        crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnConnectionChangeState as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnServiceChangedCallback",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;)Z"
,
        crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServiceChangedCallback as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnReadRemoteRssiCallback",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;II)V"
,
        crate::device::rustOnReadRemoteRssiCallback as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnCharacteristicWrite",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;I)V"
,
        crate::characteristic::rust_on_characteristic_write as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnCharacteristicRead",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[BI)V"
,
        crate::characteristic::rust_on_characteristic_read as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnServicesDiscoveredCallback",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[Lcom/maticrobots/rust_bluedroid/Service;I)Z"
,
        crate::device::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnServicesDiscoveredCallback as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnCharacteristicChanged",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[B)Z"
,
        crate::characteristic::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnCharacteristicChanged as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnDescriptorRead",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;[BI)V"
,
        crate::descriptor::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorRead as *mut _,
    ),
    JNINativeMethod::new(
        c"rustOnDescriptorWrite",
        c"(Lcom/maticrobots/java_rust_obj/RustArcBoxDynAny;I)V"
,
        crate::descriptor::Java_com_maticrobots_rust_1bluedroid_GattCallback_rustOnDescriptorWrite as *mut _,
    ),
]
};
