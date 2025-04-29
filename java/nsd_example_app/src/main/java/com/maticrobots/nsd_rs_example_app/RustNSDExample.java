package com.maticrobots.nsd_rs_example_app;

import android.content.Context;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.startup.Initializer;

import com.maticrobots.nsd_rs.RustAppContentInitializer;
import com.maticrobots.nsd_rs.RustArcBoxDynAny;

import java.util.Arrays;
import java.util.Collections;
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
        return Arrays.asList(RustAppContentInitializer.class);
    }
}
