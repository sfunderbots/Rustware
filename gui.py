import zmq
import time

IPC_SOCKET = "ipc:///tmp/underbots_zmq_test"

def main():
    print("Hello world")
    context = zmq.Context()
    socket = context.socket(zmq.SUB)
    socket.setsockopt(zmq.SUBSCRIBE, b"")
    socket.connect(IPC_SOCKET)
    poll_timeout_ms = 100

    # pub = context.socket(zmq.PUB)
    # pub.connect(IPC_SOCKET)
    # print("starting poll")

    while True:
        # print("sending data")
        # pub.send_string("hello")
        # print("done sending")
        # time.sleep(0.5)
        # if socket.poll(poll_timeout_ms, zmq.POLLIN):
        try:
            data = socket.recv()
            print("Got data")
        except zmq.ZMQError:
            print("Error")
        # else:
            # print("timeout")


if __name__ == '__main__':
    main()
