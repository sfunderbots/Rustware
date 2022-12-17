import logging
from pathlib import Path
from multiprocessing import Queue
from logging.handlers import QueueHandler, QueueListener

# from src.communication.zmq_util import *

# log_queue = Queue(maxsize=-1)
#
#
# class ZmqHandler(logging.Handler):
#     def __init__(self, interface: str):
#         logging.Handler.__init__(self)
#         self.interface = interface
#         self.context = zmq.Context()
#         self.socket = create_pub_socket(context=self.context)
#         self.socket.bind(self.interface)
#
#     def emit(self, record: logging.LogRecord) -> None:
#         try:
#             pub_obj(self.socket, (record.levelname, self.format(record)))
#         except zmq.ZMQError:
#             # TODO: Will this create a feedback loop that continues to overflow the queue for
#             # the socket?
#             LOG.warning(
#                 "ZMQ publisher queue full for interface: {}".format(self.interface)
#             )
#
#
# class UnderbotsFormatter(logging.Formatter):
#     def __init__(self, fmt: str = "%(levelno)s: %(msg)s", datefmt=None):
#         logging.Formatter.__init__(self, fmt, datefmt)
#
#     def format(self, record: logging.LogRecord) -> str:
#         format_orig = self._style._fmt
#
#         if record.levelno == logging.DEBUG:
#             # Show the file:line when debugging
#             self._style._fmt = (
#                 "%(asctime)s - %(filename)s:%(lineno)s - %(levelname)s - %(message)s"
#             )
#
#         result = logging.Formatter.format(self, record)
#
#         self._style._fmt = format_orig
#
#         return result
#
#
# def setup_logger():
#     logger = logging.getLogger("Underbots")
#
#     # In order to safely log from multiple processes, the main logger
#     # used by the code simply puts all messages in a multiprocessing.Queue.
#     # In the main process, we run a QueueListener to handle all the incoming
#     # logs. This is also required for the zmq handler to work, since the
#     # context/socket can't be passed across the fork() boundary.
#
#     queue_handler = QueueHandler(log_queue)
#     logger.addHandler(queue_handler)
#     logger.setLevel(logging.DEBUG)
#
#     formatter = UnderbotsFormatter("%(asctime)s - %(levelname)s - %(message)s")
#
#     zmq_handler = ZmqHandler(interface=socket_interface_from_topic("logs"))
#     zmq_handler.setLevel(logging.DEBUG)
#     # Show less time info in zmq, since this will be shown in the GUI and we
#     # need to preserve space
#     zmq_formatter = UnderbotsFormatter(
#         "%(asctime)s - %(levelname)s - %(message)s", "%H:%M:%S"
#     )
#     zmq_handler.setFormatter(zmq_formatter)
#
#     stream_handler = logging.StreamHandler()
#     stream_handler.setLevel(logging.DEBUG)
#     stream_handler.setFormatter(formatter)
#
#     LOGFILE = Path(__file__).parent.parent / "underbots.log"
#     file_handler = logging.FileHandler(str(LOGFILE))
#     file_handler.setFormatter(formatter)
#
#     queue_listener = QueueListener(
#         log_queue,
#         stream_handler,
#         file_handler,
#         zmq_handler,
#         respect_handler_level=False,
#     )
#     queue_listener.start()
#     return queue_listener
#
#
# QUEUE_LISTENER = setup_logger()
LOG = logging.getLogger("Underbots")
