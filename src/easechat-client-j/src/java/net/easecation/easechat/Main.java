package net.easecation.easechat;

import net.easecation.easechat.api.message.AutoSubChannelMessage;
import net.easecation.easechat.api.message.TransmitMessage;
import net.easecation.easechat.network.EaseChatClient;

import java.net.URI;

public class Main {
    public static void main(String[] args) {
        //DEMO
        EaseChatClient client = new EaseChatClient("ChinaHDJ", URI.create("wx://localhost:6500"), System.out::println);

        try {
            client.start();
        } catch (Exception e) {
            e.printStackTrace();
            System.exit(0);
        }
        client.getSender().sendSyncChannelMessage(new AutoSubChannelMessage("buglet", 5), f -> client.getLogger().info("已订阅频道！"));
        client.getSender().sendSyncChannelMessage(new AutoSubChannelMessage("lobby/main", 5), f -> client.getLogger().info("已订阅频道！"));
        while(true) {
            //client.getSender().sendSyncTransmitMessage(new TransmitMessage("buglet", "TEST!$$!" + System.currentTimeMillis()));
            try {
                Thread.sleep(1000);
            } catch (InterruptedException e) {
                e.printStackTrace();
            }
        }
    }
}
