package net.easecation.easechat.network;

import io.netty.bootstrap.ServerBootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.EventLoopGroup;
import io.netty.channel.SimpleChannelInboundHandler;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.SocketChannel;
import io.netty.channel.socket.nio.NioServerSocketChannel;
import io.netty.handler.codec.http.HttpObjectAggregator;
import io.netty.handler.codec.http.HttpServerCodec;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import io.netty.handler.codec.http.websocketx.WebSocketServerProtocolHandler;
import net.easecation.easechat.api.Logger;
import net.easecation.easechat.api.message.AutoSubChannelMessage;
import net.easecation.easechat.api.message.ChannelMessage;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.net.InetSocketAddress;
import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.function.Predicate;

class EaseChatClientTest {

    private EventLoopGroup bossGroup;
    private EventLoopGroup workerGroup;
    private Channel serverChannel;

    @AfterEach
    void tearDown() {
        if (this.serverChannel != null) {
            this.serverChannel.close().awaitUninterruptibly();
        }
        if (this.bossGroup != null) {
            this.bossGroup.shutdownGracefully().awaitUninterruptibly();
        }
        if (this.workerGroup != null) {
            this.workerGroup.shutdownGracefully().awaitUninterruptibly();
        }
    }

    @Test
    void handshakeCompletesAndFlushesQueuedSubscription() throws Exception {
        BlockingQueue<String> receivedFrames = new LinkedBlockingQueue<>();
        List<String> clientWarnings = new ArrayList<>();
        startServer(receivedFrames);

        int port = ((InetSocketAddress) this.serverChannel.localAddress()).getPort();
        URI uri = URI.create("ws://127.0.0.1:" + port + "/");
        EaseChatClient client = new EaseChatClient(
                "test-client",
                uri,
                new ChannelMessage[]{new AutoSubChannelMessage("lobby/main", 600)},
                message -> {}
        );
        client.setLogger(new CollectingLogger(clientWarnings));

        try {
            client.start();

            String helloFrame = takeMatching(receivedFrames, frame -> frame.startsWith("1h|"));
            String subscriptionFrame = takeMatching(receivedFrames, frame -> frame.startsWith("1c|"));

            Assertions.assertTrue(client.isHandshake(), "client should mark protocol handshake as completed");
            Assertions.assertEquals("1h|11|test-client", helloFrame);
            Assertions.assertEquals("1c|10|lobby/main|600|0", subscriptionFrame);
            Assertions.assertTrue(
                    clientWarnings.stream().noneMatch(message -> message.contains("BlockingOperationException")),
                    "handshake must not block on the Netty event loop"
            );
        } finally {
            client.close("test finished");
        }
    }

    private void startServer(BlockingQueue<String> receivedFrames) throws InterruptedException {
        this.bossGroup = new NioEventLoopGroup(1);
        this.workerGroup = new NioEventLoopGroup(1);
        ServerBootstrap bootstrap = new ServerBootstrap();
        bootstrap.group(this.bossGroup, this.workerGroup)
                .channel(NioServerSocketChannel.class)
                .childHandler(new ChannelInitializer<SocketChannel>() {
                    @Override
                    protected void initChannel(SocketChannel channel) {
                        channel.pipeline()
                                .addLast(new HttpServerCodec())
                                .addLast(new HttpObjectAggregator(65535))
                                .addLast(new WebSocketServerProtocolHandler("/", null, true))
                                .addLast(new SimpleChannelInboundHandler<TextWebSocketFrame>() {
                                    @Override
                                    protected void channelRead0(ChannelHandlerContext context, TextWebSocketFrame frame) {
                                        receivedFrames.offer(frame.text());
                                    }
                                });
                    }
                });
        this.serverChannel = bootstrap.bind("127.0.0.1", 0).sync().channel();
    }

    private String takeMatching(BlockingQueue<String> frames, Predicate<String> predicate) throws InterruptedException {
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
        while (System.nanoTime() < deadline) {
            String frame = frames.poll(100, TimeUnit.MILLISECONDS);
            if (frame != null && predicate.test(frame)) {
                return frame;
            }
        }
        Assertions.fail("Expected frame was not received");
        return "";
    }

    private static class CollectingLogger implements Logger {

        private final List<String> warnings;

        CollectingLogger(List<String> warnings) {
            this.warnings = warnings;
        }

        @Override
        public void emergency(String message) {
            this.warnings.add(message);
        }

        @Override
        public void alert(String message) {
            this.warnings.add(message);
        }

        @Override
        public void critical(String message) {
            this.warnings.add(message);
        }

        @Override
        public void error(String message) {
            this.warnings.add(message);
        }

        @Override
        public void warning(String message) {
            this.warnings.add(message);
        }

        @Override
        public void notice(String message) {
        }

        @Override
        public void info(String message) {
        }

        @Override
        public void debug(String message) {
        }
    }
}
