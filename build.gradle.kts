// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id("com.android.application") version "8.2.2" apply false
    id("org.jetbrains.kotlin.android") version "1.9.22" apply false
    // 添加 Hilt 插件
    id("com.google.dagger.hilt.android") version "2.51" apply false
    // 添加 Protobuf 插件
    id("com.google.protobuf") version "0.9.4" apply false
}
