package com.maticrobots.nsd_rs;

import android.net.nsd.NsdManager;
import android.net.nsd.NsdServiceInfo;

public class NSDDiscoveryListener implements NsdManager.DiscoveryListener {
    private final RustArcBoxDynAny rustDiscoveryListener;

    public NSDDiscoveryListener(RustArcBoxDynAny rustDiscoveryListener) {
        this.rustDiscoveryListener = rustDiscoveryListener;
    }

    private static native void rustOnStartDiscoveryFailed(String serviceType, int errorCode);

    private static native void rustOnStopDiscoveryFailed(String serviceType, int errorCode);

    private static native void rustOnDiscoveryStarted(String serviceType);

    private static native void rustOnDiscoveryStopped(String serviceType);

    private static native void rustOnServiceFound(NsdServiceInfo serviceInfo, long rust_ptr);

    private static native void rustOnServiceLost(NsdServiceInfo serviceInfo);

    @Override
    public void onStartDiscoveryFailed(String serviceType, int errorCode) {
        rustOnStartDiscoveryFailed(serviceType, errorCode);
    }

    @Override
    public void onStopDiscoveryFailed(String serviceType, int errorCode) {
        rustOnStopDiscoveryFailed(serviceType, errorCode);
    }

    @Override
    public void onDiscoveryStarted(String serviceType) {
        rustOnDiscoveryStarted(serviceType);
    }

    @Override
    public void onDiscoveryStopped(String serviceType) {
        rustOnDiscoveryStopped(serviceType);
    }

    @Override
    public void onServiceFound(NsdServiceInfo serviceInfo) {
        rustOnServiceFound(serviceInfo, this.rustDiscoveryListener.getRust_ptr());
    }

    @Override
    public void onServiceLost(NsdServiceInfo serviceInfo) {
        rustOnServiceLost(serviceInfo);
    }

}
