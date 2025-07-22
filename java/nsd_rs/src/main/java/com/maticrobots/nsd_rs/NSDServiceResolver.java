package com.maticrobots.nsd_rs;

import android.net.nsd.NsdManager;
import android.net.nsd.NsdServiceInfo;

import com.maticrobots.rust_android_utilities.RustArcBoxDynAny;

public class NSDServiceResolver implements NsdManager.ResolveListener {

    private final RustArcBoxDynAny rustDiscoveryListener;

    public NSDServiceResolver(RustArcBoxDynAny rustDiscoveryListener) {
        this.rustDiscoveryListener = rustDiscoveryListener;
    }

    private static native void rustOnResolveFailed(long rust_ptr, NsdServiceInfo serviceInfo, int errorCode);

    private static native void rustOnServiceResolved(long rust_ptr, NsdServiceInfo serviceInfo);

    @Override
    public void onResolveFailed(NsdServiceInfo serviceInfo, int errorCode) {
        rustOnResolveFailed(this.rustDiscoveryListener.getRust_ptr(), serviceInfo, errorCode);
    }

    @Override
    public void onServiceResolved(NsdServiceInfo serviceInfo) {
        rustOnServiceResolved(this.rustDiscoveryListener.getRust_ptr(), serviceInfo);
    }
}
