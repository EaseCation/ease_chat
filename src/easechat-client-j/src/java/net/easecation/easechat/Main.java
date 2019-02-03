package net.easecation.easechat;

import net.easecation.easechat.api.message.ChannelMessage;
import net.easecation.easechat.api.message.HelloMessage;
import net.easecation.easechat.api.message.ReceiveMessage;
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
                Thread.sleep(3000);
            } catch (InterruptedException e) {
                e.printStackTrace();
            }
            client.getSender().sendSyncChannelMessage(new ChannelMessage("c/lobby", 3000));
        }).start();
    }
}
