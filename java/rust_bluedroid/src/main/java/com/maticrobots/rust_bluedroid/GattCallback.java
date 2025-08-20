package com.maticrobots.rust_bluedroid;

import static android.bluetooth.BluetoothGattDescriptor.DISABLE_NOTIFICATION_VALUE;
import static com.maticrobots.rust_bluedroid.BluetoothDevice.CCC_DESCRIPTOR;

import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothGattCallback;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattDescriptor;
import android.bluetooth.BluetoothProfile;
import android.util.Log;

import androidx.annotation.NonNull;

import com.maticrobots.java_rust_obj.RustArcBoxDynAny;

import java.util.Arrays;
import java.util.Collection;
import java.util.Hashtable;
import java.util.Map;
import java.util.Queue;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.atomic.AtomicInteger;

public class GattCallback extends BluetoothGattCallback {
    private static final String TAG = "RustGattCallback";
    private final String macAddress;
    public Set<RustArcBoxDynAny> connectionStateChangeRequests = ConcurrentHashMap.newKeySet();
    public ConcurrentLinkedDeque<RustArcBoxDynAny> serviceDiscoveredRequests = new ConcurrentLinkedDeque<>();
    public Set<RustArcBoxDynAny> serviceChangedRequests = ConcurrentHashMap.newKeySet();
    public ConcurrentLinkedDeque<RustArcBoxDynAny> rssiRequests = new ConcurrentLinkedDeque<>();
    public Map<UUID, Collection<RustArcBoxDynAny>> readRequests = new Hashtable<>();
    public Map<UUID, Queue<RustArcBoxDynAny>> writeRequests = new Hashtable<>();
    public ConcurrentHashMap<
            UUID,
            Set<RustArcBoxDynAny>
            > characteristicNotifications = new ConcurrentHashMap<>();
    public Map<UUID, Collection<RustArcBoxDynAny>> descriptorReadRequests = new Hashtable<>();
    public Map<UUID, Queue<RustArcBoxDynAny>> descriptorWriteRequests = new Hashtable<>();
    private AtomicInteger connectionState = new AtomicInteger(BluetoothProfile.STATE_DISCONNECTED);

    public GattCallback(String macAddress) {
        this.macAddress = macAddress;
    }


    private static native boolean rustOnConnectionChangeState(
            RustArcBoxDynAny rust_obj, int status, int new_state
    );

    private static native boolean rustOnServiceChangedCallback(
            RustArcBoxDynAny rust_obj
    );

    private static native void rustOnReadRemoteRssiCallback(
            RustArcBoxDynAny rust_obj,
            int rssi,
            int status
    );

    private static native void rustOnCharacteristicWrite(
            RustArcBoxDynAny rust_obj,
            int status
    );

    private static native void rustOnCharacteristicRead(
            RustArcBoxDynAny rust_obj,
            byte[] value,
            int status
    );

    private static native boolean rustOnServicesDiscoveredCallback(
            RustArcBoxDynAny rust_obj,
            Service[] services,
            int error
    );

    private static native boolean rustOnCharacteristicChanged(
            RustArcBoxDynAny rust_obj,
            byte[] value
    );

    private static native void rustOnDescriptorRead(
            RustArcBoxDynAny rust_obj,
            byte[] value,
            int status
    );

    private static native void rustOnDescriptorWrite(
            RustArcBoxDynAny rust_obj,
            int status
    );

    @Override
    public void onReadRemoteRssi(BluetoothGatt gatt, int rssi, int status) {
        while (!rssiRequests.isEmpty()) {
            RustArcBoxDynAny rust_cb = rssiRequests.removeFirst();
            rustOnReadRemoteRssiCallback(rust_cb, rssi, status);
        }
        super.onReadRemoteRssi(gatt, rssi, status);
    }

    @Override
    public void onCharacteristicWrite(
            BluetoothGatt gatt,
            BluetoothGattCharacteristic characteristic,
            int status
    ) {
        // Android does not provide a guarantee, but this logic assumes that writes for each characteristic are processed
        // in the order they are received
        Queue<RustArcBoxDynAny> characteristicWriteRequests =
                writeRequests.get(characteristic.getUuid());
        if (characteristicWriteRequests == null) {
            return;
        }
        synchronized (characteristicWriteRequests) {
            RustArcBoxDynAny characteristicWriteRequest =
                    characteristicWriteRequests.remove();
            if (characteristicWriteRequest == null) {
                return;
            }
            rustOnCharacteristicWrite(characteristicWriteRequest, status);
        }

        super.onCharacteristicWrite(gatt, characteristic, status);
    }

