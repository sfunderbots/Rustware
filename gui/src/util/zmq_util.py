from src.constants import ZMQ_BASE_UNIX_SOCKET
import zmq


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
    socket.setsockopt_string(zmq.SUBSCRIBE, "")
    return socket


def pub_proto(socket: zmq.Socket, msg, noblock=True):
    raw_data = msg.SerializeToString()
    socket.send(data=raw_data, flags=zmq.NOBLOCK if noblock else 0)


def recv_proto(socket: zmq.Socket, msg_type):
    raw_data = socket.recv()
    msg = msg_type()
    try:
        msg.ParseFromString(raw_data)
        return msg
    except Exception as e:
        print("Failed to parse proto")
    return None


def socket_interface_from_topic(topic: str):
    return ZMQ_BASE_UNIX_SOCKET + topic
