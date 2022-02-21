package net.easecation.easechat.network;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.http.*;
import io.netty.handler.codec.http.websocketx.*;

import net.easecation.easechat.api.*;
import net.easecation.easechat.api.message.ChannelMessage;
import net.easecation.easechat.api.message.DisconnectMessage;

import java.net.URI;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class EaseChatClient {

    private Bootstrap bootstrap;
    private EventLoopGroup loopGroup = new NioEventLoopGroup();
    private Channel channel = null;
    private boolean isHandshake = false;
    private MessageSender sender;
    private MessageReceiver receiver;
    private Logger logger = new SimpleLogger();
    private URI websocketURI;
    private List<Message> initChannelMessages = new ArrayList<>();

    /*
     * name 用与 向服务端发起1h握手协议时必须带的参数
     * */
    public EaseChatClient(String name, URI websocketURI, MessageReceiver.Listener listener) {
        if (name == null || name.isEmpty()) throw new IllegalArgumentException("带个 name 参数啊");
        this.name = name;
        if (websocketURI == null) {
            throw new IllegalArgumentException("url不能为空");
        }

        this.websocketURI = websocketURI;
        this.receiver = new MessageReceiver(this, listener);
    }

    public EaseChatClient(String name, URI websocketURI, ChannelMessage[] messages, MessageReceiver.Listener listener) {
        this(name, websocketURI, listener);
        this.initChannelMessages.addAll(Arrays.asList(messages));
    }

    public Logger getLogger() {
        return logger;
    }

    public void setLogger(Logger logger) {
        this.logger = logger;
    }

    private String name;

    String getName() {
        return name;
    }

    public MessageSender getSender() {
        return sender;
    }

    MessageReceiver getReceiver() {
        return receiver;
    }

    public boolean isHandshake() {
        return isHandshake;
    }

    public void setHandshake(boolean handshake) {
        isHandshake = handshake;
    }

    public List<Message> getInitChannelMessages() {
        return initChannelMessages;
    }

    public void start() throws Exception {
        bootstrap = new Bootstrap();
        bootstrap
                .group(loopGroup)
                .channel(NioSocketChannel.class)
                .option(ChannelOption.RCVBUF_ALLOCATOR, new FixedRecvByteBufAllocator(1024 * 1024))
                .option(ChannelOption.CONNECT_TIMEOUT_MILLIS, 2000)
                .handler(new ChannelInitializer<NioSocketChannel>() {
                    @Override
                    protected void initChannel(NioSocketChannel channel) {
                        channel.pipeline()
                                .addLast(new HttpClientCodec())
                                .addLast(new HttpObjectAggregator(65535))
                                .addLast(new WebSocketFrameAggregator(65535))
                                .addLast(new WebSocketClientProtocolHandler(WebSocketClientHandshakerFactory.newHandshaker(websocketURI, WebSocketVersion.V13, null, true, new DefaultHttpHeaders())))
                                .addLast(new MessageCodec())
                                .addLast(new MessageHandler(EaseChatClient.this));
                    }
                });
        try {
            this.channel = bootstrap.connect(websocketURI.getHost(), websocketURI.getPort()).sync().channel();
            this.sender = new MessageSender(this, channel);
            getLogger().info("Nexus WebSocket 连接成功");
        } catch (Exception e) {
            getLogger().warning("Nexus WebSocket 连接失败");
            getLogger().warning(e.getMessage());
            throw e;
        }
    }

    /*
     * 关闭EaseChatClient 同步
     * */
    public boolean shutdown() {
        if (this.getSender() != null) {
            this.getSender().sendSyncMessage(new DisconnectMessage("shutdown"));
            try {
                channel.closeFuture().sync().isSuccess();
            } catch (InterruptedException e) {
                e.printStackTrace();
                return false;
            }
        }
        return loopGroup.shutdownGracefully().isSuccess();
    }

    public boolean isActive() {
        return channel != null && channel.isActive();
    }
}