    @Override
    public void onCharacteristicRead(
            @NonNull BluetoothGatt gatt,
            @NonNull BluetoothGattCharacteristic characteristic,
            @NonNull byte[] value,
            int status
    ) {
        Collection<RustArcBoxDynAny> requests = readRequests.get(
                characteristic.getUuid()
        );
        if (requests == null) {
            return;
        }
        synchronized (requests) {
            for (RustArcBoxDynAny rust_cb : requests) {
                rustOnCharacteristicRead(rust_cb, value, status);
            }
            requests.clear();
        }

        super.onCharacteristicRead(gatt, characteristic, value, status);
    }

    @Override
    public void onServicesDiscovered(BluetoothGatt gatt, int status) {
        Log.v(TAG, "onServicesDiscovered" + status);
        Service[] services = gatt
                .getServices()
                .stream()
                .map(service -> new Service(service, this))
                .toArray(Service[]::new);
        while (!serviceDiscoveredRequests.isEmpty()) {
            RustArcBoxDynAny rust_obj =
                    serviceDiscoveredRequests.removeFirst();
            rustOnServicesDiscoveredCallback(rust_obj, services, status);
        }

        super.onServicesDiscovered(gatt, status);
    }

    @Override
    public void onServiceChanged(@NonNull BluetoothGatt gatt) {
        serviceChangedRequests.removeIf(rust_obj -> !rustOnServiceChangedCallback(rust_obj));
        super.onServiceChanged(gatt);
    }

    @Override
    public void onCharacteristicChanged(
            @NonNull BluetoothGatt gatt,
            @NonNull BluetoothGattCharacteristic characteristic,
            @NonNull byte[] value
    ) {
        Log.v(TAG, "onCharacteristicChanged" + Arrays.toString(value));
        Set<RustArcBoxDynAny> notification_requests =
                characteristicNotifications.get(characteristic.getUuid());

        assert notification_requests != null;
        notification_requests.removeIf(rust_obj -> !rustOnCharacteristicChanged(rust_obj, value));

        if (notification_requests.isEmpty()) {
            gatt.setCharacteristicNotification(characteristic, false);
            BluetoothGattDescriptor descriptor = characteristic.getDescriptor(CCC_DESCRIPTOR);
            gatt.writeDescriptor(descriptor, DISABLE_NOTIFICATION_VALUE);
        }


        super.onCharacteristicChanged(gatt, characteristic, value);
    }

    @Override
    public void onDescriptorRead(
            @NonNull BluetoothGatt gatt,
            @NonNull BluetoothGattDescriptor descriptor,
            int status,
            @NonNull byte[] value
    ) {
        Collection<RustArcBoxDynAny> requests =
                descriptorReadRequests.get(descriptor.getUuid());
        if (requests == null) {
            return;
        }

        synchronized (requests) {
            for (RustArcBoxDynAny rust_cb : requests) {
                rustOnCharacteristicRead(rust_cb, value, status);
            }
            requests.clear();
        }

        super.onDescriptorRead(gatt, descriptor, status, value);
    }

    @Override
    public void onDescriptorWrite(
            BluetoothGatt gatt,
            BluetoothGattDescriptor descriptor,
            int status
    ) {
        // Android does not provide a guarantee, but this logic assumes that writes for each characteristic are processed
        // in the order they are received
        Queue<RustArcBoxDynAny> descriptorWriteRequests =
                this.descriptorWriteRequests.get(descriptor.getUuid());
        assert descriptorWriteRequests != null;
        synchronized (descriptorWriteRequests) {
            RustArcBoxDynAny characteristicWriteRequest =
                    descriptorWriteRequests.remove();
            // Expected to be null when disabling characteristic notifications
            if (characteristicWriteRequest != null) {
                rustOnDescriptorWrite(characteristicWriteRequest, status);
            }
        }

        super.onDescriptorWrite(gatt, descriptor, status);
    }

    @Override
    public void onConnectionStateChange(BluetoothGatt gatt, int status, int newState) {
        Log.v(TAG, this.macAddress + " onConnectionStateChange " + newState);
        connectionState.set(newState);
        connectionStateChangeRequests.removeIf(rust_obj -> !rustOnConnectionChangeState(rust_obj, status, newState));

        super.onConnectionStateChange(gatt, status, newState);
    }


    public int clientConnectionState() {
        return connectionState.getAcquire();
    }


}
