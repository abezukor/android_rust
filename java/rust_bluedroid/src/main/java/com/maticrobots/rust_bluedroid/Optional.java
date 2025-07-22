package com.maticrobots.rust_bluedroid;

import androidx.annotation.Nullable;

public class Optional<T> {

    @Nullable
    private T value;

    public Optional(@Nullable T value) {
        this.value = value;
    }

    public void clear() {
        value = null;
    }

    public java.util.Optional<T> get() {
        return java.util.Optional.ofNullable(value);
    }
}
