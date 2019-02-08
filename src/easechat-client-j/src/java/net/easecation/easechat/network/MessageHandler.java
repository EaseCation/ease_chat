package net.easecation.easechat.network;

import io.netty.channel.*;
import net.easecation.easechat.api.Message;
import net.easecation.easechat.api.message.HelloMessage;
import net.easecation.easechat.api.message.ReceiveMessage;

import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_COMPLETE;
import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_ISSUED;

/*
* 用与处理 消息接收
* */
public class MessageHandler extends SimpleChannelInboundHandler<ReceiveMessage> {

    private final EaseChatClient client;

    MessageHandler(EaseChatClient client){
        this.client = client;
    }

    @Override
    public void userEventTriggered(ChannelHandlerContext ctx, Object evt) throws Exception {
        super.userEventTriggered(ctx, evt);

        if (HANDSHAKE_ISSUED.equals(evt)) {
            client.getLogger().info("正在向 Nexus WebSocket 发送TCP握手");
        }

        if (HANDSHAKE_COMPLETE.equals(evt)) {
            client.getLogger().info("Nexus WebSocket TCP握手完成 即将发送1h握手");

            while (client.getSender() == null) {}

            client.getSender().sendSyncHelloMessage(new HelloMessage(client.getName()), future -> {
                if (future.isSuccess()){
                    client.setHandshake(true);
                    client.getLogger().info("1h握手数据 发送成功 开始发送握手成功前未发送的信息包");
                    for (Message message : client.getInitChannelMessages()){
                        client.getSender().sendSyncMessage(message);
                        client.getLogger().info("补充发送：" + message.toString());
                    }
                    client.getInitChannelMessages().clear();
                } else {
                    client.getLogger().warning("握手失败 即将关闭 EaseChatClient");
                    if (!client.shutdown()) throw new InterruptedException("haha"); // 暴力关闭 -1s
                }
            });

        }
    }

    protected void channelRead0(ChannelHandlerContext ctx, ReceiveMessage msg) throws Exception {
        client.getReceiver().receive(msg);
    }
}