package net.easecation.easechat.api;

import io.netty.channel.Channel;

import io.netty.util.concurrent.Future;
import net.easecation.easechat.api.message.*;
import net.easecation.easechat.network.EaseChatClient;

import java.util.Timer;
import java.util.TimerTask;

public class MessageSender {

    private final EaseChatClient client;
    private final Channel channel;
    private final Timer timer;

    public Channel getChannel() {
        return channel;
    }

    public MessageSender(EaseChatClient client, Channel channel) {
        this.client = client;
        this.channel = channel;
        this.timer = new Timer();
    }

    public void stopTimer() {
        timer.cancel();
    }

    private void catchHandleAutoSubChannelMessage(Message message) {
        if (message instanceof AutoSubChannelMessage) {
            AutoSubChannelMessage autoSubChannelMessage = (AutoSubChannelMessage) message;
            timer.schedule(new AutoSubTimerTask(autoSubChannelMessage), autoSubChannelMessage.getSubscriptionTime() * 900);
        }
    }

    private boolean checkHandshake(Message message) {
        if (message instanceof HelloMessage) return true;
        if (!client.isHandshake()) {
            client.getInitChannelMessages().add(message);
            return false;
        }
        return true;
    }

    /**
     * 发送消息 同步方式 不使用Result处理返回值
     */
    public boolean sendSyncMessage(Message message) {
        if (!checkHandshake(message)) return true;
        catchHandleAutoSubChannelMessage(message);

        try {
            return getChannel().writeAndFlush(message).sync().isSuccess();
        } catch (InterruptedException e) {
            e.printStackTrace();
            return false;
        }
    }

    /**
     * 发送消息 同步方式 使用Result处理返回值
     */
    public void sendSyncMessage(Message message, Result result) {
        if (!checkHandshake(message)) return;
        catchHandleAutoSubChannelMessage(message);

        try {
            Future future = getChannel().writeAndFlush(message).sync();

            if (result != null) result.handle(future);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    /**
     * 异步发送
     */
    public void sendAsyncMessage(Message message, Result result) {
        if (!checkHandshake(message)) return;
        catchHandleAutoSubChannelMessage(message);

        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    public boolean sendSyncChannelMessage(ChannelMessage message) {
        return sendSyncMessage(message);
    }

    public void sendSyncChannelMessage(ChannelMessage message, Result result) {
        sendSyncMessage(message, result);
    }

    public boolean sendSyncHelloMessage(HelloMessage message) {
        return sendSyncMessage(message);
    }

    public void sendSyncHelloMessage(HelloMessage message, Result result) {
        sendSyncMessage(message, result);
    }

    public boolean sendSyncTransmitMessage(TransmitMessage message) {
        return sendSyncMessage(message);
    }

    public void sendSyncTransmitMessage(TransmitMessage message, Result result) {
        sendSyncMessage(message, result);
    }

    public void sendAsyncHelloMessage(HelloMessage message, Result result) {
        sendAsyncMessage(message, result);
    }

    public void sendAsyncChannelMessage(ChannelMessage message, Result result) {
        sendAsyncMessage(message, result);
    }

    public void sendAsyncTransmitMessage(TransmitMessage message, Result result) {
        sendAsyncMessage(message, result);
    }

    public class AutoSubTimerTask extends TimerTask {

        private AutoSubChannelMessage message;

        public AutoSubTimerTask(AutoSubChannelMessage message) {
            this.message = message;
            message.setTimerTask(this);
        }

        @Override
        public void run() {
            if (message.isCloseAutoSub()) this.cancel();

            MessageSender.this.sendSyncChannelMessage(message);
        }
    }

    @FunctionalInterface
    public interface Result {
        void handle(Future future) throws InterruptedException;
    }
}
