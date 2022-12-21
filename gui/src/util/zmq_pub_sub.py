import numpy as np
from dataclasses import dataclass, field as dataclass_field
from .zmq_util import *

import multiprocessing
import time
from threading import Thread
from typing import List, Callable, Dict
from google.protobuf.message import Message

import zmq
from .logger import LOG
from collections import deque


@dataclass
class ZmqPubTopicInfo:
    topic: str
    socket: zmq.Socket = None


@dataclass
class ZmqSubTopicInfo:
    topic: str
    socket: zmq.Socket
    proto_msg_type: Message
    callbacks: List[Callable] = dataclass_field(default_factory=list)


class ZmqPubSub:
    def __init__(self, socket_prefix, pub_noblock=True):
        self.context = zmq.Context()
        self.socket_prefix = socket_prefix
        self.pub_topic_map: Dict[str, ZmqPubTopicInfo] = dict()
        self.sub_topic_map: Dict[str, ZmqSubTopicInfo] = dict()
        self.callback_handler_threads: Dict[str, Thread] = dict()
        self.shutdown_event = multiprocessing.Event()

        self.pub_noblock = pub_noblock

    def socket_interface_from_topic(self, topic: str):
        return self.socket_prefix + topic

    def shutdown(self):
        self.shutdown_event.set()
        for t in self.callback_handler_threads.values():
            t.join()

    def __del__(self):
        self.shutdown()

    def _update_pub_socket_map(self, topic: str, keep_only_last_message: bool):
        if topic not in self.pub_topic_map:
            topic_info = ZmqPubTopicInfo(topic=topic)
            topic_socket = create_pub_socket(self.context, keep_only_last_message)
            topic_socket.bind(self.socket_interface_from_topic(topic))
            topic_info.socket = topic_socket
            self.pub_topic_map[topic] = topic_info

    def register_callback(
        self, callback, topic, msg_type: Message, keep_only_last_message=True
    ):
        if topic not in self.sub_topic_map:
            socket = create_sub_socket(self.context, keep_only_last_message)
            socket.connect(self.socket_interface_from_topic(topic))
            self.sub_topic_map[topic] = ZmqSubTopicInfo(
                topic=topic, socket=socket, proto_msg_type=msg_type
            )

        self.sub_topic_map[topic].callbacks.append(callback)

    def _handle_callbacks(self, info: ZmqSubTopicInfo, poll_timeout_ms: float) -> bool:
        if info.socket.poll(poll_timeout_ms, zmq.POLLIN):
            try:
                data = recv_proto(info.socket, info.proto_msg_type)
            except zmq.ZMQError:
                return False
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
