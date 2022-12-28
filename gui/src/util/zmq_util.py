import zmq
from dataclasses import dataclass, field as dataclass_field
from google.protobuf.message import Message
from typing import List, Callable, Dict


@dataclass
class ZmqPubTopicInfo:
    topic: str
    proto_msg_type: Message
    socket: zmq.Socket


@dataclass
class ZmqSubTopicInfo:
    topic: str
    socket: zmq.Socket
    proto_msg_type: Message
    callbacks: List[Callable] = dataclass_field(default_factory=list)


def create_pub_socket(
    context: zmq.Context, buffer_size_messages: int = None, keep_only_last_message=False
):
    if buffer_size_messages is not None and keep_only_last_message:
        raise ValueError(
            "Create Pub Socket: keep_only_last_message overrides the buffer_size_messages. Please just specify one or the other."
        )

    socket = context.socket(zmq.PUB)
    if buffer_size_messages is not None and buffer_size_messages > 0:
        socket.setsockopt(zmq.SNDHWM, buffer_size_messages)
    if keep_only_last_message:
        socket.setsockopt(zmq.CONFLATE, 1)
    return socket


def create_sub_socket(
    context: zmq.Context,
    topic: str = "",
    buffer_size_messages: int = None,
    keep_only_last_message=False,
):
    if buffer_size_messages is not None and keep_only_last_message:
        raise ValueError(
            "Create Sub Socket: keep_only_last_message overrides the buffer_size_messages. Please just specify one or the other."
        )
    socket = context.socket(zmq.SUB)
    if buffer_size_messages is not None and buffer_size_messages > 0:
        socket.setsockopt(zmq.RCVHWM, buffer_size_messages)
    if keep_only_last_message:
        socket.setsockopt(zmq.CONFLATE, 1)
    socket.setsockopt_string(zmq.SUBSCRIBE, topic)
    return socket


def pub_proto(socket: zmq.Socket, msg, noblock=True):
    raw_data = msg.SerializeToString()
    socket.send(data=raw_data, flags=zmq.NOBLOCK if noblock else 0)


def recv_proto(socket: zmq.Socket, msg_type, topic: str = "") -> Message:
    raw_data = socket.recv()
    if topic:
        raw_data = raw_data[len(topic) :]
    msg = msg_type()
    msg.ParseFromString(raw_data)
    return msg
