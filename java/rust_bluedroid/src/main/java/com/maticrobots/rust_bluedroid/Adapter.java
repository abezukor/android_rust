package com.maticrobots.rust_bluedroid;

import android.Manifest;
import android.annotation.SuppressLint;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothProfile;
import android.bluetooth.le.BluetoothLeScanner;
import android.bluetooth.le.ScanFilter;
import android.content.Context;

import androidx.annotation.RequiresPermission;

import com.maticrobots.java_rust_obj.RustArcBoxDynAny;

import java.util.List;
import java.util.Set;

public class Adapter {

    private final BluetoothManager manager;
    private final BluetoothAdapter adapter;

    private final Context context;

    public Adapter(Context applicationcontext) {
        manager = (BluetoothManager) applicationcontext.getSystemService(
                Context.BLUETOOTH_SERVICE
        );
        adapter = manager.getAdapter();
        context = applicationcontext;
    }

    @RequiresPermission(Manifest.permission.BLUETOOTH_SCAN)
    public Scan leScan(RustArcBoxDynAny rust_obj) {
        BluetoothLeScanner scanner = adapter.getBluetoothLeScanner();
        if (scanner == null) {
            return null;
        }
        return new Scan(scanner, null, rust_obj);
    }

    @RequiresPermission(Manifest.permission.BLUETOOTH_SCAN)
    public Scan leScan(RustArcBoxDynAny rust_obj, List<ScanFilter> filters) {
        BluetoothLeScanner scanner = adapter.getBluetoothLeScanner();
        if (scanner == null) {
            return null;
        }
        return new Scan(scanner, filters, rust_obj);
    }

    public BluetoothDevice openDevice(String address) {
        android.bluetooth.BluetoothDevice device = adapter.getRemoteDevice(
                address
        );
        return new BluetoothDevice(device, context);
    }

    @SuppressLint("MissingPermission")
    public BluetoothDevice[] getBondedDevices() {
        Set<android.bluetooth.BluetoothDevice> bonded_devices =
                this.adapter.getBondedDevices();
        if (bonded_devices == null) {
            return null;
        }

        return bonded_devices
                .stream()
                .map(this::fromRawDevice)
                .toArray(BluetoothDevice[]::new);
    }

    @SuppressLint("MissingPermission")
    public BluetoothDevice[] getConnectedDevices() {
        List<android.bluetooth.BluetoothDevice> devices =
                this.manager.getConnectedDevices(BluetoothProfile.GATT);
        return devices
                .stream()
                .map(this::fromRawDevice)
                .toArray(BluetoothDevice[]::new);
    }

    @SuppressLint("MissingPermission")
    public int connectionState(BluetoothDevice device) {
        return manager.getConnectionState(
                device.getDevice(),
                BluetoothProfile.GATT
        );
    }

    private BluetoothDevice fromRawDevice(
            android.bluetooth.BluetoothDevice rawDevice
    ) {
        return new BluetoothDevice(rawDevice, context);
    }
}
