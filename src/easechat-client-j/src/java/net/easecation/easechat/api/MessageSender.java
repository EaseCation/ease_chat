package net.easecation.easechat.api;

import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import io.netty.util.concurrent.Future;


public class MessageSender {
    private Channel channel;

    public Channel getChannel() {
        return channel;
    }

    public MessageSender(Channel channel){
        this.channel = channel;
    }

    /*
     * 发送消息 同步方式
     * */
    public void sendSyncMessage(String message, Result result){
        try {
            result.handle(getChannel().writeAndFlush(new TextWebSocketFrame(message)).sync());
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    /*
    * 发送消息 同步方式 不使用Result处理返回值
    * */
    public boolean sendSyncMessage(Message message){
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
    public void sendSyncMessage(Message message, Result result){
        try {
            Future future = getChannel().writeAndFlush(message).sync();

            if (result != null) result.handle(future);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    /*
    * 发送消息 同步方式 使用字符串。 返回发送是否成功
    * */
    public boolean sendSyncMessage(String message){
        try {
            return getChannel().writeAndFlush(new TextWebSocketFrame(message)).sync().isSuccess();
        } catch (InterruptedException e) {
            e.printStackTrace();
            return false;
        }
    }

    /*
     * 异步发送
     * */
    public void sendAsyncMessage(String message, Result result){
        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    public void sendAsyncMessage(Message message, Result result){
        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    @FunctionalInterface
    public interface  Result {
        void handle(Future future);
    }
}
