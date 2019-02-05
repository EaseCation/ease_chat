package net.easecation.easechat.api.message;

import net.easecation.easechat.api.MessageSender;

public class AutoSubChannelMessage extends ChannelMessage {
    private long subscriptionTime;
    private boolean closeAutoSub;
    private MessageSender.AutoSubTimerTask timerTask;

    public AutoSubChannelMessage(String channelName) {
        super(channelName);
    }

    public AutoSubChannelMessage(String channelName, int subscriptionTime) {
        super(channelName, subscriptionTime);
    }

    public AutoSubChannelMessage(String channelName, int subscriptionTime, int subscriptionTimeNS) {
        super(channelName, subscriptionTime, subscriptionTimeNS);
        this.subscriptionTime = subscriptionTime;
    }

    public long getSubscriptionTime() {
        return subscriptionTime;
    }

    public void setCloseAutoSub(boolean closeAutoSub) {
        this.closeAutoSub = closeAutoSub;
    }

    public boolean isCloseAutoSub() {
        return closeAutoSub;
    }

    public MessageSender.AutoSubTimerTask getTimerTask() {
        return timerTask;
    }

    public void setTimerTask(MessageSender.AutoSubTimerTask timerTask) {
        if(this.timerTask == null) this.timerTask = timerTask;
    }
}
