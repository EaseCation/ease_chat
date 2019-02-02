package net.easecation.easechat.network;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.http.*;
import io.netty.handler.codec.http.websocketx.*;

import net.easecation.easechat.api.MessageSender;

import java.net.URI;

public class EaseChatClient {
    private Bootstrap bootstrap;
    private EventLoopGroup loopGroup = new NioEventLoopGroup();
    private Channel channel = null;
    private MessageSender sender;

    private String name;

    String getName() {
        return name;
    }

    /*
    * name 用与 向服务端发起1h握手协议时必须带的参数
    * */
    public EaseChatClient(String name){
        if (name == null || name.isEmpty()) throw new IllegalArgumentException("带个 name 参数啊");
        this.name = name;
    }

    MessageSender getSender() {
        return sender;
    }

    public void start() {
        URI websocketURI = URI.create("wx://localhost:6500");
        bootstrap = new Bootstrap();
        bootstrap
                .group(loopGroup)
                .channel(NioSocketChannel.class)
                .option(ChannelOption.RCVBUF_ALLOCATOR, new FixedRecvByteBufAllocator(1024 * 1024))
                .handler(new ChannelInitializer<NioSocketChannel>() {
                    @Override
                    protected void initChannel(NioSocketChannel channel) throws Exception {
                        channel.pipeline()
                                .addLast(new HttpClientCodec())
                                .addLast(new HttpObjectAggregator(65535))
                                .addLast(new WebSocketFrameAggregator(65535))
                                .addLast(new WebSocketClientProtocolHandler(WebSocketClientHandshakerFactory.newHandshaker(websocketURI, WebSocketVersion.V13,  null, true, new DefaultHttpHeaders())))
                                .addLast(new MessageCodec())
                                .addLast(new MessageHandler(EaseChatClient.this));
                    }
                });
        try {
            this.channel = bootstrap.connect(websocketURI.getHost(), websocketURI.getPort()).sync().channel();
            this.sender = new MessageSender(channel);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }

        new Thread(() -> {
            try {
                Thread.sleep(5000);
                getSender().sendSyncMessage("1c|7|c/lobby|300000|0", future -> System.out.println(future.isSuccess()));
            } catch (InterruptedException e) {
                e.printStackTrace();
            }
        }).start();
    }

    /*
    * 关闭EaseChatClient 同步
    * */
    public boolean shutdown(){
        try {
            return channel.closeFuture().sync().isSuccess();
        } catch (InterruptedException e) {
            e.printStackTrace();
            return false;
        }
    }
}