plugins {
    application
    id("ecbuild.java-conventions")
}

application {
    mainClass = "net.easecation.easechat.Main"
}

dependencies {
    api(libs.netty.all)
}

description = "easechat-client-j"
