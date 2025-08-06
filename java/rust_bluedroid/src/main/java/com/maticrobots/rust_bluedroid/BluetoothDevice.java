package com.maticrobots.rust_bluedroid;

import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattDescriptor;
import android.bluetooth.BluetoothStatusCodes;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.IntentFilter;
import android.system.SystemCleaner;
import android.util.Log;

import com.maticrobots.rust_android_utilities.RustArcBoxDynAny;

import java.lang.ref.Cleaner;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collection;
import java.util.NoSuchElementException;
import java.util.Queue;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public class BluetoothDevice implements AutoCloseable {

    private static final Cleaner cleaner = SystemCleaner.cleaner();

    public static UUID CCC_DESCRIPTOR = new UUID(
            0x290200001000L,
            0x800000805f9b34fbL
    );
    android.bluetooth.BluetoothDevice device;

    Optional<BluetoothGatt> gatt;

    GattCallback gattCB;

    public BluetoothDevice(
            android.bluetooth.BluetoothDevice device,
            Context context
    ) {
        this.device = device;
        this.gattCB = new GattCallback(this.device.getAddress());
        Optional<BluetoothGatt> bluetoothGatt = new Optional<>(
                this.device.connectGatt(context, false, this.gattCB)
        );
        cleaner.register(this, () -> drop(bluetoothGatt));
        gatt = bluetoothGatt;
    }

    public static void drop(Optional<BluetoothGatt> possibleGatt) {
        try {
            BluetoothGatt gatt = possibleGatt.get().orElseThrow();
            gatt.close();
            possibleGatt.clear();
        } catch (NoSuchElementException ignored) {
        }
    }

    public String id() {
        return device.getAddress();
    }

    public String name() {
        return device.getName();
    }

    public boolean isPaired() {
        return (
                device.getBondState() ==
                        android.bluetooth.BluetoothDevice.BOND_BONDED
        );
    }

    public BroadcastReceiver pair(Context context, RustArcBoxDynAny rust_obj) {
        BroadcastReceiver recv = new PairingReceiver(rust_obj);
        IntentFilter pairingFilter = new IntentFilter(
                android.bluetooth.BluetoothDevice.ACTION_BOND_STATE_CHANGED
        );
        context.registerReceiver(
                recv,
                pairingFilter,
                Context.RECEIVER_EXPORTED
        );

        device.createBond();
        return recv;
    }

    public boolean connect() {
        return this.gatt.get().orElseThrow().connect();
    }

    public void disconnect() {
        this.gatt.get().orElseThrow().disconnect();
    }

    public void connectionStateChange(RustArcBoxDynAny rust_obj) {
        this.gattCB.connectionStateChangeRequests.add(rust_obj);
    }

    public boolean discoverServices(RustArcBoxDynAny rust_obj) {
        this.gattCB.serviceDiscoveredRequests.add(rust_obj);
        boolean toReturn = this.gatt.get().orElseThrow().discoverServices();
        if (!toReturn) {
            this.gattCB.serviceDiscoveredRequests.remove(rust_obj);
        }
        Log.v("BluetoothDevice", "discoverServices " + toReturn);
        return toReturn;
    }

    public void services_changed(RustArcBoxDynAny rust_obj) {
        assert gattCB != null;
        gattCB.serviceChangedRequests.add(rust_obj);
    }

    public boolean rssi(RustArcBoxDynAny rust_obj) {
        gattCB.rssiRequests.add(rust_obj);
        boolean toReturn = gatt.get().orElseThrow().readRemoteRssi();
        if (!toReturn) {
            this.gattCB.rssiRequests.remove(rust_obj);
        }
        return toReturn;
    }

    public boolean readCharacteristic(
            BluetoothGattCharacteristic characteristic,
            RustArcBoxDynAny rust_obj
    ) {
        Collection<RustArcBoxDynAny> callback_objects =
                gattCB.readRequests.computeIfAbsent(characteristic.getUuid(), k ->
                        new ArrayList<>(1)
                );

        // Add the callback to the queue to make sure we will not miss a callback
        callback_objects.add(rust_obj);
        boolean toReturn = gatt
                .get()
                .orElseThrow()
                .readCharacteristic(characteristic);

        // Remove from the queue if the read has already failed.
        if (!toReturn) {
            callback_objects.remove(rust_obj);
        }
        return toReturn;
    }

    public int writeCharacteristic(
            BluetoothGattCharacteristic characteristic,
            RustArcBoxDynAny rust_obj,
            byte[] value,
            int writeType
    ) {
        Queue<RustArcBoxDynAny> callback_objects =
                gattCB.writeRequests.computeIfAbsent(characteristic.getUuid(), k ->
                        new ArrayDeque<>(1)
                );
        callback_objects.add(rust_obj);
        int toReturn = gatt
                .get()
                .orElseThrow()
                .writeCharacteristic(characteristic, value, writeType);
        if (toReturn != BluetoothStatusCodes.SUCCESS) {
            callback_objects.remove(rust_obj);
        }
        return toReturn;
    }

    public boolean enableCharacteristicNotification(
            BluetoothGattCharacteristic characteristic,
            RustArcBoxDynAny rust_obj
    ) {
        Set<RustArcBoxDynAny> characteristic_notifications =
                gattCB.characteristicNotifications.computeIfAbsent(
                        characteristic.getUuid(),
                        k -> ConcurrentHashMap.newKeySet()
                );
        characteristic_notifications.add(rust_obj);
        return gatt
                .get()
                .orElseThrow()
                .setCharacteristicNotification(characteristic, true);
    }

    public boolean readDescriptor(
            BluetoothGattDescriptor descriptor,
            RustArcBoxDynAny rust_obj
    ) {
        Collection<RustArcBoxDynAny> callback_objects =
                gattCB.descriptorReadRequests.computeIfAbsent(
                        descriptor.getUuid(),
                        k -> new ArrayList<>(1)
                );
        callback_objects.add(rust_obj);
        boolean toReturn = gatt.get().orElseThrow().readDescriptor(descriptor);
        if (!toReturn) {
            callback_objects.remove(rust_obj);
        }
        return toReturn;
    }

    public int writeDescriptor(
            BluetoothGattDescriptor descriptor,
            RustArcBoxDynAny rust_obj,
            byte[] value
    ) {
        Queue<RustArcBoxDynAny> callback_objects =
                gattCB.descriptorWriteRequests.computeIfAbsent(
                        descriptor.getUuid(),
                        k -> new ArrayDeque<>(1)
                );
        callback_objects.add(rust_obj);
        int toReturn = gatt
                .get()
                .orElseThrow()
                .writeDescriptor(descriptor, value);
        if (toReturn != BluetoothStatusCodes.SUCCESS) {
            callback_objects.remove(rust_obj);
        }
        return toReturn;
    }

    public Service[] cachedServices() {
        return gatt
                .get()
                .orElseThrow()
                .getServices()
                .stream()
                .map(service -> new Service(service, gattCB))
                .toArray(Service[]::new);
    }

    public int clientConnectionState() {
        return gattCB.clientConnectionState();
    }

    public android.bluetooth.BluetoothDevice getDevice() {
        return device;
    }

    @Override
    public void close() {
        drop(this.gatt);
    }
}
