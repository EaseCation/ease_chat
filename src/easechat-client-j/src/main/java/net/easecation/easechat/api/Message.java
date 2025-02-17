package net.easecation.easechat.api;

/*
* 封装一条 消息数据
* */
public interface Message {
    String MESSAGE_HELLO = "1h";    //客户端向服务端 握手
    String MESSAGE_CHANNEL = "1c";  // 客户端向服务端 订阅频道
    String MESSAGE_TRANSMIT = "1t"; // 客户端向服务端发送消息
    String MESSAGE_RECEIVE = "1r";  // 服务端 向 客户端发送的数据
    String MESSAGE_DISCONNECT = "1d";  // 客户端主动断开与服务端的连接

    int getMessageLength();

    String getMessageType();
}
