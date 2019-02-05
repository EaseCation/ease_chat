package net.easecation.easechat;

import net.easecation.easechat.api.message.AutoSubChannelMessage;
import net.easecation.easechat.api.message.TransmitMessage;
import net.easecation.easechat.network.EaseChatClient;

import java.net.URI;

public class Main {
    public static void main(String[] args) {
        //DEMO
        EaseChatClient client = new EaseChatClient("ChinaHDJ", URI.create("wx://localhost:6500"), System.out::println);

        client.start();

        new Thread(() -> {
            try {
                Thread.sleep(1000);
            } catch (InterruptedException e) {
                e.printStackTrace();
            }
            client.getSender().sendSyncChannelMessage(new AutoSubChannelMessage("c/lobby", 3000), f -> client.info("已订阅频道！"));
            while(true) {
                client.getSender().sendSyncTransmitMessage(new TransmitMessage("c/lobby", "" + System.currentTimeMillis()));
                try {
                    Thread.sleep(1000);
                } catch (InterruptedException e) {
                    e.printStackTrace();
                }
            }
        }).start();
    }
}
