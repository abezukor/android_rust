package com.maticrobots.nsd_rs_example_app;

import android.content.Context;

import androidx.annotation.NonNull;
import androidx.startup.Initializer;

import com.maticrobots.java_rust_obj.RustArcBoxDynAny;
import com.maticrobots.rust_context_autoinitialization.RustInitialization;

import java.util.List;

import kotlin.Unit;

public class RustNSDExample implements Initializer<Unit> {
    public static native void init_logging();

    public static native RustArcBoxDynAny start_discovery();

    @NonNull
    @Override
    public Unit create(@NonNull Context context) {
        System.loadLibrary("example_app_rust_lib");

        return Unit.INSTANCE;
    }

    @NonNull
    @Override
    public List<Class<? extends Initializer<?>>> dependencies() {
        return List.of(RustInitialization.class);
    }
}
