package net.easecation.easechat.api.message;

import net.easecation.easechat.api.Message;

/*
* 握手消息 封装
* */
public class HelloMessage implements Message {
    private String text;

    public HelloMessage(String text){
        this.text = text;
    }

    @Override
    public String getMessageType() {
        return Message.MESSAGE_HELLO;
    }

    @Override
    public int getMessageLength() {
        return text.getBytes().length;
    }

    @Override
    public String toString() {
        return  String.join("|", getMessageType(), String.valueOf(getMessageLength()), text);
    }
}
