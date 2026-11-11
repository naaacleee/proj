plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.serialization)
}

android {
    namespace = "com.mentor.core.network"
    compileSdk = 35

    defaultConfig {
        minSdk = 26
        // Адрес бэкенда по умолчанию: эмулятор Android обращается к хостовой
        // машине через 10.0.2.2. Для физического телефона укажите LAN-IP
        // ноутбука в настройках приложения.
        buildConfigField("String", "DEFAULT_BASE_URL", "\"http://10.0.2.2:8080/\"")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation(libs.coroutines.android)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.retrofit)
    implementation(libs.okhttp)
    implementation(libs.serialization.converter)
}
