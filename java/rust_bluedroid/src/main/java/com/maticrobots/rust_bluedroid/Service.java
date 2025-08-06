package com.maticrobots.rust_bluedroid;

import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattService;

import java.util.UUID;

public class Service {
    private final BluetoothGattService service;
    private final GattCallback callback;

    public Service(BluetoothGattService service, GattCallback callback) {
        this.service = service;
        this.callback = callback;
    }

    public UUID uuid() {
        return service.getUuid();
    }

    /**
     * Gets service type
     *
     * @return The type of this service, SERVICE_TYPE_PRIMARY or SERVICE_TYPE_SECONDARY
     */
    public int type() {
        return service.getType();
    }

    public BluetoothGattCharacteristic[] characteristics() {
        return service.getCharacteristics().toArray(BluetoothGattCharacteristic[]::new);
    }

    public Service[] includedServices() {
        return service.getIncludedServices().stream().map(service -> new Service(service, callback)).toArray(Service[]::new);
    }
}
