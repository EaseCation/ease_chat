package net.easecation.easechat.network;

import io.netty.channel.*;
import io.netty.handler.codec.http.websocketx.*;
import net.easecation.easechat.api.message.HelloMessage;
import net.easecation.easechat.api.message.ReceiveMessage;
import net.easecation.easechat.api.message.TransmitMessage;

import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_COMPLETE;
import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_ISSUED;

/*
* 用与处理 消息接收
* */
public class MessageHandler extends SimpleChannelInboundHandler<ReceiveMessage> {

    private EaseChatClient client;

    MessageHandler(EaseChatClient client){
        this.client = client;
    }

    @Override
    public void userEventTriggered(ChannelHandlerContext ctx, Object evt) throws Exception {
        super.userEventTriggered(ctx, evt);

        if (HANDSHAKE_ISSUED.equals(evt)) {
            System.out.println("正在向Nexus WebSocket 发送TCP握手");
        }

        if (HANDSHAKE_COMPLETE.equals(evt)) {
            System.out.println("Nexus WebSocket TCP握手完成 即将发送1h握手");

            client.getSender().sendSyncMessage(new HelloMessage(client.getName()), future -> {
                if (future.isSuccess()){
                    System.out.println("1h握手数据 发送成功");
                }else {
                    System.out.println("握手失败 即将关闭 EaseChatClient");
                    if (!client.shutdown()) System.exit(0); // 暴力关闭 -1s
                }
            });
        }
    }

    protected void channelRead0(ChannelHandlerContext ctx, ReceiveMessage msg) throws Exception {


        ctx.channel().writeAndFlush(new TextWebSocketFrame(new TransmitMessage("c/lobby", "xxxxxxxxx").toString()));
    }

}