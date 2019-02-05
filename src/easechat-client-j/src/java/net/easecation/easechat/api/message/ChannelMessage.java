package net.easecation.easechat.api.message;

import net.easecation.easechat.api.Message;

/*
* 订阅消息封装
* */
public class ChannelMessage implements Message {

    private String channelName;
    private int subscriptionTime;
    private int subscriptionTimeNS;

    public final static int DEFAULT_SUBSCRIPTION_TIME = 300; //默认订阅时间 五分钟

    public final static int DEFAULT_SUBSCRIPTION_TIME_NS = 0; //默认订阅时间（纳秒）0

    public ChannelMessage(String channelName){
        this(channelName, DEFAULT_SUBSCRIPTION_TIME);
    }

    public ChannelMessage(String channelName, int subscriptionTime){
        this(channelName, subscriptionTime, DEFAULT_SUBSCRIPTION_TIME_NS);
    }

    public ChannelMessage(String channelName, int subscriptionTime, int subscriptionTimeNS){
        this.channelName = channelName;
        this.subscriptionTime = subscriptionTime;
        this.subscriptionTimeNS = subscriptionTimeNS;
    }

    @Override
    public int getMessageLength() {
        return channelName.getBytes().length;
    }

    @Override
    public String getMessageType() {
        return Message.MESSAGE_CHANNEL;
    }

    @Override
    public String toString() {
        return String.join(
                "|",
                getMessageType(),
                String.valueOf(getMessageLength()),
                this.channelName,
                String.valueOf(this.subscriptionTime),
                String.valueOf(this.subscriptionTimeNS)
        );
    }
}
