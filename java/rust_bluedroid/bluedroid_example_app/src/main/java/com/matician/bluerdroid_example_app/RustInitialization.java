package com.matician.bluerdroid_example_app;

import android.content.Context;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.startup.Initializer;

import com.maticrobots.rust_android_utilities.RustArcBoxDynAny;

import java.util.List;

import kotlin.Unit;

public class RustInitialization implements Initializer<Unit> {
    final String TAG = "RustInitialization";

    public static native RustArcBoxDynAny rust_bluetooth_search(String device_mac);

    public static native void test_gatt(RustArcBoxDynAny device);

    @NonNull
    @Override
    public Unit create(@NonNull Context context) {
        System.loadLibrary("bluedroid_example_app");
        Log.d(TAG, "Initialized Rust Library");

        return Unit.INSTANCE;
    }

    @NonNull
    @Override
    public List<Class<? extends Initializer<?>>> dependencies() {
        return List.of(com.maticrobots.rust_android_utilities.RustInitialization.class);
    }
}
