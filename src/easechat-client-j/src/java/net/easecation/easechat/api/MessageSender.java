package net.easecation.easechat.api;

import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import io.netty.util.concurrent.Future;
import net.easecation.easechat.api.message.ChannelMessage;
import net.easecation.easechat.api.message.HelloMessage;
import net.easecation.easechat.api.message.ReceiveMessage;
import net.easecation.easechat.api.message.TransmitMessage;


public class MessageSender {
    private Channel channel;

    public Channel getChannel() {
        return channel;
    }

    public MessageSender(Channel channel){
        this.channel = channel;
    }

    /*
    * 发送消息 同步方式 不使用Result处理返回值
    * */
    private boolean sendSyncMessage(Message message){
        try {
            return getChannel().writeAndFlush(message).sync().isSuccess();
        } catch (InterruptedException e) {
            e.printStackTrace();
            return false;
        }
    }

    /*
    * 发送消息 同步方式 使用Result处理返回值
    * */
    private void sendSyncMessage(Message message, Result result){
        try {
            Future future = getChannel().writeAndFlush(message).sync();

            if (result != null) result.handle(future);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    /*
     * 异步发送
     * */

    private void sendAsyncMessage(Message message, Result result){
        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    public boolean sendSyncChannelMessage(ChannelMessage message){
        return sendSyncMessage(message);
    }

    public void sendSyncChannelMessage(ChannelMessage message, Result result){
        sendSyncMessage(message, result);
    }

    public boolean sendSyncHelloMessage(HelloMessage message){
        return sendSyncMessage(message);
    }

    public void sendSyncHelloMessage(HelloMessage message, Result result){
        sendSyncMessage(message, result);
    }

    public boolean sendSyncTransmitMessage(TransmitMessage message){
        return sendSyncMessage(message);
    }

    public void sendSyncTransmitMessage(TransmitMessage message, Result result){
        sendSyncMessage(message, result);
    }

    public void sendAsyncHelloMessage(HelloMessage message, Result result){
        sendAsyncMessage(message, result);
    }

    public void sendAsyncChannelMessage(ChannelMessage message, Result result){
        sendAsyncMessage(message, result);
    }

    public void sendAsyncTransmitMessage(TransmitMessage message, Result result){
        sendAsyncMessage(message, result);
    }

    @FunctionalInterface
    public interface Result {
        void handle(Future future);
    }
}
