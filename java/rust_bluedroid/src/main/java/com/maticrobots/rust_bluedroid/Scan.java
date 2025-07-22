package com.maticrobots.rust_bluedroid;


import android.annotation.SuppressLint;
import android.bluetooth.le.BluetoothLeScanner;
import android.bluetooth.le.ScanCallback;
import android.bluetooth.le.ScanFilter;
import android.bluetooth.le.ScanResult;
import android.bluetooth.le.ScanSettings;
import android.system.SystemCleaner;
import android.util.Log;

import com.maticrobots.rust_android_utilities.RustArcBoxDynAny;
import com.maticrobots.rust_bluedroid.Optional;

import java.lang.ref.Cleaner;
import java.util.List;
import java.util.NoSuchElementException;

public class Scan implements AutoCloseable {
    private static final Cleaner cleaner = SystemCleaner.cleaner();
    private final String TAG = "Rust Bluetooth Scanner";

    Optional<ScanInner> scanner;

    public Scan(BluetoothLeScanner scanner, List<ScanFilter> filters, RustArcBoxDynAny rust_obj) {
        Log.v(TAG, "Starting Rust Bluetooth Scan");


        LEScanCallback callback = new LEScanCallback(rust_obj);
        if (filters != null) {
            ScanSettings settings = new ScanSettings.Builder().build();
            scanner.startScan(filters, settings, callback);
        } else {
            scanner.startScan(callback);
        }

        Optional<ScanInner> inner = new Optional<>(new ScanInner(scanner, callback));

        cleaner.register(this, () -> stop(inner));

        this.scanner = inner;

    }

    private static void stop(Optional<ScanInner> possibleScanner) {
        try {
            ScanInner scanner = possibleScanner.get().orElseThrow();
            scanner.scanner.stopScan(scanner.callback);
            possibleScanner.clear();
        } catch (NoSuchElementException ignored) {

        }
    }

    @Override
    public void close() throws Exception {
        stop(scanner);
    }
}

class ScanInner {
    public BluetoothLeScanner scanner;
    public LEScanCallback callback;

    public ScanInner(BluetoothLeScanner scanner, LEScanCallback callback) {
        this.scanner = scanner;
        this.callback = callback;
    }
}

class LEScanCallback extends ScanCallback {
    RustArcBoxDynAny rust_obj;

    public LEScanCallback(RustArcBoxDynAny rust_obj) {
        this.rust_obj = rust_obj;
    }

    private static native void processScanResult(RustArcBoxDynAny rust_obj, ScanResult scanResult);
    private static native void processScanError(RustArcBoxDynAny rust_obj, int errorCode);

    @Override
    public void onScanResult(int callbackType, ScanResult result) {
        processScanResult(rust_obj,  result);
        super.onScanResult(callbackType, result);
    }

    @Override
    public void onBatchScanResults(List<ScanResult> results) {

        for (ScanResult result : results) {
            processScanResult(rust_obj,  result);
        }
        super.onBatchScanResults(results);
    }

    @Override
    public void onScanFailed(int errorCode) {
        processScanError(rust_obj, errorCode);
        super.onScanFailed(errorCode);
    }

}

