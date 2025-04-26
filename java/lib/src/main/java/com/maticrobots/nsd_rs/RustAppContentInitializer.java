package com.maticrobots.nsd_rs;

import android.content.Context;

import androidx.annotation.NonNull;
import androidx.startup.Initializer;

import java.util.Collections;
import java.util.List;

public class RustAppContentInitializer implements Initializer<Void> {
    public static Context applicationContext;

    @NonNull
    @Override
    public Void create(@NonNull Context context) {
        applicationContext = context.getApplicationContext();
        return null;
    }

    @NonNull
    @Override
    public List<Class<? extends Initializer<?>>> dependencies() {
        return Collections.emptyList();
    }
}
