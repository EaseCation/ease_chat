package net.easecation.easechat.api.message;

import net.easecation.easechat.api.Message;

/*
* 消息接收数据 封装
* */
public class ReceiveMessage implements Message {
    private final String text;
    private final String channelName;
    private final String form;

    public static ReceiveMessage valueOf(String source){
        String[] data = source.split("\\|",7);

        if (!data[0].equals(MESSAGE_RECEIVE)){
            throw new IllegalArgumentException("协议头有误");
        }
        if (data[2].getBytes().length != Integer.parseInt(data[1])){
            throw new IllegalArgumentException("协议头有误");
        }

        if (data[4].getBytes().length != Integer.parseInt(data[3])){
            throw new IllegalArgumentException("协议头有误");
        }

        if (data[6].getBytes().length != Integer.parseInt(data[5])){
            throw new IllegalArgumentException("协议头有误");
        }

        return new ReceiveMessage(data[2], data[4], data[6]);
    }

    private ReceiveMessage(String form, String channelName, String text){
        this.form = form;
        this.channelName = channelName;
        this.text = text;
    }

    public String getForm() {
        return form;
    }

    public String getChannelName() {
        return channelName;
    }

    public String getText() {
        return text;
    }

    @Override
    public int getMessageLength() {
        return text.getBytes().length;
    }

    @Override
    public String getMessageType() {
        return Message.MESSAGE_RECEIVE;
    }

    @Override
    public String toString() {
        return String.join(
                "|",
                getMessageType(),
                String.valueOf(form.getBytes().length),
                getForm(),
                String.valueOf(channelName.getBytes().length),
                getChannelName(),
                String.valueOf(text.getBytes().length),
                getText()
        );
    }
}
