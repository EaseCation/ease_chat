package net.easecation.easechat.network;

import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageCodec;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import net.easecation.easechat.api.Message;
import net.easecation.easechat.api.message.ReceiveMessage;

import java.util.List;

public class MessageCodec extends MessageToMessageCodec<TextWebSocketFrame, Message>{
    @Override
    protected void decode(ChannelHandlerContext channelHandlerContext, TextWebSocketFrame frame, List<Object> list) throws Exception {
        list.add(ReceiveMessage.valueOf(frame.text()));
    }

    @Override
    protected void encode(ChannelHandlerContext channelHandlerContext, Message message, List<Object> list) throws Exception {
        list.add(new TextWebSocketFrame(message.toString()));
    }
}
