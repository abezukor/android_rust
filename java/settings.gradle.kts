pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }

}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS) // Good practice
    repositories {
        google()        // <<-- Add this repository for dependencies like aapt2
        mavenCentral()  // <<-- Add this for general dependencies
    }
}



rootProject.name = "matic-android-utilities"

include(":rust_bluedroid")
include(":rust_bluedroid:bluedroid_example_app")
include(":java_rust_obj")
include(":nsd_rs")
include(":nsd_rs:nsd_example_app")
include(":rust_context_autoinitialization")
