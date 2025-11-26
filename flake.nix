{
  description = "Flake for Rust Android Integration Repo";

  inputs = {
    # You can keep using the registry shorthand, but this is the explicit form:
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config = {
          android_sdk.accept_license = true;
          allowUnfree = true;
        };
      };

      androidBuildVersion = "35.0.0";
      androidComposition = pkgs.androidenv.composeAndroidPackages {
        platformVersions = [ "35" ];
        buildToolsVersions = [ androidBuildVersion ];
        cmakeVersions = [ "3.22.1" ];
        includeNDK = true;
        ndkVersions = [ "27.0.12077973" "28.1.13356709" ];
      };

      androidSdk = androidComposition.androidsdk;
      androidSdkPath = "${androidSdk}/libexec/android-sdk";
      jdk17 = pkgs.jdk17;

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [
          "armv7-linux-androideabi"
          "aarch64-linux-android"
          "x86_64-linux-android"
        ];
      };

    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          androidSdk
          jdk17
          glibc
          android-studio
          rustToolchain
          python3Minimal
        ];

        # SDK / NDK
        ANDROID_SDK_ROOT = androidSdkPath;
        ANDROID_HOME = androidSdkPath;
        ANDROID_NDK_ROOT = "${androidSdkPath}/ndk-bundle";
        ANDROID_NDK_HOME = "${androidSdkPath}/ndk-bundle";
        GRADLE_OPTS =
          "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/${androidBuildVersion}/aapt2";

        # Java
        JAVA_HOME = "${jdk17}";

        # Extra PATH + Gradle override
        shellHook = ''
          export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools/bin:$PATH"
        '';
      };
    };

}
