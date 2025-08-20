package com.maticrobots.java_rust_obj;

import java.lang.ref.Cleaner;
import java.util.concurrent.atomic.AtomicLong;

public class RustArcBoxDynAny implements AutoCloseable {
    private static final String TAG = RustArcBoxDynAny.class.getName();

    /*
    So Java destructors are complicated (and discouraged)
    Essentially there are two ways of doing it
    - the finalize method
    - Cleaner
    The finalize method is pretty simple but is deprecated because it blocks the GC during collection
    The mor modern way of doing things is using a Cleaner Object. The cleaner object gets queued
    on a cleanable thread once the object becomes unreachable (but possible before finalize runs).
    See The following blog posts for details
    https://inside.java/2022/05/25/clean-cleaner/
    https://inside.java/2022/05/27/testing-clean-cleaner-cleanup/

    Essentially if both are registered, there is no deterministic order in which they will run, so
    we just trust that when Cleaner is supported, it will work.
     */
    private static final Cleaner cleaner = Cleaner.create();

    private final Cleaner.Cleanable cleanable;
    private final AtomicLong rust_ptr;

    public RustArcBoxDynAny(long rust_ptr) {
        this.rust_ptr = new AtomicLong(rust_ptr);

        this.cleanable = cleaner.register(this, runRustDestructor(this.rust_ptr));
    }

    private static native long rust_object_clone(long rust_ptr);

    private static Runnable runRustDestructor(AtomicLong rust_ptr) {
        return () -> {
            long current_rust_ptr = rust_ptr.getAndSet(0);
            if (current_rust_ptr != 0) {
                rust_object_destruct(current_rust_ptr);
            } else {
                System.exit(1);
            }

        };
    }

    private static native void rust_object_destruct(long rust_ptr);

    public long getRust_ptr() {
        return rust_ptr.get();
    }

    @Override
    protected Object clone() throws CloneNotSupportedException {
        rust_object_clone(this.rust_ptr.get());
        return super.clone();
    }

    @Override
    public void close() throws Exception {
        cleanable.clean();
    }

}
