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
        // Add other repositories if needed (e.g., jcenter(), maven("..."))
    }
}



rootProject.name = "nsd_rs"

include("nsd_rs")
include("nsd_example_app")
include("rust_android_utilities")