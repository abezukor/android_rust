//
// Created by abe on 4/21/25.
//

#include <android/log.h>
#include <jni.h>
#include <stdatomic.h>
#include <stdlib.h>

#define TAG "Rust Android Utilities C Code"

atomic_uintptr_t VM = ATOMIC_VAR_INIT((uintptr_t)NULL);
atomic_uintptr_t APPLICATION_CONTEXT = ATOMIC_VAR_INIT((uintptr_t)NULL);
atomic_uintptr_t CLASS_LOADER = ATOMIC_VAR_INIT((uintptr_t)NULL);

JNIEXPORT void
Java_com_maticrobots_rust_1android_1utilities_RustInitialization_initialize_1rust(
    JNIEnv *env, __attribute__((unused)) jobject this,
    jobject application_context_object) {
  uintptr_t expected_initial_value = (uintptr_t)NULL;
  jint err = 0;
  JavaVM *vm = NULL;

  err = (*env)->GetJavaVM(env, &vm);

  if (err || vm == NULL) {
    __android_log_print(ANDROID_LOG_ERROR, TAG, "Failed to get VM pointer");
    abort();
  }
  atomic_compare_exchange_strong_explicit(&VM, &expected_initial_value,
                                          (uintptr_t)vm, memory_order_release,
                                          memory_order_relaxed);

  __android_log_print(ANDROID_LOG_VERBOSE, TAG,
                      "Getting Global Application Context");
  jobject globalApplicationContext =
      (*env)->NewGlobalRef(env, application_context_object);
  if (globalApplicationContext == NULL) {
    __android_log_print(ANDROID_LOG_ERROR, TAG,
                        "Could not get global application context");
    abort();
  }
  atomic_compare_exchange_strong_explicit(
      &APPLICATION_CONTEXT, &expected_initial_value,
      (uintptr_t)globalApplicationContext, memory_order_release,
      memory_order_relaxed);

  jclass application_context_class =
      (*env)->GetObjectClass(env, application_context_object);
  if (application_context_class == NULL) {
    __android_log_print(ANDROID_LOG_ERROR, TAG,
                        "Could not get application context class");
    abort();
  }
  jmethodID getClassLoader =
      (*env)->GetMethodID(env, application_context_class, "getClassLoader",
                          "()Ljava/lang/ClassLoader;");
  jobject class_loader = (*env)->CallObjectMethodA(
      env, application_context_object, getClassLoader, NULL);
  jthrowable exception = (*env)->ExceptionOccurred(env);
  if (exception != NULL) {
    (*env)->ExceptionClear(env);
    __android_log_print(ANDROID_LOG_ERROR, TAG,
                        "getClassLoader on application context failed");
    abort();
  }
  if (class_loader == NULL) {
    __android_log_print(ANDROID_LOG_ERROR, TAG,
                        "Application Context Class loader is null");
    abort();
  }
  class_loader = (*env)->NewGlobalRef(env, class_loader);
  if (class_loader == NULL) {
    __android_log_print(ANDROID_LOG_ERROR, TAG,
                        "Failed to get global application class loader");
    abort();
  }
  atomic_compare_exchange_strong_explicit(
      &CLASS_LOADER, &expected_initial_value, (uintptr_t)class_loader,
      memory_order_release, memory_order_relaxed);

  __android_log_print(ANDROID_LOG_VERBOSE, TAG, "Initialized Android Rust");
}

__attribute__((unused)) __attribute__((visibility("default"))) jobject
android_rust_initialization_vm() {
  return (jobject)atomic_load_explicit(&VM, memory_order_consume);
}

__attribute__((unused)) __attribute__((visibility("default"))) jobject
android_rust_initialization_application_context() {
  return (jobject)atomic_load_explicit(&APPLICATION_CONTEXT,
                                       memory_order_consume);
}
__attribute__((unused)) __attribute__((visibility("default"))) jobject
android_rust_initialization_class_loader() {
  return (jobject)atomic_load_explicit(&CLASS_LOADER, memory_order_consume);
}