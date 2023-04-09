package net.easecation.easechat.api;

import net.easecation.easechat.api.message.ReceiveMessage;
import net.easecation.easechat.network.EaseChatClient;

import java.util.logging.Logger;

/*
* 消息接收器
* */
public class MessageReceiver {
    private Listener listener;
    private final EaseChatClient client;

    public MessageReceiver(EaseChatClient client, Listener listener){
        this.client = client;
        this.listener = listener;
    }

    public void setListener(Listener listener) {
        if (this.listener == null){
            this.listener = listener;
        }
    }

    public void receive(ReceiveMessage message){
        if (listener != null) listener.listen(message);
    }

    /*
    * 实现 Listener 接口处理接数据接收
    * */
    @FunctionalInterface
    public interface Listener{
        void listen(ReceiveMessage logger);
    }
}
