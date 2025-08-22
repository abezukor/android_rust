package com.maticrobots.rust_bluedroid;

public class Optional<T> {

    private T value;

    public Optional(T value) {
        this.value = value;
    }

    public void clear() {
        value = null;
    }

    public java.util.Optional<T> get() {
        return java.util.Optional.ofNullable(value);
    }
}
