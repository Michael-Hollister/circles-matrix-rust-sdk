# syntax=docker/dockerfile:1
FROM docker.io/rust:1.74.0-slim-bookworm
RUN apt-get update && apt-get -y --no-install-recommends install \
    pkg-config \
    libssl-dev \
    wget \
    unzip \
    default-jdk

ENV ANDROID_NDK_VERSION=26.1.10909125

RUN wget https://dl.google.com/android/repository/android-ndk-r26b-linux.zip
RUN unzip android-ndk-r26b-linux.zip
RUN mkdir -p /root/Android/Sdk/ndk/$ANDROID_NDK_VERSION && cp -r android-ndk-r26b/* /root/Android/Sdk/ndk/$ANDROID_NDK_VERSION

RUN wget https://dl.google.com/android/repository/commandlinetools-linux-10406996_latest.zip
RUN unzip commandlinetools-linux-10406996_latest.zip
RUN cp -r cmdline-tools /root/Android/Sdk/ndk/$ANDROID_NDK_VERSION

RUN rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
RUN cargo install cargo-ndk

# Envrionment setup
ENV ANDROID_NDK=/root/Android/Sdk/ndk/$ANDROID_NDK_VERSION
ENV ANDROID_NDK_HOME=/root/Android/Sdk/ndk/$ANDROID_NDK_VERSION
ENV ANDROID_HOME=/root/Android/Sdk/ndk/$ANDROID_NDK_VERSION
ENV ANDROID_API_VERSION=33
ENV NDK_CLANG_VERSION=17
ENV PATH $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

# Build fixes
RUN cp /usr/include/x86_64-linux-gnu/openssl/opensslconf.h /usr/include/openssl/opensslconf.h
RUN yes | /root/Android/Sdk/ndk/$ANDROID_NDK_VERSION/cmdline-tools/bin/sdkmanager --licenses --sdk_root=/root/Android/Sdk/ndk/26.1.10909125/

RUN mkdir /matrix-rust-sdk
RUN mkdir /circles-rust-components-kotlin
# WORKDIR /matrix-rust-sdk/bindings/matrix-sdk-crypto-ffi
ENTRYPOINT [ "bash" ]
