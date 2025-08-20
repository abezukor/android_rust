package com.maticrobots.rust_bluedroid;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

import com.maticrobots.java_rust_obj.RustArcBoxDynAny;

public class PairingReceiver extends BroadcastReceiver {

    RustArcBoxDynAny rust_obj;

    public PairingReceiver(RustArcBoxDynAny rust_obj) {
        this.rust_obj = rust_obj;
    }

    private static native boolean rustPairingEvent(
            RustArcBoxDynAny rust_obj,
            int bondState
    );

    @Override
    public void onReceive(Context context, Intent intent) {
        String action = intent.getAction();
        assert action != null;
        if (
                action.equals(
                        android.bluetooth.BluetoothDevice.ACTION_BOND_STATE_CHANGED
                )
        ) {
            android.bluetooth.BluetoothDevice device =
                    intent.getParcelableExtra(
                            android.bluetooth.BluetoothDevice.EXTRA_DEVICE,
                            android.bluetooth.BluetoothDevice.class
                    );
            int bondState = intent.getIntExtra(
                    android.bluetooth.BluetoothDevice.EXTRA_BOND_STATE,
                    -1
            );
            // Rust channel closed
            if (!rustPairingEvent(rust_obj, bondState)) {
                this.abortBroadcast();
            }
        } else {
            throw new RuntimeException(
                    action +
                            " is not a valid event to receive on the pairing BroadcastReceiver"
            );
        }
    }
}
