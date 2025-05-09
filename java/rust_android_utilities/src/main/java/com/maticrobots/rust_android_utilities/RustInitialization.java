package com.maticrobots.rust_android_utilities;

import android.content.Context;

import androidx.annotation.NonNull;
import androidx.startup.Initializer;

import java.util.Collections;
import java.util.List;

public class RustInitialization implements Initializer<Void> {
    private static native void initialize_rust(Context application_context);

    @NonNull
    @Override
    public Void create(@NonNull Context context) {
        System.loadLibrary("rust_android_utilities_setter");

        initialize_rust(context.getApplicationContext());
        return null;
    }

    @NonNull
    @Override
    public List<Class<? extends Initializer<?>>> dependencies() {
        return Collections.emptyList();
    }
}
