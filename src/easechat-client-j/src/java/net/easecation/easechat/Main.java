package net.easecation.easechat;

import net.easecation.easechat.api.message.ChannelMessage;
import net.easecation.easechat.api.message.HelloMessage;
import net.easecation.easechat.api.message.ReceiveMessage;
import net.easecation.easechat.api.message.TransmitMessage;
import net.easecation.easechat.network.EaseChatClient;

public class Main {
    public static void main(String[] args) {
        System.out.println("海星");
        System.out.println("黄德健".getBytes().length);
        System.out.println("HDJ".getBytes().length);
        System.out.println(new TransmitMessage("Channel11", "我套你妈的猴子"));
        for (String s : "1r|9|123456789|7|c/lobby|9|xxxxxxxxx".split("\\|", 7)){
            System.out.println(s);
        }

        System.out.println(ReceiveMessage.valueOf("1r|9|123456789|7|c/lobby|9|xxxxxxxxx"));
        EaseChatClient client = new EaseChatClient("ChinaHDJ");

        client.start();
    }
}
