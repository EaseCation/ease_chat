plugins {
    application
    id("ecbuild.java-conventions")
}

application {
    mainClass = "net.easecation.easechat.Main"
}

dependencies {
    api(libs.netty.http)
}

description = "easechat-client-j"
