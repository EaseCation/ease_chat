package net.easecation.easechat;

import net.easecation.easechat.api.message.AutoSubChannelMessage;
import net.easecation.easechat.api.message.TransmitMessage;
import net.easecation.easechat.network.EaseChatClient;

import java.net.URI;
import java.util.Arrays;
import java.util.Objects;

public class Main {

    public static void main(String[] args) {

        EaseChatClient[] clients = new EaseChatClient[1000];

        //压力测试 统计并显示已成功握手的连接数量
        new Thread(() -> {
            while(true) {
                try {
                    System.out.println("连接成功数：" + Arrays.stream(clients).filter(Objects::nonNull).filter(EaseChatClient::isHandshake).count());
                } catch (Exception e) {
                    e.printStackTrace();
                }
                try {
                    Thread.sleep(1000);
                } catch (InterruptedException e) {
                    e.printStackTrace();
                }
            }
        }).start();

        //压力测试 创建大批量连接
        for (int i = 0; i < clients.length; i++) {
            try {
                clients[i] = startClient();
            } catch (Exception e) {
                e.printStackTrace();
            }
        }

        //压力测试 定时发送消息
        new Thread(() -> {
            while(true) {
                Arrays.stream(clients).filter(Objects::nonNull).filter(EaseChatClient::isHandshake).forEach(client -> {
                    client.getSender().sendSyncTransmitMessage(new TransmitMessage("buglet", "TEST!$$!" + System.currentTimeMillis()));
                });
                try {
                    Thread.sleep(1000);
                } catch (InterruptedException e) {
                    e.printStackTrace();
                }
            }
        }).start();

    }

    private static EaseChatClient startClient() {
        EaseChatClient client = new EaseChatClient("ChinaHDJ", URI.create("wx://ntest.easecation.net:6500"), System.out::println);

        try {
            client.start();
        } catch (Exception e) {
            e.printStackTrace();
            System.exit(0);
        }
        client.getSender().sendSyncChannelMessage(new AutoSubChannelMessage("buglet", 5), f -> client.getLogger().info("已订阅频道！"));
        client.getSender().sendSyncChannelMessage(new AutoSubChannelMessage("lobby/main", 5), f -> client.getLogger().info("已订阅频道！"));

        return client;
    }
}
