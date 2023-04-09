package net.easecation.easechat.api.message;

import net.easecation.easechat.api.Message;

/*
* 发送消息 封装
* */
public class TransmitMessage implements Message {

    private final String channelName;
    private final String text;

    public TransmitMessage(String channelName, String text){
        if (channelName == null || channelName.isEmpty()){
            throw new IllegalArgumentException("channel 不能为空");
        }

        if (text == null || text.isEmpty()){
            throw new IllegalArgumentException("text 不能为空");
        }

        this.channelName = channelName;
        this.text = text;
    }

    @Override
    public int getMessageLength() {
        return text.getBytes().length;
    }

    @Override
    public String getMessageType() {
        return Message.MESSAGE_TRANSMIT;
    }

    @Override
    public String toString() {
        return String.join(
                "|",
                getMessageType(),
                String.valueOf(this.channelName.getBytes().length),
                this.channelName,
                String.valueOf(getMessageLength()),
                text
        );
    }
}
