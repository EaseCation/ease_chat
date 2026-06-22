plugins {
    application
    id("ecbuild.java-conventions")
}

application {
    mainClass = "net.easecation.easechat.Main"
}

dependencies {
    api(libs.netty.http)
    testImplementation(libs.jupiter.engine)
}

description = "easechat-client-j"
