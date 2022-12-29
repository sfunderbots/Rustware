from .zmq_util import *

import multiprocessing
import time
from threading import Thread
from typing import List, Callable, Dict
from google.protobuf.message import Message

from src.util.logger import LOG
import zmq


class ZmqPubSub:
    def __init__(self, pub_socket: str, sub_socket: str, pub_noblock=True):
        self.context = zmq.Context()
        self.pub_socket = pub_socket
        self.sub_socket = sub_socket
        self.pub_topic_map: Dict[str, ZmqPubTopicInfo] = dict()
        self.sub_topic_map: Dict[str, ZmqSubTopicInfo] = dict()
        self.callback_handler_threads: Dict[str, Thread] = dict()
        self.shutdown_event = multiprocessing.Event()
        self.pub_noblock = pub_noblock

    def shutdown(self):
        self.shutdown_event.set()
        for t in self.callback_handler_threads.values():
            t.join()

    def __del__(self):
        self.shutdown()

    def _update_pub_socket_map(self, topic: str, keep_only_last_message: bool):
        if topic not in self.pub_topic_map:
            topic_socket = create_pub_socket(self.context, keep_only_last_message)
            topic_socket.bind(self.pub_socket)
            topic_info = ZmqPubTopicInfo(topic=topic, socket=topic_socket)
            self.pub_topic_map[topic] = topic_info

    def pub(self, msg, topic: str, keep_only_last_message: bool = True):
        self._update_pub_socket_map(topic, keep_only_last_message)
        pub_info = self.pub_topic_map[topic]
        try:
            pub_proto(
                socket=pub_info.socket,
                msg=msg,
                topic=pub_info.topic,
                noblock=self.pub_noblock,
            )
        except zmq.ZMQError:
            LOG.error("ZMQ publisher queue full for topic: {}".format(topic))

    def register_callback(
        self, callback, topic: str, msg_type: Message, keep_only_last_message=True
    ):
        if topic not in self.sub_topic_map:
            socket = create_sub_socket(
                context=self.context,
                topic=topic,
                keep_only_last_message=keep_only_last_message,
            )
            socket.connect(self.sub_socket)
            self.sub_topic_map[topic] = ZmqSubTopicInfo(
                topic=topic, socket=socket, proto_msg_type=msg_type
            )

        self.sub_topic_map[topic].callbacks.append(callback)

    def _handle_callbacks(self, info: ZmqSubTopicInfo, poll_timeout_ms: float) -> bool:
        if info.socket.poll(poll_timeout_ms, zmq.POLLIN):
            try:
                data = recv_proto(
                    socket=info.socket, msg_type=info.proto_msg_type, topic=info.topic
                )
            except zmq.ZMQError:
                return False
            except Exception as e:
                print(e)
                return True

            for c in info.callbacks:
                c(data)
            return True

    def handle_callbacks_asynchronously(self):
        def _run(info: ZmqSubTopicInfo):
            while not self.shutdown_event.is_set():
                self._handle_callbacks(info, poll_timeout_ms=250)

        for topic, info in self.sub_topic_map.items():
            t = Thread(target=_run, kwargs={"info": info}, daemon=True)
            self.callback_handler_threads[topic] = t
            self.callback_handler_threads[topic].start()
