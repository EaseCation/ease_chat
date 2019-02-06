package net.easecation.easechat.api;

import java.text.SimpleDateFormat;
import java.util.Date;

public class SimpleLogger implements Logger {

    @Override
    public void emergency(String message) {
        System.out.println(getTimeFormat() + "[EMERGENCY] " + message);
    }

    @Override
    public void alert(String message) {
        System.out.println(getTimeFormat() + "[ALERT] " + message);
    }

    @Override
    public void critical(String message) {
        System.out.println(getTimeFormat() + "[CRITICAL] " + message);
    }

    @Override
    public void error(String message) {
        System.out.println(getTimeFormat() + "[ERROR] " + message);
    }

    @Override
    public void warning(String message) {
        System.out.println(getTimeFormat() + "[WARNING] " + message);
    }

    @Override
    public void notice(String message) {
        System.out.println(getTimeFormat() + "[NOTICE] " + message);
    }

    @Override
    public void info(String message) {
        System.out.println(getTimeFormat() + "[INFO] " + message);
    }

    @Override
    public void debug(String message) {
        System.out.println(getTimeFormat() + "[DEBUG] " + message);
    }

    private String getTimeFormat() {
        Date now = new Date();
        return new SimpleDateFormat("HH:mm:ss ").format(now);
    }
}
