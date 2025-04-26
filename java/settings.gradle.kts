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

    plugins {
        // Apply the foojay-resolver plugin to allow automatic download of JDKs
        id("org.gradle.toolchains.foojay-resolver-convention") version "0.9.0"
        id("com.android.library") version "8.9.2" apply false
    }

}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS) // Good practice
    repositories {
        google()        // <<-- Add this repository for dependencies like aapt2
        mavenCentral()  // <<-- Add this for general dependencies
        // Add other repositories if needed (e.g., jcenter(), maven("..."))
    }
}



rootProject.name = "nsd_rs"

include("lib") 