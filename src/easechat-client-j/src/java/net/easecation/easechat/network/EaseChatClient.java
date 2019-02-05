package net.easecation.easechat.network;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.http.*;
import io.netty.handler.codec.http.websocketx.*;

import net.easecation.easechat.api.MessageReceiver;
import net.easecation.easechat.api.MessageSender;
import net.easecation.easechat.api.message.ChannelMessage;

import java.net.URI;
import java.util.logging.Logger;

public class EaseChatClient {
    private Bootstrap bootstrap;
    private EventLoopGroup loopGroup = new NioEventLoopGroup();
    private Channel channel = null;
    private MessageSender sender;
    private MessageReceiver receiver;
    private Logger logger;
    private URI websocketURI;
    private ChannelMessage[] initChannelMessages;

    public ChannelMessage[] getInitChannelMessage() {
        if (initChannelMessages == null) return new ChannelMessage[]{};

        ChannelMessage[] channelMessages = initChannelMessages.clone();
        initChannelMessages = null;

        return channelMessages;
    }

    /*
     * name 用与 向服务端发起1h握手协议时必须带的参数
     * */
    public EaseChatClient(String name, URI websocketURI, MessageReceiver.Listener listener){
        if (name == null || name.isEmpty()) throw new IllegalArgumentException("带个 name 参数啊");
        this.name = name;
        if (websocketURI == null){
            throw new IllegalArgumentException("url不能为空");
        }

        this.websocketURI = websocketURI;
        this.receiver = new MessageReceiver(this, listener);
    }

    public EaseChatClient(String name, URI websocketURI, ChannelMessage[] messages, MessageReceiver.Listener listener){
        this(name, websocketURI, listener);

        this.initChannelMessages = messages;
    }

    public Logger getLogger() {
        return logger;
    }

    public void setLogger(Logger logger) {
        this.logger = logger;
    }

    public void info(String info){
        if (logger != null) logger.info(info);
    }

    public void warning(String warnStr) {
        if (logger != null) logger.warning(warnStr);
    }

    private String name;

    String getName() {
        return name;
    }

    public MessageSender getSender() {
        return sender;
    }

    MessageReceiver getReceiver(){
        return receiver;
    }

    public void start() {
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
            info("Nexus WebSocket 连接成功");
        } catch (InterruptedException e) {
            warning("Nexus WebSocket 连接失败");
            warning(e.getMessage());
            e.printStackTrace();
        }
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