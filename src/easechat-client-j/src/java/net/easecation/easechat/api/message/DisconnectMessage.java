package net.easecation.easechat.api.message;

import net.easecation.easechat.api.Message;

/*
* 断开连接 封装
* */
public class DisconnectMessage implements Message {
    private String text;

    public DisconnectMessage(String text){
        this.text = text;
    }

    @Override
    public String getMessageType() {
        return Message.MESSAGE_DISCONNECT;
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
